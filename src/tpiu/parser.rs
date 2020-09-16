//! Parses TPIU frames

use std::io::{Read, Error};
use std::collections::VecDeque;
use super::types::*;
use ::utils::readpos::ReadPos;

pub struct Parser {
    input: ReadPos,
    source: TraceSourceID,
    error: Option<Error>,
    buffer: VecDeque<TPIUPacket>,
}

impl Parser {
    pub fn new(input: Box<Read>) -> Parser {
        Parser {
            input: ReadPos::new(input),
            source:TraceSourceID(0),
            error: None,
            buffer: VecDeque::<TPIUPacket>::with_capacity(16),
        }
    }

    pub fn position(&self) -> usize {
        self.input.position()
    }

    pub fn error(&self) -> Option<&Error> {
        match self.error {
            Some(ref e) => Some(&e),
            None => None
        }
    }

    fn parse_frame(&mut self)
    {
        let mut frame: [u8; 16] = [0; 16];
        if let Err(e) = self.input.read(&mut frame) {
            self.error = Some(e);
            return;
        }
        
        let mut data = Vec::<u8>::with_capacity(16);
        let mut i = 0;
        while i < 15 {
            let aux_bit = (frame[15] >> (i / 2)) & 1;

            if (frame[i] & 0x01) == 0 {
                // Two data bytes, lowest bit of first byte is in byte 15
                data.push((frame[i] & 0xFE) | aux_bit);

                if i != 14 {
                    data.push(frame[i+1])
                }
            } else {
                // Source change + one data byte
                if i != 14 && aux_bit == 1 {
                    data.push(frame[i+1]);
                }
                
                if data.len() > 0 {
                    self.buffer.push_back(self.source.to_packet(data));
                    data = Vec::<u8>::with_capacity(16);
                }

                self.source = TraceSourceID(frame[i] >> 1);

                if i != 14 && aux_bit == 0 {
                    data.push(frame[i+1]);
                }
            }

            i += 2;
        }

        if data.len() > 0 {
            self.buffer.push_back(self.source.to_packet(data));
        }
    }
}

impl Iterator for Parser {
    type Item = TPIUPacket;
    fn next(&mut self) -> Option<TPIUPacket> {
        if self.buffer.len() == 0 {
            self.parse_frame();
        }

        self.buffer.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn test_single(v: Vec<u8>, r: Vec<TPIUPacket>) {
        let mut parser = Parser::new(Box::new(Cursor::new(v)));
        let result: Vec<TPIUPacket> = parser.collect();
        assert_eq!(result, r);
    }

    #[test]
    fn test_one_stream() {
        test_single(vec![0x03, 0x17, 0x14, 0x02, 0x00, 0x08, 0x01, 0x00,
                         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    vec![TPIUPacket::Data(TraceSourceID(1), vec![0x17, 0x14, 0x02, 0x00, 0x08]),
                         TPIUPacket::Null(vec![0x00;8])]);
    }

    #[test]
    fn test_multiple_streams() {
        test_single(vec![0x03, 0x0E, 0x2C, 0x10, 0x05, 0x00, 0xFB, 0x00,
                         0x05, 0x00, 0x00, 0x00, 0x00, 0x80, 0x08, 0x00],
                    vec![TPIUPacket::Data(TraceSourceID(1), vec![0x0E, 0x2C, 0x10]),
                         TPIUPacket::Data(TraceSourceID(2), vec![0x00]),
                         TPIUPacket::Trigger(vec![0x00]),
                         TPIUPacket::Data(TraceSourceID(2), vec![0x00, 0x00, 0x00, 0x00, 0x80, 0x08])]);
    }
}
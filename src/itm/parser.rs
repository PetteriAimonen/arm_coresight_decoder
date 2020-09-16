//! Parses ITM packets from binary to Rust enums.

extern crate byteorder;
use self::byteorder::ReadBytesExt;
use self::byteorder::LittleEndian as LE;

use std::io::{Read, Error, ErrorKind};
use super::types::*;

use ::utils::bittuple::{to_bits,to_u32};

/// Reads variable length value encoded in the "protocol" encoding format,
/// where the top bit marks continuation. Returns value and length in bits.
fn read_protocol_value(input: &mut Read) -> Result<(u32,u8), Error> {
    let mut bitcount = 0;
    let mut result: u32 = 0;
    while bitcount < 28 {
        let byte = input.read_u8()?;
        result = (result << 7) | ((byte as u32) & 0x7F);
        bitcount += 7;
        if byte & 0x80 == 0 {
            return Ok((result, bitcount));
        }
    }
    return Err(Error::new(ErrorKind::InvalidData, "Too long protocol value"))
}

fn read_source_value(input: &mut Read, header: u8) -> Result<DataValue, Error> {
    match header & 3 {
        1 => Ok(DataValue::U8(input.read_u8()?)),
        2 => Ok(DataValue::U16(input.read_u16::<LE>()?)),
        3 => Ok(DataValue::U32(input.read_u32::<LE>()?)),
        _ => Err(Error::new(ErrorKind::InvalidData, "Zero length source value")),
    }
}

/// Parse "Synchronization packet", section ARMv7-M D.2.1
fn parse_synchronization_packet(input: &mut Read) -> Result<ITMPacket, Error> {
    loop {
        match input.read_u8()? {
            0x00 => continue,
            0x80 => return Ok(ITMPacket::Synchronization),
            other => return Ok(ITMPacket::Reserved(other))
        }
    }
}

/// Parse "Protocol packet", section ARMv7-M D.2.2
fn parse_protocol_packet(input: &mut Read, header: u8) -> Result<ITMPacket, Error> {
    let (payload, bitcount) = if header & 0x80 != 0 {
        read_protocol_value(input)?
    } else { (0, 0) };

    Ok(match to_bits(header) {
        (0,1,1,1,0,0,0,0) => ITMPacket::Overflow,
        (1,1,a,b,0,0,0,0) => ITMPacket::LocalTimestamp(
                                match (a,b) {
                                    (0,0) => TimestampSync::Synchronous,
                                    (0,1) => TimestampSync::TimestampDelayed,
                                    (1,0) => TimestampSync::DataDelayed,
                                    (1,1) => TimestampSync::BothDelayed,
                                    _ => panic!("Invalid TimestampSync")
                                },
                                LocalTimestampDelta(payload)
                            ),
        (0,a,b,c,0,0,0,0) => ITMPacket::LocalTimestamp(
                                TimestampSync::Synchronous,
                                LocalTimestampDelta(to_u32(&[a,b,c]))
                            ),
        (1,0,0,1,0,1,0,0) => ITMPacket::GlobalTimestamp(GlobalTimestampValue{
                                timestamp: (payload as u64) & 0x03FFFFFF,
                                known_mask: 0x03FFFFFF,
                                wrap: (payload & (1 << 27)) != 0,
                                clock_change: (payload & (1 << 26)) != 0
                            }),
        (1,0,1,1,0,1,0,0) => ITMPacket::GlobalTimestamp(GlobalTimestampValue{
                                timestamp: payload as u64,
                                known_mask: (0x3FFFFF << 26),
                                wrap: false,
                                clock_change: false
                            }),
        (0,a,b,c,1,0,0,0) => ITMPacket::SoftwarePageNumber(InstrumentationPort(to_u32(&[a,b,c]) << 5)),
        (_,a,b,c,1,s,0,0) => ITMPacket::Extension(ExtendedInformation {
                                data: (payload << 3) | to_u32(&[a,b,c]),
                                bitcount: bitcount + 3,
                                source: s
                            }),
        (_,_,_,_,_,_,_,_) => ITMPacket::Reserved(header)
    })
}

/// Parse "Source packet", section ARMv7-M D.2.7
fn parse_source_packet(input: &mut Read, header: u8) -> Result<ITMPacket, Error>
{
    let datavalue = read_source_value(input, header)?;
    let payload = datavalue.to_u32();

    Ok(match to_bits(header) {
        (a,b,c,d,e,0,_,_) => ITMPacket::Software(InstrumentationPort(to_u32(&[a,b,c,d,e])),
                                                 datavalue),
        (0,0,0,0,0,1,0,1) => ITMPacket::EventCounter(EventCounterFlags{
                                cpicnt: payload & 0x01 != 0,
                                exccnt: payload & 0x02 != 0,
                                sleepcnt: payload & 0x04 != 0,
                                lsucnt: payload & 0x08 != 0,
                                foldcnt: payload & 0x10 != 0,
                                postcnt: payload & 0x20 != 0
                            }),
        (0,0,0,0,1,1,1,0) => ITMPacket::Exception(
                                match payload >> 12 {
                                    1 => Ok(ExceptionEvent::Enter),
                                    2 => Ok(ExceptionEvent::Exit),
                                    3 => Ok(ExceptionEvent::Resume),
                                    _ => Err(Error::new(ErrorKind::InvalidData, "Unknown ExceptionEvent"))
                                }?,
                                ExceptionNumber(payload & 0x1FF)
                            ),
        (0,0,0,1,0,1,1,1) => ITMPacket::ProgramCounter(Address(payload)),
        (0,1,a,b,0,1,1,0) => ITMPacket::DataTracePC(ComparatorIndex(to_u32(&[a,b])), Address(payload)),
        (0,1,a,b,1,1,1,0) => ITMPacket::DataTraceOffset(ComparatorIndex(to_u32(&[a,b])), Address(payload)),
        (1,0,a,b,0,1,_,_) => ITMPacket::DataTraceReadData(ComparatorIndex(to_u32(&[a,b])), datavalue),
        (1,0,a,b,1,1,_,_) => ITMPacket::DataTraceWriteData(ComparatorIndex(to_u32(&[a,b])), datavalue),
        (_,_,_,_,_,_,_,_) => ITMPacket::Reserved(header)
    })
}

pub fn parse_one(input: &mut Read) -> Result<ITMPacket, Error> {
    let header = input.read_u8()?;

    let result = match to_bits(header) {
        (0,0,0,0,0,0,0,0) => parse_synchronization_packet(input),
        (_,_,_,_,_,_,0,0) => parse_protocol_packet(input, header),
        (_,_,_,_,_,_,_,_) => parse_source_packet(input, header)
    };

    match result {
        Err(ref e) if e.kind() == ErrorKind::InvalidData => {
            Ok(ITMPacket::Invalid(e.to_string()))
        }
        result => {
            result
        }
    }
}

pub struct Parser<T> {
    input: T,
    error: Option<Error>,
}

impl<T:Read> Parser<T> {
    pub fn new(input: T) -> Parser<T> {
        Parser{ input, error: None }
    }
}

impl<T:Read> Iterator for Parser<T> {
    type Item = ITMPacket;
    fn next(&mut self) -> Option<ITMPacket> {
        match parse_one(&mut self.input) {
            Ok(result) => Some(result),
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => None,
            Err(e) => {self.error = Some(e); None}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    fn test_single(v: Vec<u8>, r: ITMPacket) {
        assert_eq!(parse_one(&mut Cursor::new(v)).unwrap(), r);
    }

    #[test]
    fn test_basic() {
        test_single(vec![0x17, 0x16, 0x02, 0x00, 0x08],
                    ITMPacket::ProgramCounter(Address(0x08000216)));
        test_single(vec![0x4e, 0x10, 0x10],
                    ITMPacket::DataTraceOffset(ComparatorIndex(0), Address(0x1010)));
    }
}


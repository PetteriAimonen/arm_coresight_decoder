//! Wrapper for a std::io::Read that counts the number
//! of bytes read so far. Useful for correlating packet
//! position to input byte timestamps.

use std::io::{Read, Result};

pub struct ReadPos {
    inner: Box<Read>,
    position: usize
}

impl ReadPos {
    pub fn new(inner: Box<Read>) -> ReadPos {
        ReadPos{inner, position:0}
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

impl Read for ReadPos {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>
    {
        let result = self.inner.read(buf);
        
        if let Ok(size) = result {
            self.position += size;
        }

        result
    }
}

//! Allows iterating through parsed packets.

use std::io::{Read, Error};

pub struct ParserIterator<T,R> {
    input: T,
    parse: fn(&mut Read) -> Result<R, Error>,
    error: Option<Error>,
}

impl<T,R> ParserIterator<T,R> {
    pub fn new(input: T,
               parse: fn(&mut Read) -> Result<R, Error>)
               -> ParserIterator<T,R>
    {
        ParserIterator{input, parse, error: None}
    }
}

impl<T:Read,R> Iterator for ParserIterator<T,R> {
    type Item = R;
    fn next(&mut self) -> Option<R> {
        match (self.parse)(&mut self.input) {
            Ok(result) => Some(result),
            Err(error) => {self.error = Some(error); None}
        }
    }
}

use crate::errors::ParseError;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    name: String,
    value: String,
}

pub struct TagsReader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> TagsReader<'a> {
    pub fn deserialize(buffer: &'a [u8]) -> Result<Vec<Tag>> {
        TagsReader { buffer, pos: 0 }.read_tags()
    }

    fn skip_long(&mut self) {
        while self.buffer.get(self.pos).map_or(false, |&b| b & 0x80 != 0) {
            self.pos += 1;
        }
        // Move past the last byte that was part of the long
        if self.pos < self.buffer.len() {
            self.pos += 1;
        }
    }

    fn read_tags(&mut self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();

        while let Some(ref mut length) = self.read_long() {
            if length.is_negative() {
                *length = length.abs();
                self.skip_long()
            }

            for _ in 0..*length {
                let name = self.read_string()?;
                let value = self.read_string()?;
                tags.push(Tag { name, value });
            }
        }

        Ok(tags)
    }

    fn read_long(&mut self) -> Option<i64> {
        let mut number = 0i64;
        let mut shift = 0usize;

        while self.pos < self.buffer.len() {
            let byte = self.buffer[self.pos];
            self.pos += 1;

            number |= ((byte & 0x7f) as i64) << shift;
            shift += 7;

            if byte & 0x80 == 0 || shift >= 28 {
                return Some((number >> 1) ^ -(number & 1));
            }
        }

        None
    }

    fn read_string(&mut self) -> Result<String> {
        let length = self.read_long().ok_or_else(|| ParseError::ExpectedLong)?;

        if length < 0 || (self.pos + length as usize) > self.buffer.len() {
            return Err(ParseError::InvalidLengthString);
        }

        let start_pos = self.pos;
        self.pos += length as usize;

        let str = String::from_utf8(self.buffer[start_pos..self.pos].to_vec())?;
        Ok(str)
    }
}

use crate::errors::ParseError;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParseError;

    #[test]
    fn test_deserialize_tags() {
        let buffer = b"\x05\x00\x04\x00\x06\x00\x07\x00\x08\x00";
        let expected_tags = vec![
            Tag {
                name: "name1".to_string(),
                value: "value1".to_string(),
            },
            Tag {
                name: "name2".to_string(),
                value: "value2".to_string(),
            },
            Tag {
                name: "name3".to_string(),
                value: "value3".to_string(),
            },
            Tag {
                name: "name4".to_string(),
                value: "value4".to_string(),
            },
            Tag {
                name: "name5".to_string(),
                value: "value5".to_string(),
            },
        ];

        let result = TagsReader::deserialize(buffer);
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags, expected_tags);
    }

    #[test]
    fn test_deserialize_tags_invalid_length() {
        let buffer = b"\x02\x00\x03\x00\x04\x00\x05\x00\x06\x00\x07\x00";
        let result = TagsReader::deserialize(buffer);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error, ParseError::InvalidLengthString);
    }

    #[test]
    fn test_deserialize_tags_expected_long() {
        let buffer = b"\x02\x00\x03\x00\x04\x00\x05\x00\x06";
        let result = TagsReader::deserialize(buffer);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error, ParseError::ExpectedLong);
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

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

    fn skip_long(&mut self) -> Result<()> {
        while self.buffer.get(self.pos).map_or(false, |&b| b & 0x80 != 0) {
            self.pos += 1;
        }
        // Move past the last byte that was part of the long
        if self.pos < self.buffer.len() {
            self.pos += 1;
        }
        Ok(())
    }

    fn read_tags(&mut self) -> Result<Vec<Tag>> {
        let mut val = Vec::new();

        while let Some(ref mut n) = self.read_long()? {
            if n.is_negative() {
                *n = n.abs();
                self.skip_long()?
            }

            for _ in 0..*n {
                let name = self.read_string()?;
                let value = self.read_string()?;
                val.push(Tag { name, value });
            }
        }

        Ok(val)
    }

    fn read_long(&mut self) -> Result<Option<i64>> {
        let mut n = 0i64;
        let mut k = 0usize;

        loop {
            if self.pos >= self.buffer.len() {
                return Ok(None);
            }

            let b = self.buffer[self.pos];
            self.pos += 1;

            n |= ((b & 0x7f) as i64) << k;
            k += 7;

            if b & 0x80 == 0 || k >= 28 {
                break;
            }
        }

        Ok(Some((n >> 1) ^ -(n & 1)))
    }

    fn read_string(&mut self) -> Result<String> {
        let len = match self.read_long()? {
            Some(len) => len,
            None => return Err(anyhow::anyhow!("Failed to read length")),
        };

        if len < 0 || (self.pos + len as usize) > self.buffer.len() {
            return Err(anyhow::anyhow!("Invalid string length"));
        }

        let start_pos = self.pos;
        self.pos += len as usize;

        match String::from_utf8(self.buffer[start_pos..self.pos].to_vec()) {
            Ok(s) => Ok(s),
            Err(_) => Err(anyhow::anyhow!("Invalid UTF-8 sequence")),
        }
    }
}

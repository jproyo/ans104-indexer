use std::string::FromUtf8Error;

use base64::DecodeError;
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Error)]
pub enum ParseError {
    #[error("Expected Long value but cannot be extracted")]
    ExpectedLong,
    #[error("Invalid length when trying to parse String for Tags")]
    InvalidLengthString,
    #[error("Invalid String UTF-8 {0}")]
    InvalidString(#[from] FromUtf8Error),
    #[error("Invalid Signature Type {0}")]
    InvalidSignatureType(u16),
    #[error("Invalid Presence Byte {0}")]
    InvalidPresenceByte(u8),
    #[error("Invalid Tags Length. Expected {0} - Parsed {0}")]
    InvalidTagsLength(u64, usize),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Cannot Serialize Item to json - {0}")]
    CannotSerializeItem(String),
    #[error("Error with IO {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Error parsing data - {0}")]
    ParsingData(#[from] ParseError),
    #[error("Decoding from Base64 {0}")]
    DecodeBase64Error(#[from] DecodeError),
    #[error("Error communication with Server {0}")]
    CommunicationError(#[from] reqwest::Error),
    #[error("Parse Url Error - {0}")]
    ParseUrl(String),
}

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Downloader Error - {0}")]
    Downloader(#[from] ClientError),
    #[error("Storage Error - {0}")]
    Storage(#[from] StorageError),
    #[error("Parser Error - {0}")]
    Parser(#[from] ParseError),
}

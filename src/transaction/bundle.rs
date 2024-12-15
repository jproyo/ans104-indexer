use super::tags::Tag;
use crate::errors::ParseError;
use crate::transaction::tags::TagsReader;
use base64::{encode_config, URL_SAFE_NO_PAD};
use bytes::{Buf, BytesMut};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::task::Poll;
use tokio_stream::Stream;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum SignatureType {
    Arweave,
    ED25519,
    Ethereum,
    Solana,
    Injectedaptos,
    Multiaptos,
    Typedethereum,
    Starknet,
}

impl TryFrom<u16> for SignatureType {
    type Error = ParseError;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            1 => Ok(SignatureType::Arweave),
            2 => Ok(SignatureType::ED25519),
            3 => Ok(SignatureType::Ethereum),
            4 => Ok(SignatureType::Solana),
            5 => Ok(SignatureType::Injectedaptos),
            6 => Ok(SignatureType::Multiaptos),
            7 => Ok(SignatureType::Typedethereum),
            8 => Ok(SignatureType::Starknet),
            _ => Err(ParseError::InvalidSignatureType(value)),
        }
    }
}

lazy_static! {
    static ref SIG_CONFIG: HashMap<SignatureType, (usize, usize)> = HashMap::from([
        (SignatureType::Arweave, (512, 512)),
        (SignatureType::ED25519, (64, 32)),
        (SignatureType::Ethereum, (65, 65)),
        (SignatureType::Solana, (64, 32)),
        (SignatureType::Injectedaptos, (64, 32)),
        (SignatureType::Multiaptos, (2052, 1025)),
        (SignatureType::Typedethereum, (65, 42)),
        (SignatureType::Starknet, (128, 33))
    ]);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BundleItem {
    id: String,
    signature: String,
    owner: String,
    target: Option<String>,
    anchor: Option<String>,
    tags: Vec<Tag>,
    data: String,
}

impl BundleItem {
    fn parse_item(mut data: BytesMut, id: String) -> Result<Self> {
        // Read signature type (2 bytes)
        let signature_type: SignatureType = data.get_u16_le().try_into()?;

        let (sig_size, owner_size) = SIG_CONFIG[&signature_type];
        let signature = encode_config(data.split_to(sig_size), URL_SAFE_NO_PAD);
        let owner = encode_config(data.split_to(owner_size), URL_SAFE_NO_PAD);

        let target = Self::read_optional_string(&mut data)?;
        let anchor = Self::read_optional_string(&mut data)?;

        let tags_length = data.get_u64_le();
        let num_bytes_for_tags = data.get_u64_le();
        let tags = if tags_length > 0 && num_bytes_for_tags > 0 {
            let tags_bytes = data.split_to(num_bytes_for_tags as usize);
            let tags = TagsReader::deserialize(&tags_bytes)?;
            if tags.len() != tags_length as usize {
                return Err(ParseError::InvalidTagsLength(tags_length, tags.len()));
            }
            tags
        } else {
            vec![]
        };

        let data = encode_config(data, URL_SAFE_NO_PAD);

        Ok(BundleItem {
            id,
            signature,
            owner,
            target,
            anchor,
            tags,
            data,
        })
    }

    pub fn stream(mut data: BytesMut) -> Result<BundleStream> {
        let num_entries = data.get_u32_le();
        data.advance(28);
        let mut entries = vec![];
        for _ in 0..num_entries {
            let size_entry = data.get_u32_le();
            data.advance(28);
            let id = encode_config(data.split_to(32), URL_SAFE_NO_PAD);
            entries.push((size_entry, id));
        }

        Ok(BundleStream {
            bytes: data,
            entries,
            current_entry: 0,
        })
    }

    fn read_optional_string(data: &mut BytesMut) -> Result<Option<String>> {
        match data.get_u8() {
            0 => Ok(None),
            1 => Ok(Some(encode_config(data.split_to(32), URL_SAFE_NO_PAD))),
            _ => Err(ParseError::InvalidPresenceByte(data.get_u8())),
        }
    }
}

pub struct BundleStream {
    bytes: BytesMut,
    entries: Vec<(u32, String)>,
    current_entry: usize,
}

impl Stream for BundleStream {
    type Item = Result<BundleItem>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        if this.current_entry < this.entries.len() {
            let (size, ref id) = this.entries[this.current_entry];
            let item_data = this.bytes.split_to(size as usize);
            let bundle = BundleItem::parse_item(item_data, id.to_string());
            this.current_entry += 1;
            return Poll::Ready(Some(bundle));
        }
        std::task::Poll::Ready(None)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use bytes::BytesMut;
    use tokio::io::AsyncBufReadExt;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_stream() {
        let data = tokio::fs::read("tests/fixtures/NbhWDsl290UWZ7GfMIM2xdrgCORceNgzmHf4l2uDypc")
            .await
            .unwrap();
        let result = BundleItem::stream(BytesMut::from(data.as_slice()));

        let mut read_lines = tokio::io::BufReader::new(
            tokio::fs::File::open(
                "tests/fixtures/NbhWDsl290UWZ7GfMIM2xdrgCORceNgzmHf4l2uDypc.expected",
            )
            .await
            .unwrap(),
        )
        .lines();

        let mut expected: Vec<BundleItem> = vec![];
        while let Some(line) = read_lines.next_line().await.unwrap() {
            expected.push(serde_json::from_str(line.as_str()).unwrap());
        }

        assert!(result.is_ok());
        let mut stream = result.unwrap();
        let mut parsed = vec![];
        while let Some(Ok(i)) = stream.next().await {
            parsed.push(i);
        }
        assert_eq!(parsed, expected);
    }
}

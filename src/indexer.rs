use anyhow::{Context, Result};
use base64::{decode_config, encode, encode_config, URL_SAFE_NO_PAD};
use bytes::{Buf, BytesMut};
use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::tags::{Tag, TagsReader};

const ARWEAVE_GATEWAY: &str = "https://arweave.net";

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
    type Error = anyhow::Error;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(SignatureType::Arweave),
            2 => Ok(SignatureType::ED25519),
            3 => Ok(SignatureType::Ethereum),
            4 => Ok(SignatureType::Solana),
            5 => Ok(SignatureType::Injectedaptos),
            6 => Ok(SignatureType::Multiaptos),
            7 => Ok(SignatureType::Typedethereum),
            8 => Ok(SignatureType::Starknet),
            _ => Err(anyhow::anyhow!("Invalid value for SignatureType")),
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

#[derive(Debug, Serialize, Deserialize)]
struct BundleItem {
    id: String,
    signature: String,
    owner: String,
    target: String,
    anchor: String,
    tags: Vec<Tag>,
    data: String,
}

pub async fn index_bundle(transaction_id: &str, output_file: &str) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/tx/{}/data", ARWEAVE_GATEWAY, transaction_id);

    let response = client.get(&url).send().await?;
    let output = response.bytes().await?;
    let output = decode_config(output, URL_SAFE_NO_PAD)?;
    let mut data: BytesMut = BytesMut::from(output.as_slice());

    let bundle_items = parse_bundle(&mut data)?;

    let json_output = serde_json::to_string_pretty(&bundle_items)?;
    let mut file = File::create(output_file).context("Failed to create output file")?;
    file.write_all(json_output.as_bytes())
        .context("Failed to write to output file")?;

    println!("Bundle indexed and saved to {}", output_file);
    Ok(())
}

fn parse_bundle(data: &mut BytesMut) -> Result<Vec<BundleItem>> {
    let mut items = Vec::new();

    let num_entries = data.get_u32_le();
    data.advance(28);
    dbg!(num_entries);
    let mut entries = vec![];
    dbg!(&data.len());
    for _ in 0..num_entries {
        let size_entry = data.get_u32_le();
        data.advance(28);
        let id = encode(data.split_to(32));
        entries.push((size_entry, id));
    }
    dbg!(&data.len());

    dbg!(&entries);
    for (size, id) in entries {
        let item = parse_bundle_item(&mut data.split_to(size as usize), id)?;
        items.push(item);
    }

    Ok(items)
}

fn parse_bundle_item(data: &mut BytesMut, id: String) -> Result<BundleItem> {
    // Read signature type (2 bytes)
    let signature_type: SignatureType = data.get_u16_le().try_into()?;

    dbg!(signature_type);

    let (sig_size, owner_size) = SIG_CONFIG[&signature_type];
    let sig_bytes = data.split_to(sig_size);
    let signature = encode_config(sig_bytes, URL_SAFE_NO_PAD);

    dbg!(&signature);

    let owner_bytes = data.split_to(owner_size);
    let owner = encode_config(owner_bytes, URL_SAFE_NO_PAD);
    dbg!(&owner);

    let presence_target = data.get_u8();
    let target = match presence_target {
        0 => String::new(),
        1 => encode_config(data.split_to(32), URL_SAFE_NO_PAD),
        _ => {
            return Err(anyhow::anyhow!(
                "Error presence target byte {presence_target}"
            ))
        }
    };
    dbg!(&target);

    let presence_anchor = data.get_u8();
    let anchor = match presence_anchor {
        0 => String::new(),
        1 => encode_config(data.split_to(32), URL_SAFE_NO_PAD),
        _ => {
            return Err(anyhow::anyhow!(
                "Error presence anchor byte {presence_anchor}"
            ))
        }
    };
    dbg!(&anchor);

    let tags_length = data.get_u64_le();
    let num_bytes_for_tags = data.get_u64_le();
    let mut tags = vec![];
    if tags_length > 0 && num_bytes_for_tags > 0 {
        let mut tags_bytes = data.split_to(num_bytes_for_tags as usize);
        tags = parse_tags(&mut tags_bytes)?;
        if tags.len() != tags_length as usize {
            return Err(anyhow::anyhow!("Different number of tags with respect to length [Encoded Length={tags_length}, Deserialized Tags={}", tags.len()));
        }
    }

    let item_data = encode_config(data, URL_SAFE_NO_PAD);

    let item = BundleItem {
        id,
        signature,
        owner,
        target,
        anchor,
        tags,
        data: item_data,
    };
    dbg!(&item);

    Ok(item)
}

fn parse_tags(data: &mut BytesMut) -> Result<Vec<Tag>> {
    TagsReader::deserialize(data)
}

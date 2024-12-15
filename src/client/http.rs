use crate::errors::ClientError;
use crate::transaction::bundle::BundleItem;
use base64::{decode_config, URL_SAFE_NO_PAD};
use bytes::BytesMut;
use reqwest::{Client, Url};

use super::Downloader;

pub struct HttpDownloader {
    url: Url,
}

impl HttpDownloader {
    pub fn new(url: String) -> Result<Self, ClientError> {
        let url = Url::parse(&url).map_err(|e| ClientError::ParseUrl(e.to_string()))?;
        Ok(Self { url })
    }
}

#[async_trait::async_trait]
impl Downloader for HttpDownloader {
    async fn download(&self, transaction_id: String) -> Result<Vec<BundleItem>, ClientError> {
        let client = Client::new();
        let url = self
            .url
            .join(format!("tx/{}/data", transaction_id).as_str())
            .map_err(|e| ClientError::ParseUrl(e.to_string()))?;

        let response = client.get(url).send().await?;
        let output = response.bytes().await?;
        let output = decode_config(output, URL_SAFE_NO_PAD)?;
        let mut data = BytesMut::from(output.as_slice());

        let bundles = BundleItem::deserialize(&mut data)?;
        Ok(bundles)
    }
}

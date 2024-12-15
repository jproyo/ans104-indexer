use crate::errors::ClientError;
use crate::transaction::bundle::BundleStream;

pub mod http;

#[async_trait::async_trait]
pub trait Downloader {
    async fn download(&self, transaction_id: String) -> Result<BundleStream, ClientError>;
}

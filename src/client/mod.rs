use crate::errors::ClientError;
use crate::transaction::bundle::BundleItem;

pub mod http;

#[async_trait::async_trait]
pub trait Downloader {
    async fn download(&self, transaction_id: String) -> Result<Vec<BundleItem>, ClientError>;
}

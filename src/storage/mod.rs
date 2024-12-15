pub mod fs;

use crate::errors::StorageError;
use crate::transaction::bundle::BundleItem;

#[async_trait::async_trait]
pub trait Storage {
    async fn store(&mut self, bundle_item: BundleItem) -> Result<(), StorageError>;

    async fn commit(self) -> Result<(), StorageError>;

    async fn rollback(self);
}

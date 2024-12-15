use std::env::temp_dir;
use std::path::PathBuf;

use super::Storage;
use crate::errors::StorageError;
use crate::transaction::bundle::BundleItem;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct LocalStorageFS<W: AsyncWrite> {
    storage: PathBuf,
    transaction_id: String,
    current_file: PathBuf,
    fs: W,
}

impl LocalStorageFS<tokio::fs::File> {
    pub async fn new(transaction_id: String, storage_folder: String) -> Result<Self, StorageError> {
        let current_file = temp_dir().join(transaction_id.clone());
        let storage = storage_folder.clone().into();
        tokio::fs::create_dir_all(storage_folder).await?;
        let fs = tokio::fs::File::options()
            .append(true)
            .create(true)
            .open(current_file.clone())
            .await?;
        Ok(Self {
            current_file,
            transaction_id,
            storage,
            fs,
        })
    }
}

#[async_trait::async_trait]
impl<W: AsyncWrite + Send + Unpin> Storage for LocalStorageFS<W> {
    async fn store(&mut self, bundle_item: BundleItem) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(&bundle_item)
            .map_err(|e| StorageError::CannotSerializeItem(e.to_string()))?;
        self.fs.write_all(&bytes).await?;
        self.fs.flush().await?;
        Ok(())
    }

    async fn commit(self) -> Result<(), StorageError> {
        tokio::fs::rename(self.current_file, self.storage.join(self.transaction_id)).await?;
        Ok(())
    }

    async fn rollback(self) {
        tokio::fs::remove_file(self.current_file)
            .await
            .unwrap_or_default()
    }
}

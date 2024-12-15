use crate::client::http::HttpDownloader;
use crate::client::Downloader;
use crate::errors::IndexerError;
use crate::storage::fs::LocalStorageFS;
use crate::storage::Storage;

pub struct Indexer<D, S> {
    downloader: D,
    storage: S,
}

impl Indexer<HttpDownloader, LocalStorageFS<tokio::fs::File>> {
    pub async fn default(
        url_download: &str,
        transaction_id: &str,
        storage_folder: &str,
    ) -> Result<Self, IndexerError> {
        let storage =
            LocalStorageFS::new(transaction_id.to_string(), storage_folder.to_string()).await?;
        let downloader = HttpDownloader::new(url_download.to_string())?;
        Ok(Self {
            downloader,
            storage,
        })
    }
}

impl<D, S> Indexer<D, S>
where
    D: Downloader,
    S: Storage,
{
    pub async fn index(mut self, transaction_id: String) -> Result<(), IndexerError> {
        let items = self.downloader.download(transaction_id).await?;
        let mut r: Option<IndexerError> = None;
        for i in items {
            match self.storage.store(i).await {
                Err(e) => {
                    r = Some(IndexerError::Storage(e));
                    break;
                }
                Ok(_) => continue,
            }
        }
        let final_ref = self.storage;
        match r {
            None => final_ref.commit().await?,
            Some(e) => {
                final_ref.rollback().await;
                return Err(e);
            }
        }
        Ok(())
    }
}

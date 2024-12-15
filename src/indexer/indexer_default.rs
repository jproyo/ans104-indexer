use crate::client::http::HttpDownloader;
use crate::client::Downloader;
use crate::errors::IndexerError;
use crate::storage::fs::LocalStorageFS;
use crate::storage::Storage;
use std::path::PathBuf;
use tokio_stream::StreamExt;

pub struct Indexer<D> {
    downloader: D,
    storage_folder: PathBuf,
}

impl Indexer<HttpDownloader> {
    pub async fn new(url_download: &str, storage_folder: &str) -> Result<Self, IndexerError> {
        let downloader = HttpDownloader::new(url_download.to_string())?;
        Ok(Self {
            downloader,
            storage_folder: storage_folder.into(),
        })
    }
}

impl<D> Indexer<D>
where
    D: Downloader,
{
    pub async fn index(&self, transaction_id: String) -> Result<(), IndexerError> {
        let mut storage =
            LocalStorageFS::new(transaction_id.clone(), self.storage_folder.clone()).await?;
        let mut items = self.downloader.download(transaction_id).await?;
        while let Some(item) = items.next().await {
            match item {
                Ok(value) => match storage.store(value).await {
                    Err(e) => {
                        storage.rollback().await;
                        return Err(IndexerError::Storage(e));
                    }
                    Ok(_) => continue,
                },
                Err(e) => {
                    storage.rollback().await;
                    return Err(IndexerError::Parser(e));
                }
            }
        }
        storage.commit().await?;
        Ok(())
    }
}

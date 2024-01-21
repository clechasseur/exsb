use std::sync::Arc;

use tokio::sync::{Semaphore, SemaphorePermit};

#[derive(Debug, Clone)]
pub struct DownloadLimiter(Arc<Semaphore>);

#[derive(Debug)]
pub struct DownloadPermit<'a>(SemaphorePermit<'a>);

impl DownloadLimiter {
    pub fn new(max_downloads: usize) -> Self {
        Self(Arc::new(Semaphore::new(max_downloads)))
    }

    pub async fn get_permit(&self) -> DownloadPermit<'_> {
        DownloadPermit(self.0.acquire().await.unwrap())
    }
}

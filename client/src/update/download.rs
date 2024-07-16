use bytes::{BufMut, BytesMut};
use std::{path::PathBuf, time::Duration};

use reqwest::{RequestBuilder, StatusCode};
use tokio::{fs::File, io::AsyncWriteExt, time::Instant};

#[derive(Debug, Clone)]
pub struct ProgressData {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub bytes_per_sec: u64,
    pub content: DownloadContent,
}

#[derive(Debug, Clone)]
pub enum DownloadContent {
    CentralDirectory,
    FullZip,
    SingleFile(String),
}

impl ProgressData {
    pub(crate) fn new(total_bytes: u64, content: DownloadContent) -> Self {
        Self {
            total_bytes,
            content,
            bytes_per_sec: 0,
            downloaded_bytes: 0,
        }
    }

    pub(crate) fn percent_complete(&self) -> u64 {
        (self.downloaded_bytes as f32 * 100.0 / self.total_bytes as f32) as u64
    }

    pub(crate) fn remaining(&self) -> Duration {
        Duration::from_secs_f32(
            (self.total_bytes - self.downloaded_bytes) as f32
                / self.bytes_per_sec.max(1) as f32,
        )
    }
}

#[derive(Debug, Clone)]
pub(super) struct InternalProgressData {
    pub(super) progress: ProgressData,
    last_rate_check: Instant,
    downloaded_since_last_check: usize,
}

impl InternalProgressData {
    pub(crate) fn new(progress: ProgressData) -> Self {
        Self {
            progress,
            last_rate_check: Instant::now(),
            downloaded_since_last_check: 0,
        }
    }

    pub(crate) fn add_chunk(&mut self, data: u64) {
        self.progress.downloaded_bytes += data;

        let current_time = Instant::now();
        let since_last_check = current_time - self.last_rate_check;
        let since_last_check_f32 = since_last_check.as_secs_f32();
        if since_last_check >= Duration::from_millis(500)
            || (since_last_check_f32 > 0.0 && self.progress.downloaded_bytes == data)
        {
            let bytes_per_sec =
                (self.downloaded_since_last_check as f32 / since_last_check_f32) as u64;
            self.downloaded_since_last_check = 0;
            self.last_rate_check = current_time;
        }
    }
}

#[derive(Debug)]
pub(super) enum Storage {
    FileInfo(PathBuf),
    File(File),
    Memory(BytesMut),
}

#[derive(Debug, thiserror::Error)]
pub(super) enum DownloadError {
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Non-OK Status: {0}")]
    InvalidStatus(StatusCode),
    #[error("StorageWrite Error: {0}")]
    WriteError(#[from] std::io::Error),
}

#[derive(Debug)]
pub(super) enum Download {
    Start(RequestBuilder, Storage, DownloadContent),
    Progress(reqwest::Response, Storage, InternalProgressData),
    Finished(Storage),
}

impl Download {
    /// downloads a single "thing" partially, so it can be showed in UI
    pub(super) async fn progress(self) -> Result<Self, DownloadError> {
        match self {
            Download::Start(request, storage, content) => {
                let response = request.send().await?;

                if !response.status().is_success() {
                    return Err(DownloadError::InvalidStatus(response.status()));
                }

                let storage = match storage {
                    Storage::FileInfo(path) => Storage::File(File::create(path).await?),
                    storage => storage,
                };

                let total = response.content_length().unwrap_or_default();
                let progress =
                    InternalProgressData::new(ProgressData::new(total, content));
                Ok(Self::Progress(response, storage, progress))
            },
            Download::Progress(mut response, mut storage, mut progress) => {
                match response.chunk().await? {
                    Some(chunk) => {
                        progress.add_chunk(chunk.len() as u64);

                        match &mut storage {
                            Storage::FileInfo(_) => unreachable!(),
                            Storage::File(ref mut file) => file.write_all(&chunk).await?,
                            Storage::Memory(ref mut mem) => mem.put(chunk),
                        }
                        Ok(Self::Progress(response, storage, progress))
                    },
                    None => {
                        if let Storage::File(file) = &storage {
                            file.sync_all().await?;
                        }
                        Ok(Self::Finished(storage))
                    },
                }
            },
            Download::Finished(storage) => Ok(Download::Finished(storage)),
        }
    }
}

impl DownloadContent {
    pub fn show(&self) -> &str {
        match self {
            DownloadContent::SingleFile(x) => &x,
            _ => "",
        }
    }
}

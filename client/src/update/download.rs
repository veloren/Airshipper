use bytes::{BufMut, BytesMut};
use std::{path::PathBuf, time::Duration};

use reqwest::{RequestBuilder, StatusCode};
use tokio::{fs::File, io::AsyncWriteExt, time::Instant};

use crate::error::ClientError;

#[derive(Debug, Clone)]
pub struct StepProgress {
    pub total_bytes: u64,
    pub processed_bytes: u64,
    // internal buffer to be applied to overall progress
    buf_processed_bytes: u64,
    pub content: UpdateContent,
}

#[derive(Debug, Clone)]
pub struct OverallProgress {
    last_rate_check: Instant,
    downloaded_since_last_check: u64,
    bytes_per_sec: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct ProgressData {
    cur_step: StepProgress,
    overall: OverallProgress,
}

#[derive(Debug, Clone)]
pub enum UpdateContent {
    CentralDirectory,
    DownloadFullZip,
    HashLocalFile(String),
    DownloadFile(String),
    Decompress(String),
}

impl StepProgress {
    pub(crate) fn new(total_bytes: u64, content: UpdateContent) -> Self {
        Self {
            total_bytes,
            processed_bytes: 0,
            buf_processed_bytes: 0,
            content,
        }
    }

    pub(crate) fn add_chunk(&mut self, data: u64) {
        self.processed_bytes += data;
        self.buf_processed_bytes += data;
    }

    pub(crate) fn percent_complete(&self) -> u64 {
        (self.processed_bytes as f32 * 100.0 / self.total_bytes as f32) as u64
    }
}

impl Default for OverallProgress {
    fn default() -> Self {
        Self {
            last_rate_check: Instant::now(),
            downloaded_since_last_check: 0,
            bytes_per_sec: 0,
        }
    }
}

impl OverallProgress {
    fn add_from_step(&mut self, step: &mut StepProgress) -> u64 {
        let data = std::mem::take(&mut step.buf_processed_bytes);

        self.downloaded_since_last_check += data;

        let current_time = Instant::now();
        let since_last_check = current_time - self.last_rate_check;
        let since_last_check_f32 = since_last_check.as_secs_f32();
        if since_last_check >= Duration::from_millis(250)
            || (since_last_check_f32 > 0.0 && self.bytes_per_sec == 0)
        {
            let bytes_per_sec =
                (self.downloaded_since_last_check as f32 / since_last_check_f32) as u64;
            self.downloaded_since_last_check = 0;
            self.last_rate_check = current_time;
            self.bytes_per_sec = bytes_per_sec;
        }
        data
    }

    pub fn bytes_per_sec(&self) -> u64 {
        self.bytes_per_sec
    }
}

impl ProgressData {
    pub(crate) fn new(step: StepProgress, overall: OverallProgress) -> Self {
        Self {
            cur_step: step,
            overall,
        }
    }

    pub(crate) fn add_chunk(&mut self, data: u64) {
        self.cur_step.add_chunk(data);
        self.overall.add_from_step(&mut self.cur_step);
    }

    pub(crate) fn add_from_step(&mut self, step: &mut StepProgress) {
        // adds to local overall from other step
        let data = self.overall.add_from_step(step);
        self.cur_step.processed_bytes += data;
    }

    pub fn cur_step(&self) -> &StepProgress {
        &self.cur_step
    }

    pub fn overall(&self) -> &OverallProgress {
        &self.overall
    }

    pub(super) fn cur_step_mut(&mut self) -> &mut StepProgress {
        &mut self.cur_step
    }

    pub(crate) fn cur_step_remaining(&self) -> Duration {
        Duration::from_secs_f32(
            (self.cur_step.total_bytes - self.cur_step.processed_bytes) as f32
                / self.overall.bytes_per_sec.max(1) as f32,
        )
    }
}

#[derive(Debug)]
pub enum Storage {
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
pub(super) enum Download<T> {
    Start(RequestBuilder, Storage, UpdateContent, T),
    Progress(reqwest::Response, Storage, StepProgress, T),
    Finished(Storage, T),
}

impl<T> Download<T> {
    /// downloads a single "thing" partially, so it can be showed in UI
    pub(super) async fn progress(self) -> Result<Self, DownloadError> {
        match self {
            Download::Start(request, storage, content, c) => {
                let response = request.send().await?;

                if !response.status().is_success() {
                    return Err(DownloadError::InvalidStatus(response.status()));
                }

                let storage = match storage {
                    Storage::FileInfo(path) => Storage::File(File::create(path).await?),
                    storage => storage,
                };

                let total = response.content_length().unwrap_or_default();
                let progress = StepProgress::new(total, content);
                Ok(Self::Progress(response, storage, progress, c))
            },
            Download::Progress(mut response, mut storage, mut progress, c) => {
                match response.chunk().await? {
                    Some(chunk) => {
                        progress.add_chunk(chunk.len() as u64);

                        match &mut storage {
                            Storage::FileInfo(_) => unreachable!(),
                            Storage::File(ref mut file) => file.write_all(&chunk).await?,
                            Storage::Memory(ref mut mem) => mem.put(chunk),
                        }
                        Ok(Self::Progress(response, storage, progress, c))
                    },
                    None => {
                        if let Storage::File(file) = &storage {
                            file.sync_all().await?;
                        }
                        Ok(Self::Finished(storage, c))
                    },
                }
            },
            Download::Finished(storage, c) => Ok(Download::Finished(storage, c)),
        }
    }
}

impl UpdateContent {
    pub fn show(&self) -> &str {
        match self {
            UpdateContent::DownloadFile(x) => x,
            UpdateContent::Decompress(x) => x,
            UpdateContent::HashLocalFile(x) => x,
            _ => "",
        }
    }
}

impl From<DownloadError> for ClientError {
    fn from(value: DownloadError) -> Self {
        match value {
            DownloadError::InvalidStatus(_) => ClientError::NetworkError,
            DownloadError::Reqwest(_) => ClientError::NetworkError,
            DownloadError::WriteError(_) => ClientError::IoError,
        }
    }
}

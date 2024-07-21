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

    #[allow(dead_code)]
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
            self.progress.bytes_per_sec = bytes_per_sec;
        }
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
    Start(RequestBuilder, Storage, DownloadContent, T),
    Progress(reqwest::Response, Storage, InternalProgressData, T),
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
                let progress =
                    InternalProgressData::new(ProgressData::new(total, content));
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

impl DownloadContent {
    pub fn show(&self) -> &str {
        match self {
            DownloadContent::SingleFile(x) => x,
            _ => "",
        }
    }
}

use futures_util::stream::Stream;
use iced::futures;

use super::{Progress, UpdateError};

pub fn download_stream<T>(
    rb: RequestBuilder,
    s: Storage,
    t: T,
) -> impl Stream<Item = Progress> {
    let state: Result<Download<T>, UpdateError> =
        Ok(Download::Start(rb, s, DownloadContent::FullZip, t));
    futures::stream::unfold(state, |state| async move {
        let download = match state {
            Err(e) => return Some((Progress::Errored(e.clone()), Err(e))),
            Ok(download) => download,
        };

        let res = match download {
            Download::Start(rb, s, dl, t) => Download::Start(rb, s, dl.clone(), t)
                .progress()
                .await
                .map(|s| Some((Progress::InProgress(ProgressData::new(0, dl)), Ok(s)))),
            Download::Progress(res, s, id, t) => {
                Download::Progress(res, s, id.clone(), t)
                    .progress()
                    .await
                    .map(|s| Some((Progress::InProgress(id.progress.clone()), Ok(s))))
            },
            Download::Finished(s, t) => Download::Finished(s, t)
                .progress()
                .await
                .map(|s| Some((Progress::Successful(None), Ok(s)))),
        };

        match res {
            Ok(ok) => ok,
            Err(e) => {
                let e = super::UpdateError::Custom(format!("{e}"));
                Some((Progress::Errored(e.clone()), Err(e)))
            },
        }
    })
}

use futures_util::stream::Stream;
use iced::futures;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct ProgressData {
    pub percent_complete: f32,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub bytes_per_sec: u64,
    pub remaining: Duration,
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(ProgressData),
    Finished,
    Errored(String),
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum State {
    Ready(String, PathBuf),
    Downloading {
        response: reqwest::Response,
        file: File,
        last_rate_check: Instant,
        downloaded_since_last_check: usize,
        total_bytes: u64,
        downloaded_bytes: u64,
        bytes_per_sec: u64,
        remaining: Duration,
    },
    Finished,
}

pub(crate) fn download(url: String, location: PathBuf) -> impl Stream<Item = Progress> {
    tracing::debug!(?url, ?location, "start downloading");
    futures::stream::unfold(State::Ready(url, location), |state| async move {
        match state {
            State::Ready(url, location) => {
                let response = crate::net::client::WEB_CLIENT.get(&url).send().await;

                match response {
                    Ok(response) => {
                        if !response.status().is_success() {
                            return Some((
                                Progress::Errored(format!(
                                    "Server returned invalid status: {:?}",
                                    response.status()
                                )),
                                State::Finished,
                            ));
                        }

                        if let Some(total) = response.content_length() {
                            match File::create(location).await {
                                Ok(file) => {
                                    Some((Progress::Started, State::Downloading {
                                        response,
                                        file,
                                        last_rate_check: Instant::now(),
                                        downloaded_since_last_check: 0,
                                        total_bytes: total,
                                        downloaded_bytes: 0,
                                        bytes_per_sec: 0,
                                        remaining: Duration::from_secs(0),
                                    }))
                                },
                                Err(e) => Some((
                                    Progress::Errored(format!("{:?}", e)),
                                    State::Finished,
                                )),
                            }
                        } else {
                            Some((
                                Progress::Errored("could not calculate file size".into()),
                                State::Finished,
                            ))
                        }
                    },
                    Err(e) => {
                        Some((Progress::Errored(format!("{:?}", e)), State::Finished))
                    },
                }
            },
            State::Downloading {
                mut response,
                mut file,
                mut last_rate_check,
                mut downloaded_since_last_check,
                total_bytes,
                mut downloaded_bytes,
                mut bytes_per_sec,
                mut remaining,
            } => match response.chunk().await {
                Ok(Some(chunk)) => {
                    downloaded_bytes += chunk.len() as u64;
                    let percent_complete =
                        (downloaded_bytes * 100) as f32 / total_bytes as f32;

                    // Calculate download speed
                    let since_last_check = Instant::now() - last_rate_check;
                    if since_last_check >= Duration::from_millis(500) {
                        bytes_per_sec = (downloaded_since_last_check as f32
                            / since_last_check.as_secs_f32())
                            as u64;
                        remaining = Duration::from_secs_f32(
                            (total_bytes - downloaded_bytes) as f32
                                / bytes_per_sec.max(1) as f32,
                        );
                        downloaded_since_last_check = 0;
                        last_rate_check = Instant::now()
                    }
                    downloaded_since_last_check += chunk.len();

                    match file.write_all(&chunk).await {
                        Ok(_) => Some((
                            Progress::Advanced(ProgressData {
                                percent_complete,
                                total_bytes,
                                downloaded_bytes,
                                bytes_per_sec,
                                remaining,
                            }),
                            State::Downloading {
                                response,
                                file,
                                last_rate_check,
                                downloaded_since_last_check,
                                total_bytes,
                                downloaded_bytes,
                                bytes_per_sec,
                                remaining,
                            },
                        )),
                        Err(e) => {
                            Some((Progress::Errored(format!("{}", e)), State::Finished))
                        },
                    }
                },
                Ok(None) => {
                    if let Err(e) = file.sync_all().await {
                        Some((Progress::Errored(format!("{}", e)), State::Finished))
                    } else {
                        Some((Progress::Finished, State::Finished))
                    }
                },
                Err(e) => Some((Progress::Errored(format!("{}", e)), State::Finished)),
            },
            State::Finished => {
                // We do not let the stream die, as it would start a
                // new download repeatedly if the user is not careful
                // in case of errors.
                #[allow(clippy::let_unit_value)]
                let _: () = futures::future::pending().await;
                None
            },
        }
    })
}

use futures_util::stream::{Stream};
use iced::futures;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(String, u64),
    Finished,
    Errored(String),
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Started => write!(f, "Download started..."),
            Self::Advanced(msg, percentage) => write!(f, "{} [{}%]", msg, percentage),
            Self::Finished => write!(f, "Download done!"),
            Self::Errored(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug)]
enum State {
    Ready(String, PathBuf),
    Downloading {
        response: reqwest::Response,
        file: File,
        total: u64,
        downloaded: u64,
    },
    Finished,
}

pub(crate) fn download(url: String, location: PathBuf) -> impl Stream<Item = Progress> {
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
                                        total,
                                        downloaded: 0,
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
                total,
                downloaded,
            } => match response.chunk().await {
                Ok(Some(chunk)) => {
                    let downloaded = downloaded + chunk.len() as u64;
                    let percentage = downloaded * 100 / total;
                    let progress = format!(
                        "{} / {}",
                        bytesize::ByteSize(downloaded),
                        bytesize::ByteSize(total)
                    );

                    match file.write_all(&chunk).await {
                        Ok(_) => Some((
                            Progress::Advanced(progress, percentage),
                            State::Downloading {
                                response,
                                file,
                                total,
                                downloaded,
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
                let _: () = futures::future::pending().await;
                None
            },
        }
    })
}

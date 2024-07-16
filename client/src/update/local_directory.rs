use std::{
    collections::VecDeque,
    path::{PathBuf, StripPrefixError},
};
use thiserror::Error;
use tokio::fs::ReadDir;

use super::download::{DownloadContent, InternalProgressData, ProgressData};

#[derive(Error, Debug)]
pub(super) enum LocalDirectoryError {
    #[error("tokio Error: ")]
    Tokio(#[from] std::io::Error),
    #[error("Invalid UTF8-Filename. this code requires filenames to match UTF8")]
    InvalidUtf8Filename,
    #[error("FileName not within Root Directory, is this some escape attack?")]
    StripPrefixError(#[from] StripPrefixError),
}

#[derive(Clone, Debug)]
pub(super) struct FileInformation {
    #[allow(dead_code)]
    pub path: PathBuf,
    // with stripped prefix
    pub local_path: String,
    pub crc32: u32,
}

#[derive(Debug)]
pub(super) enum LocalDirectory {
    Start(PathBuf),
    Progress(
        PathBuf,
        ReadDir,
        Vec<FileInformation>,
        VecDeque<PathBuf>,
        InternalProgressData,
    ),
    Finished(Vec<FileInformation>),
}

impl LocalDirectory {
    pub(super) async fn progress(self) -> Result<Self, LocalDirectoryError> {
        match self {
            LocalDirectory::Start(root) => {
                let dir = tokio::fs::read_dir(&root).await?;
                let progress = InternalProgressData::new(ProgressData::new(
                    0,
                    DownloadContent::SingleFile(root.to_string_lossy().to_string()),
                ));
                let nextdirs = VecDeque::new();
                Ok(Self::Progress(root, dir, Vec::new(), nextdirs, progress))
            },
            LocalDirectory::Progress(
                root,
                mut dir,
                mut fileinfo,
                mut nextdirs,
                mut progress,
            ) => match dir.next_entry().await? {
                Some(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        nextdirs.push_back(path);
                    } else {
                        let file_bytes = tokio::fs::read(&path).await?;
                        let crc32 = crc32fast::hash(&file_bytes);
                        let local_path = path
                            .strip_prefix(&root)?
                            .to_str()
                            .ok_or(LocalDirectoryError::InvalidUtf8Filename)?
                            .to_string();

                        fileinfo.push(FileInformation {
                            path,
                            crc32,
                            local_path,
                        });
                    }
                    Ok(Self::Progress(root, dir, fileinfo, nextdirs, progress))
                },
                None => {
                    if let Some(next) = nextdirs.pop_front() {
                        let dir = tokio::fs::read_dir(&next).await?;
                        progress.progress.content = DownloadContent::SingleFile(
                            next.to_string_lossy().to_string(),
                        );
                        Ok(Self::Progress(root, dir, fileinfo, nextdirs, progress))
                    } else {
                        Ok(LocalDirectory::Finished(fileinfo))
                    }
                },
            },
            LocalDirectory::Finished(storage) => Ok(LocalDirectory::Finished(storage)),
        }
    }
}

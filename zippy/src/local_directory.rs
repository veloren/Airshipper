use std::{
    path::{PathBuf, StripPrefixError},
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum LocalDirectoryError {
    #[error("tokio Error: ")]
    Tokio(#[from] std::io::Error),
    #[error("Invalid UTF8-Filename. this code requires filenames to match UTF8")]
    InvalidUtf8Filename,
    #[error("FileName not within Root Directory, is this some escape attack?")]
    StripPrefixError(#[from] StripPrefixError),
}

#[derive(Clone, Debug)]
pub struct FileInformation {
    pub path: PathBuf,
    // with stripped prefix
    pub local_path: String,
    pub crc32: u32,
}

pub struct LocalDirectory {
    path: std::path::PathBuf,
    files: Vec<FileInformation>,
}

impl LocalDirectory {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            files: Vec::new(),
        }
    }

    /// Reads the remote Zip and gives information on all contained files
    pub async fn fetch_file_information(
        &mut self,
    ) -> Result<Vec<FileInformation>, LocalDirectoryError> {
        self.ensure_local_dirs().await?;

        Ok(self.files.clone())
    }

    async fn ensure_local_dirs(&mut self) -> Result<(), LocalDirectoryError> {
        if !self.files.is_empty() {
            return Ok(());
        }
        let files = Arc::new(Mutex::new(Vec::<FileInformation>::new()));
        Self::visit_dir(self.path.clone(), files.clone(), self.path.clone()).await?;

        self.files = Arc::into_inner(files).unwrap().into_inner();
        Ok(())
    }

    async fn visit_dir(
        dir: PathBuf,
        files: Arc<Mutex<Vec<FileInformation>>>,
        root: PathBuf,
    ) -> Result<(), LocalDirectoryError> {
        let mut dir = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let fut = Box::pin(Self::visit_dir(path, files.clone(), root.clone()));
                fut.await?;
            } else {
                let file_bytes = tokio::fs::read(&path).await?;
                let crc32 = crc32fast::hash(&file_bytes);
                let local_path = path
                    .strip_prefix(&root)?
                    .to_str()
                    .ok_or(LocalDirectoryError::InvalidUtf8Filename)?
                    .to_string();

                let info = FileInformation {
                    path,
                    crc32,
                    local_path,
                };

                files.lock().await.push(info);
            }
        }
        Ok(())
    }
}

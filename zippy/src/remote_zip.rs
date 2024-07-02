use reqwest::header::RANGE;
use zip_core::{
    raw::{
        parse::{find_next_signature, Parse},
        CentralDirectoryHeader, EndOfCentralDirectory, EndOfCentralDirectoryFixed,
    },
    Signature,
};

#[derive(Debug)]
pub enum RemoteZipError {
    Reqwest(reqwest::Error),
    Zip(zip_core::raw::parse::DynamicSizeError),
    NoCentralDirectoryFound,
}

pub struct RemoteZip {
    client: reqwest::Client,
    url: reqwest::Url,
    content_length: Option<u64>,
    end_of_central_dir: Option<EndOfCentralDirectory>,
    central_directory: Vec<CentralDirectoryHeader>,
}

impl RemoteZip {
    pub fn new(client: reqwest::Client, url: reqwest::Url) -> Self {
        Self {
            client,
            url,
            content_length: None,
            end_of_central_dir: None,
            central_directory: Vec::new(),
        }
    }

    /// Reads the remote Zip and gives information on all contained files
    pub async fn fetch_file_information(
        &mut self,
    ) -> Result<Vec<CentralDirectoryHeader>, RemoteZipError> {
        self.ensure_central_directory().await?;

        Ok(self.central_directory.clone())
    }

    async fn ensure_content_length(&mut self) -> Result<(), RemoteZipError> {
        if self.content_length.is_none() {
            let document_size = self
                .client
                .get(self.url.clone())
                .send()
                .await
                .map_err(RemoteZipError::Reqwest)?;
            self.content_length = document_size.content_length();
        }
        Ok(())
    }

    async fn ensure_end_of_central_dir(&mut self) -> Result<(), RemoteZipError> {
        self.ensure_content_length().await?;
        const APPROX_MTU: u64 = 1400;
        let content_length = self.content_length.unwrap_or(0);
        let approx_eocd_start = content_length.saturating_sub(APPROX_MTU);
        // Get EOCD
        let range = format!("bytes={}-{}", approx_eocd_start, content_length);
        let eocd_res = self
            .client
            .get(self.url.clone())
            .header(RANGE, range)
            .send()
            .await
            .map_err(RemoteZipError::Reqwest)?;
        let eocd_bytes = eocd_res.bytes().await.map_err(RemoteZipError::Reqwest)?;

        let pos = find_next_signature(
            &eocd_bytes,
            EndOfCentralDirectoryFixed::END_OF_CENTRAL_DIR_SIGNATURE.to_le_bytes(),
        );

        if let Some(pos) = pos {
            let mut buf = &eocd_bytes[pos..];
            let eocd =
                EndOfCentralDirectory::from_buf(&mut buf).map_err(RemoteZipError::Zip)?;
            self.end_of_central_dir = Some(eocd);
        }

        Ok(())
    }

    async fn ensure_central_directory(&mut self) -> Result<(), RemoteZipError> {
        self.ensure_end_of_central_dir().await?;
        let eocd = self
            .end_of_central_dir
            .as_ref()
            .ok_or(RemoteZipError::NoCentralDirectoryFound)?;

        let cd_start = eocd.fixed.offset_of_start_of_central_directory_with_respect_to_the_starting_disk_number;
        let cd_end = cd_start.saturating_add(eocd.fixed.size_of_the_central_directory);
        // Get EOCD
        let range = format!("bytes={}-{}", cd_start, cd_end);
        let cd_res = self
            .client
            .get(self.url.clone())
            .header(RANGE, range)
            .send()
            .await
            .map_err(RemoteZipError::Reqwest)?;
        let mut cd_bytes = cd_res.bytes().await.map_err(RemoteZipError::Reqwest)?;

        let mut cds = Vec::new();

        while let Ok(cd) = CentralDirectoryHeader::from_buf(&mut cd_bytes) {
            if !cd.is_valid_signature() {
                break;
            }
            cds.push(cd);
        }

        self.central_directory = cds;

        Ok(())
    }
}

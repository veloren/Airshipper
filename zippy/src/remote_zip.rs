use std::io::Read;

use bytes::{Buf, Bytes};
use flate2::read::DeflateDecoder;
use reqwest::header::RANGE;
use thiserror::Error;
use zip_core::{
    raw::{
        parse::{find_next_signature, Parse},
        CentralDirectoryHeader, EndOfCentralDirectory, EndOfCentralDirectoryFixed,
        LocalFileHeader,
    },
    structs::CompressionMethod,
    Signature,
};

use crate::util::compression_supported;

#[derive(Debug, Error)]
pub enum RemoteZipError {
    #[error("Reqwest Error: ")]
    Reqwest(#[from] reqwest::Error),
    #[error("Zip Error: ")]
    Zip(#[from] zip_core::raw::parse::DynamicSizeError),
    #[error("Remote Zip invalid, no central Directory Found")]
    NoCentralDirectoryFound,
    #[error("Remote Zip probably valid, but compression is unsupported by Airshipper")]
    UnsuportedCompression,
    #[error("Remote Zip invalid, LocalHeader not found")]
    InvalidLocalHeader,
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

    /// downloads from remote and returned actuall content
    pub async fn download(
        &self,
        cd: &CentralDirectoryHeader,
    ) -> Result<bytes::Bytes, RemoteZipError> {
        let compression_method =
            compression_supported(cd).ok_or(RemoteZipError::UnsuportedCompression)?;
        let remote_file_size = cd.fixed.compressed_size as usize;

        const APPROX_MTU: u64 = 1400;
        let probable_end = cd.fixed.relative_offset_of_local_header as u64
            + remote_file_size as u64
            + APPROX_MTU;
        let range = format!(
            "bytes={}-{}",
            cd.fixed.relative_offset_of_local_header, probable_end
        );
        let data_res = self
            .client
            .get(self.url.clone())
            .header(RANGE, range)
            .send()
            .await?;
        let mut remote_bytes = data_res.bytes().await?;
        let local_header = LocalFileHeader::from_buf(&mut remote_bytes)?;
        if !local_header.is_valid_signature() {
            return Err(RemoteZipError::InvalidLocalHeader);
        }
        let remaining = remote_bytes.remaining();
        let mut file_bytes: &mut dyn Buf = &mut remote_bytes;
        let mut _chain;

        //println!("cd: {}", cd.fixed.compressed_size);
        if remaining < remote_file_size {
            let missing = remote_file_size - remote_bytes.remaining();
            let range =
                format!("bytes={}-{}", probable_end, probable_end + missing as u64);
            let data_res = self
                .client
                .get(self.url.clone())
                .header(RANGE, range)
                .send()
                .await?;
            let remote_bytes2 = data_res.bytes().await?;
            _chain = remote_bytes.chain(remote_bytes2);
            file_bytes = &mut _chain;
        }

        match compression_method {
            CompressionMethod::Deflated => {
                let compressed = file_bytes.take(remote_file_size);
                let mut deflate_reader = DeflateDecoder::new(compressed.reader());
                let mut decompressed = Vec::with_capacity(remote_file_size);
                deflate_reader.read_to_end(&mut decompressed).unwrap();
                Ok(Bytes::copy_from_slice(&decompressed))
            },
            CompressionMethod::Stored => Ok(file_bytes
                .take(remote_file_size)
                .copy_to_bytes(remote_file_size)),
            _ => Err(RemoteZipError::UnsuportedCompression),
        }
    }

    async fn ensure_content_length(&mut self) -> Result<(), RemoteZipError> {
        if self.content_length.is_none() {
            let document_size = self.client.head(self.url.clone()).send().await?;
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
            .await?;
        let eocd_bytes = eocd_res.bytes().await?;

        let pos = find_next_signature(
            &eocd_bytes,
            EndOfCentralDirectoryFixed::END_OF_CENTRAL_DIR_SIGNATURE.to_le_bytes(),
        );

        if let Some(pos) = pos {
            let mut buf = &eocd_bytes[pos..];
            let eocd = EndOfCentralDirectory::from_buf(&mut buf)?;
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
            .await?;
        let mut cd_bytes = cd_res.bytes().await?;

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

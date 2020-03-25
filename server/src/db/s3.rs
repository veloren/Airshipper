use crate::{models::Artifact, Result, CONFIG};
use rocket::http::ContentType;
use s3::{bucket::Bucket, credentials::Credentials};

pub struct S3Connection(Bucket);

impl S3Connection {
    pub fn new() -> Result<Self> {
        let credentials = Credentials::new(
            Some(CONFIG.bucket_access_key.clone()),
            Some(CONFIG.bucket_secret_key.clone()),
            None,
            None,
        );
        let mut bucket = Bucket::new(&CONFIG.bucket_name, CONFIG.bucket_region.clone(), credentials)?;
        bucket.add_header("x-amz-acl", "public-read");
        Ok(Self(bucket))
    }

    pub fn delete<T: ToString>(&self, artifact: &Artifact) -> Result<()> {
        let (_, code) = self.0.delete_object(&format!(
            "/nightly/{}",
            &artifact.download_path.file_name().unwrap().to_string_lossy()
        ))?;
        tracing::info!("Bucket responded with {}", code); // TODO: Check if that code is success!
        Ok(())
    }

    pub fn upload(&self, artifact: &Artifact) -> Result<()> {
        let (_, code) = self.0.put_object(
            &format!(
                "/nightly/{}",
                &artifact.download_path.file_name().unwrap().to_string_lossy()
            ), /* Unwrap safe. We always
                * have a file extension! */
            &std::fs::read(&artifact.download_path).expect("Failed to read file for upload!"),
            &ContentType::from_extension(
                &artifact
                    .download_path
                    .extension()
                    .unwrap_or(std::ffi::OsStr::new("zip"))
                    .to_string_lossy(),
            )
            .unwrap_or(ContentType::ZIP)
            .to_string(),
        )?;
        tracing::info!("Bucket responded with {}", code); // TODO: Check if that code is success!
        Ok(())
    }
}

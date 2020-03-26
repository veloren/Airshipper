use crate::{models::Artifact, Result, CONFIG};
use s3::{bucket::Bucket, credentials::Credentials, region::Region};
#[derive(Debug)]
pub struct S3Connection(Bucket);

impl S3Connection {
    pub fn new() -> Result<Self> {
        let credentials = Credentials::new(
            Some(CONFIG.bucket_access_key.clone()),
            Some(CONFIG.bucket_secret_key.clone()),
            None,
            None,
        );
        let region = Region::Custom {
            region: CONFIG.bucket_region.clone(),
            // For some reason the region needs to be included in the endpoint.
            endpoint: format!("{}.{}", CONFIG.bucket_region, CONFIG.bucket_endpoint),
        };
        let mut bucket = Bucket::new(&CONFIG.bucket_name, region, credentials)?;
        bucket.add_header("x-amz-acl", "public-read");
        Ok(Self(bucket))
    }

    #[tracing::instrument]
    pub async fn upload(&self, artifact: &Artifact) -> Result<u16> {
        let code = self
            .0
            .put_object_stream(&artifact.file_name, &format!("/nightly/{}", &artifact.file_name))
            .await?;
        Ok(code)
    }

    #[tracing::instrument]
    pub async fn delete(&self, artifact: &Artifact) -> Result<u16> {
        let (_, code) = self
            .0
            .delete_object(&format!("/nightly/{}", &artifact.file_name))
            .await?;
        Ok(code)
    }
}

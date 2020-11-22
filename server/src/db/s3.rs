use crate::{models::Artifact, Result, CONFIG};
use awscreds::Credentials;
use awsregion::Region;
use s3::bucket::Bucket;

#[derive(Debug)]
pub struct S3Connection(Bucket);

impl S3Connection {
    pub async fn new() -> Result<Self> {
        let credentials = Credentials::new(
            Some(&CONFIG.bucket_access_key),
            Some(&CONFIG.bucket_secret_key),
            None,
            None,
            None,
        )?;
        let region = Region::Custom {
            region: CONFIG.bucket_region.clone(),
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
            .tokio_put_object_stream(
                &mut tokio::fs::File::open(&artifact.file_name).await?,
                &format!("/nightly/{}", &artifact.file_name),
            )
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

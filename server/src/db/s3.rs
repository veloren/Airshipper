use crate::{models::Artifact, Result, CONFIG};
use bytes::BytesMut;
use futures::TryStreamExt;
use rusoto_core::{
    credential::{AwsCredentials, StaticProvider},
    HttpClient, Region,
};
use rusoto_s3::{S3Client as Client, *};
use tokio_util::codec;

pub struct S3Connection(Client);

impl Default for S3Connection {
    fn default() -> Self {
        let region = Region::Custom {
            name: "DigitalOcean".to_owned(),
            endpoint: format!(
                "https:://{}.{}.{}",
                CONFIG.bucket_name, CONFIG.bucket_region, CONFIG.bucket_endpoint
            ),
        };
        let creds = AwsCredentials::new(
            &CONFIG.bucket_access_key,
            &CONFIG.bucket_secret_key,
            None,
            None,
        );
        let client = Client::new_with(
            HttpClient::new().expect("Failed to create HTTPClient for S3!"),
            StaticProvider::from(creds),
            region,
        );

        Self(client)
    }
}

impl S3Connection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Uploads artifact and returns the corresponding etag.
    #[tracing::instrument(skip(self))]
    pub async fn upload(&self, artifact: &Artifact) -> Result<Option<String>> {
        self.upload_file("nightly", &artifact.file_name, &artifact.file_name)
            .await
    }

    /// Deletes artifact from s3 compatible storage.
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, artifact: &Artifact) -> Result<()> {
        self.delete_file("nightly", &artifact.file_name).await
    }

    /// Uploads file to s3 compatible storage.
    /// TODO: Convert to a multipart upload.
    ///
    /// Note: Does not stream the file which means
    /// we atleast need as much ram as the file size!
    #[tracing::instrument(skip(self))]
    async fn upload_file(
        &self,
        bucket: impl ToString + std::fmt::Debug,
        local_filename: impl ToString + std::fmt::Debug,
        dest_filename: impl ToString + std::fmt::Debug,
    ) -> Result<Option<String>> {
        let bucket = bucket.to_string();
        let local_filename = local_filename.to_string();
        let dest_filename = dest_filename.to_string();

        let meta = ::std::fs::metadata(&local_filename).unwrap();
        let file = tokio::fs::File::open(&local_filename).await.unwrap();
        let byte_stream = codec::FramedRead::new(file, codec::BytesCodec::new())
            .map_ok(BytesMut::freeze);

        let req = PutObjectRequest {
            bucket: bucket.to_owned(),
            key: dest_filename.to_owned(),
            content_length: Some(meta.len() as i64),
            body: Some(StreamingBody::new(byte_stream)),
            acl: Some("public-read".to_string()), // Object is readable by everyone.
            ..Default::default()
        };

        let result = self.0.put_object(req).await?;

        // Filter out non-hex characters from hash
        Ok(result
            .e_tag
            .map(|s| s.chars().filter(|c| c.is_digit(16)).collect()))
    }

    /// Deletes files from s3 compatible storage.
    #[tracing::instrument(skip(self))]
    async fn delete_file(
        &self,
        bucket: impl ToString + std::fmt::Debug,
        filename: impl ToString + std::fmt::Debug,
    ) -> Result<()> {
        let bucket = bucket.to_string();
        let filename = filename.to_string();

        let delete_object_req = DeleteObjectRequest {
            bucket,
            key: filename,
            ..Default::default()
        };

        let result = self.0.delete_object(delete_object_req).await?;
        dbg!(result);
        Ok(())
    }
}

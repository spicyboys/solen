use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_sdk_s3::{self as s3, Client};
use bytes::Bytes;

use crate::config::S3Config;

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: Arc<String>,
}

impl S3Client {
    pub async fn new(settings: S3Config) -> S3Client {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(settings.endpoint)
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                settings.access_key_id,
                settings.secret_access_key,
                None, // session token is not used with R2
                None,
                "R2",
            ))
            .region("auto") // Required by SDK but not used by R2
            .load()
            .await;

        S3Client {
            client: s3::Client::new(&config),
            bucket: Arc::new(settings.bucket),
        }
    }

    pub async fn upload_bytes(&self, key: &str, bytes: Bytes) -> Result<(), s3::Error> {
        self.client
            .put_object()
            .bucket(self.bucket.as_str())
            .key(key)
            .body(bytes.into())
            .send()
            .await?;
        Ok(())
    }

    pub async fn download_bytes(&self, key: &str) -> anyhow::Result<Bytes> {
        let resp = self
            .client
            .get_object()
            .bucket(self.bucket.as_str())
            .key(key)
            .send()
            .await?;
        let data = resp.body.collect().await?;
        Ok(data.into_bytes())
    }

    pub async fn delete(&self, key: &str) -> Result<(), s3::Error> {
        self.client
            .delete_object()
            .bucket(self.bucket.as_str())
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

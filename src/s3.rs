use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use s3::{AddressingStyle, Auth, Client, Credentials};

use crate::config::S3Config;

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: Arc<String>,
}

impl S3Client {
    pub async fn new(settings: S3Config) -> Result<S3Client> {
        let credentials = Credentials::new(settings.access_key_id, settings.secret_access_key)?;
        let client = Client::builder(settings.endpoint)?
            .region("auto") // Required for signing but not used by R2
            .auth(Auth::Static(credentials))
            .addressing_style(AddressingStyle::Path)
            .build()?;

        Ok(S3Client {
            client,
            bucket: Arc::new(settings.bucket),
        })
    }

    pub async fn upload_bytes(&self, key: &str, bytes: Bytes) -> Result<(), s3::Error> {
        self.client
            .objects()
            .put(self.bucket.as_str(), key)
            .body_bytes(bytes)
            .send()
            .await?;
        Ok(())
    }

    pub async fn download_bytes(&self, key: &str) -> anyhow::Result<Bytes> {
        let data = self
            .client
            .objects()
            .get(self.bucket.as_str(), key)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(data)
    }

    pub async fn delete(&self, key: &str) -> Result<(), s3::Error> {
        self.client
            .objects()
            .delete(self.bucket.as_str(), key)
            .send()
            .await?;
        Ok(())
    }
}
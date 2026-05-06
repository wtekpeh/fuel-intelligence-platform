use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::ReadingBatch;

pub struct ApiClient {
    client: Client,
    ingestion_url: String,
}

impl ApiClient {
    pub fn new(ingestion_url: String) -> Self {
        Self {
            client: Client::new(),
            ingestion_url,
        }
    }

    pub async fn send_batch(&self, batch: &ReadingBatch) -> Result<()> {
        let response = self
            .client
            .post(&self.ingestion_url)
            .json(batch)
            .send()
            .await
            .context("Failed to send batch to ingestion service")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            anyhow::bail!(
                "Ingestion service returned error: status={}, body={}",
                status,
                body
            );
        }

        Ok(())
    }
}

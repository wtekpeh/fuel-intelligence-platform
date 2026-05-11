use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::{HeartbeatRequest, ReadingBatch};

pub struct ApiClient {
    client: Client,
    ingestion_url: String,
    heartbeat_url: String,
}

impl ApiClient {
    pub fn new(ingestion_url: String, heartbeat_url: String) -> Self {
        Self {
            client: Client::new(),
            ingestion_url,
            heartbeat_url,
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

    pub async fn send_heartbeat(&self, heartbeat: &HeartbeatRequest) -> Result<()> {
        let response = self
            .client
            .post(&self.heartbeat_url)
            .json(heartbeat)
            .send()
            .await
            .context("Failed to send heartbeat to ingestion service")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            anyhow::bail!(
                "Heartbeat endpoint returned error: status={}, body={}",
                status,
                body
            );
        }

        Ok(())
    }
}

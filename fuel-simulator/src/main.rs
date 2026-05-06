mod api_client;
mod config;
mod models;
mod network;
mod simulator;
mod storage;
mod sync_queue;

use api_client::ApiClient;
use config::AppConfig;
use network::{NetworkSimulator, NetworkStatus};
use simulator::FuelSimulator;
use storage::FileStorage;
use sync_queue::SyncQueue;

use std::{thread, time};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;

    let mut simulator = FuelSimulator::new(&config);
    let mut network = NetworkSimulator::new();
    let storage = FileStorage::new()?;
    let api_client = ApiClient::new(config.ingestion_url.clone());

    let mut sync_queue = SyncQueue::new(config.device_id.clone(), config.batch_size);

    println!("Starting fuel simulator with real API sync...\n");

    loop {
        let reading = simulator.next_reading();
        let network_status = network.current_status();

        storage.save_reading(&reading)?;

        println!("Network Status: {:?}\n", network_status);
        println!("Generated reading:");
        println!("{}", serde_json::to_string_pretty(&reading)?);

        sync_queue.add_reading(reading);

        println!(
            "Pending readings waiting to sync: {}\n",
            sync_queue.pending_count()
        );

        match network_status {
            NetworkStatus::Online => {
                if sync_queue.is_ready_to_sync() {
                    let batch = sync_queue.build_batch();

                    println!("==================================");
                    println!("SYNCING BATCH TO INGESTION API");
                    println!("==================================");

                    match api_client.send_batch(&batch).await {
                        Ok(_) => {
                            sync_queue.mark_synced();
                            storage.save_synced_batch(&batch)?;
                            println!("Batch sent successfully.");
                        }
                        Err(err) => {
                            println!("Failed to send batch: {}", err);
                            println!("Batch kept in queue for retry.");
                        }
                    }

                    println!("==================================\n");
                } else {
                    println!("Online, but waiting for enough readings before syncing.\n");
                }
            }
            NetworkStatus::Offline => {
                println!("Device offline. Reading stored locally.\n");
            }
        }

        thread::sleep(time::Duration::from_secs(config.reading_sleep_seconds));
    }
}

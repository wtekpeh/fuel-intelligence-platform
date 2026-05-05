mod models;
mod network;
mod simulator;
mod storage;
mod sync_queue;

use network::{NetworkSimulator, NetworkStatus};
use simulator::FuelSimulator;
use storage::FileStorage;
use sync_queue::SyncQueue;

use std::{thread, time};

fn main() -> anyhow::Result<()> {
    let mut simulator = FuelSimulator::new();
    let mut network = NetworkSimulator::new();
    let storage = FileStorage::new()?;

    let mut sync_queue = SyncQueue::new("DEV001".to_string(), 5);

    println!("Starting fuel simulator with online/offline sync + file logging...\n");

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
                    let batch = sync_queue.create_batch();

                    storage.save_synced_batch(&batch)?;

                    println!("==================================");
                    println!("SYNCING BATCH TO SERVER");
                    println!("==================================");
                    println!("{}", serde_json::to_string_pretty(&batch)?);
                    println!("==================================\n");
                } else {
                    println!("Online, but waiting for enough readings before syncing.\n");
                }
            }
            NetworkStatus::Offline => {
                println!("Device offline. Reading stored locally.\n");
            }
        }

        thread::sleep(time::Duration::from_secs(2));
    }
}

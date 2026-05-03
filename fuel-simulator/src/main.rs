mod models;
mod simulator;
mod sync_queue;

use simulator::FuelSimulator;
use sync_queue::SyncQueue;

use std::{thread, time};

fn main() {
    let mut simulator = FuelSimulator::new();

    let mut sync_queue = SyncQueue::new(
        "DEV001".to_string(),
        5, // send after every 5 readings
    );

    println!("Starting fuel simulator with delayed sync...\n");

    loop {
        let reading = simulator.next_reading();

        println!("Generated reading:");
        println!("{}", serde_json::to_string_pretty(&reading).unwrap());

        sync_queue.add_reading(reading);

        println!(
            "Pending readings waiting to sync: {}\n",
            sync_queue.pending_count()
        );

        if sync_queue.is_ready_to_sync() {
            let batch = sync_queue.create_batch();

            println!("==================================");
            println!("SYNCING BATCH TO SERVER");
            println!("==================================");
            println!("{}", serde_json::to_string_pretty(&batch).unwrap());
            println!("==================================\n");
        }

        thread::sleep(time::Duration::from_secs(2));
    }
}

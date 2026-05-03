mod models;
mod simulator;

use simulator::FuelSimulator;
use std::{thread, time};

fn main() {
    let mut simulator = FuelSimulator::new();

    println!("Starting fuel simulator...\n");

    loop {
        let reading = simulator.next_reading();

        let json = serde_json::to_string_pretty(&reading).unwrap();
        println!("{}", json);

        thread::sleep(time::Duration::from_secs(2));
    }
}

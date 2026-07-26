use esp_hal::delay::Delay;
use esp_println::println;

use crate::{drivers::Modem, storage::RecordStorage, telemetry::publisher};

pub fn replay_pending_records<S>(modem: &mut Modem, delay: &Delay, storage: Option<&mut S>)
where
    S: RecordStorage,
{
    println!("========================");
    println!("ORBI QUEUE REPLAY");
    println!("========================");

    println!("Starting persistent queue replay...");

    if publisher::flush_queue(modem, delay, storage) {
        println!("Replay completed successfully.");
    } else {
        println!("Replay finished. Queue may still contain pending records.");
    }
}

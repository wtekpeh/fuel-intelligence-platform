use esp_hal::delay::Delay;
use esp_println::println;

use crate::{drivers::Modem, network::http, storage::RecordStorage, telemetry::payload};

pub fn replay_pending_records<S>(modem: &mut Modem, delay: &Delay, mut storage: Option<&mut S>)
where
    S: RecordStorage,
{
    println!("========================");
    println!("ORBI QUEUE REPLAY");
    println!("========================");

    const MAX_REPLAY_PER_BOOT: usize = 100;

    let mut replayed = 0;

    loop {
        if replayed >= MAX_REPLAY_PER_BOOT {
            println!("Reached replay limit for this boot.");
            break;
        }

        let queued_record = match storage.as_deref_mut() {
            Some(storage) => match storage.read_first_record() {
                Some(record) => record,

                None => {
                    println!("Queue empty.");
                    break;
                }
            },

            None => {
                println!("SD storage unavailable.");
                break;
            }
        };

        let (device_id, timestamp) = match payload::extract_replay_identity(queued_record.as_str())
        {
            Some(identity) => identity,

            None => {
                println!("Invalid queued record.");
                break;
            }
        };

        if let Some(storage) = storage.as_deref_mut() {
            if storage.is_acknowledged(device_id, timestamp) {
                println!("Already acknowledged.");

                storage.remove_first_record();

                continue;
            }
        }

        let replay_payload = match payload::build_replay_batch_payload(queued_record.as_str()) {
            Some(payload) => payload,

            None => {
                println!("Replay payload failed.");
                break;
            }
        };

        println!("Replay record {}", replayed + 1);

        let upload_success = http::send_payload(modem, delay, &replay_payload);

        if !upload_success {
            println!("Replay stopped because upload failed.");
            break;
        }

        if let Some(storage) = storage.as_deref_mut() {
            if !storage.append_ack(device_id, timestamp) {
                println!("ACK failed.");
                break;
            }

            if !storage.remove_first_record() {
                println!("Queue cleanup failed.");
                break;
            }
        }

        replayed += 1;
    }

    println!("Replay completed. {} record(s) processed.", replayed);
}

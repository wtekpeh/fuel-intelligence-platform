use embassy_time::{Duration, Timer};
use esp_hal::delay::Delay;
use esp_println::println;

use crate::{
    drivers::Modem,
    storage::RecordStorage,
    telemetry::publisher::{self, QueueBatchOutcome},
};

/*
 * Replay pending telemetry without monopolising the Embassy executor.
 *
 * Each replay iteration processes at most one bounded queue batch.
 * After a successful batch, the task yields before attempting the next one.
 *
 * There is deliberately NO total replay-record limit here.
 *
 * A backlog of 10, 100, 1,000 or more records can therefore continue
 * draining until ORBIQ.LOG contains no pending records.
 */
pub async fn replay_pending_records<S>(
    modem: &mut Modem<'_>,
    delay: &Delay,
    storage: Option<&mut S>,
) where
    S: RecordStorage,
{
    println!("========================");
    println!("ORBI QUEUE REPLAY");
    println!("========================");

    println!("Starting persistent queue replay...");

    let storage = match storage {
        Some(storage) => storage,

        None => {
            println!("SD storage unavailable. Replay skipped.");

            return;
        }
    };

    let mut total_replayed_records = 0usize;

    loop {
        match publisher::flush_one_queue_batch(modem, delay, storage).await {
            QueueBatchOutcome::NoPendingRecords => {
                println!(
                    "Replay complete. {} queued record(s) replayed.",
                    total_replayed_records
                );

                break;
            }

            QueueBatchOutcome::Uploaded {
                removed_records,
                expected_records,
            } => {
                total_replayed_records += removed_records;

                println!(
                    "Replay progress: {} record(s) replayed.",
                    total_replayed_records
                );

                /*
                 * A partially removed batch already has persisted ACKs.
                 * Stop here so the next replay attempt can recover the
                 * acknowledged queue front safely.
                 */
                if removed_records != expected_records {
                    println!("Replay stopped after incomplete queue removal.");

                    break;
                }

                /*
                 * Cooperative scheduling boundary.
                 *
                 * The HTTP and SD work inside one batch is still currently
                 * synchronous, but after every bounded batch we give Embassy
                 * an opportunity to run other ready tasks.
                 */
                Timer::after(Duration::from_millis(1)).await;
            }

            QueueBatchOutcome::Failed => {
                println!("Replay stopped because the current batch failed.");
                println!("Pending records remain safely queued.");

                break;
            }
        }
    }
}

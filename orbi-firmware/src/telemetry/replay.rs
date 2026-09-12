use embassy_time::{Duration, Timer};
use esp_println::println;

use crate::{
    network::modem_owner,
    storage::owner::{self, ReplayPrepareResult},
    telemetry::payload,
};

/*
 * Replay pending telemetry without owning either the modem
 * or the SD card directly.
 *
 * Each replay iteration:
 *
 * 1. asks the storage owner for one bounded queue batch;
 * 2. asks the modem owner to upload that batch;
 * 3. asks the storage owner to persist ACKs and remove that batch;
 * 4. yields before attempting another batch.
 *
 * There is deliberately NO total replay-record limit here.
 *
 * A backlog of 10, 100, 1,000 or more records can therefore
 * continue draining until ORBIQ.LOG contains no pending records.
 */
pub async fn replay_pending_records() {
    println!("========================");
    println!("ORBI QUEUE REPLAY");
    println!("========================");

    println!("Starting persistent queue replay...");

    let mut total_replayed_records = 0usize;

    loop {
        /*
         * Ask the dedicated storage owner for one bounded batch.
         *
         * The storage owner also performs bounded recovery of records
         * whose ACKs were persisted previously but whose queue removal
         * was interrupted.
         */
        let queued_records = match owner::request_replay_prepare().await {
            ReplayPrepareResult::NoPendingRecords => {
                println!(
                    "Replay complete. {} queued record(s) replayed.",
                    total_replayed_records
                );

                break;
            }

            ReplayPrepareResult::Batch(records) => records,

            ReplayPrepareResult::StorageUnavailable => {
                println!("SD storage unavailable. Replay stopped.");

                break;
            }
        };

        let batch_record_count = queued_records.len();

        /*
         * Build the HTTP payload from exactly the records returned
         * by the storage owner.
         */
        let batch_payload = match payload::build_queue_batch_payload(&queued_records) {
            Some(payload) => payload,

            None => {
                println!("Failed to build telemetry queue batch payload.");
                println!("Queue remains unchanged.");

                break;
            }
        };

        println!("========================");
        println!("QUEUED TELEMETRY BATCH");
        println!("========================");
        println!("Records: {}", batch_record_count);
        println!("{}", batch_payload);

        /*
         * The replay task never touches the modem directly.
         *
         * Replay modem work remains lower priority than foreground
         * live telemetry work inside the modem owner.
         */
        if !modem_owner::request_replay_batch(batch_payload).await {
            println!("Queued telemetry upload failed.");
            println!("Queue remains unchanged.");
            println!("Replay will retry after 10 seconds.");

            /*
             * Do not terminate the background replay task because of a
             * temporary network/server failure.
             *
             * The queued records remain untouched, so the same oldest
             * batch can be retried later.
             *
             * The delay also prevents replay from hammering the modem or
             * backend continuously while connectivity is unhealthy.
             */
            Timer::after(Duration::from_secs(10)).await;

            continue;
        }
        /*
         * The HTTP upload succeeded.
         *
         * Return the exact uploaded records to the storage owner so
         * it can:
         *
         * 1. persist their ACKs;
         * 2. remove only those acknowledged queue entries.
         */
        let finalize_result = owner::request_replay_finalize(queued_records).await;

        total_replayed_records += finalize_result.removed_records;

        println!(
            "Published and removed {} queued telemetry record(s).",
            finalize_result.removed_records
        );

        println!(
            "Replay progress: {} record(s) replayed.",
            total_replayed_records
        );

        /*
         * If ACK persistence or queue removal was incomplete, stop.
         *
         * Any ACKs that were successfully persisted remain durable.
         * A later replay pass can recover acknowledged records from
         * the front of ORBIQ.LOG.
         */
        if !finalize_result.success {
            println!(
                "Only {} of {} uploaded record(s) were removed.",
                finalize_result.removed_records, finalize_result.expected_records
            );

            println!("Replay stopped after incomplete queue finalization.");

            break;
        }

        /*
         * Cooperative scheduling boundary between replay batches.
         *
         * This is intentionally not a total replay limit.
         * It simply gives other ready Embassy tasks another
         * opportunity before replay requests the next batch.
         */
        Timer::after(Duration::from_millis(1)).await;
    }
}

#[embassy_executor::task]
pub async fn replay_task() {
    loop {
        /*
         * Run one replay pass.
         *
         * A pass may finish because:
         *
         * - the queue is empty;
         * - SD storage is temporarily unavailable;
         * - payload construction fails;
         * - queue finalization is incomplete.
         *
         * The Embassy task itself must remain alive.
         *
         * New telemetry may be persisted to ORBIQ.LOG later if live
         * cloud publishing fails, so replay must periodically return
         * and inspect the persistent queue again.
         */
        replay_pending_records().await;

        println!("Replay pass finished.");
        println!("Waiting 10 seconds before checking ORBIQ.LOG again.");

        /*
         * Avoid continuously polling the SD card when no backlog exists.
         *
         * Foreground live telemetry retains modem priority, while the
         * background replay service periodically checks whether new
         * queued telemetry has appeared.
         */
        Timer::after(Duration::from_secs(10)).await;
    }
}

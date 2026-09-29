use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};

use esp_hal::peripherals::{GPIO13, GPIO14, GPIO15, GPIO2, SPI2};

use esp_println::println;
use heapless::{String, Vec};

use crate::storage::{
    owned_record::{OwnedAckRecord, OwnedGnssDiagnosticRecord, OwnedTelemetryRecord},
    RecordStorage,
};

/*
 * Replay remains deliberately bounded.
 *
 * This matches the existing telemetry publisher batch size.
 * A replay operation must never attempt to drain the entire SD queue
 * while foreground telemetry is waiting.
 */
const REPLAY_BATCH_SIZE: usize = 4;

/*
 * One bounded set of raw queue records returned by the storage owner.
 *
 * Records remain in their existing serialized JSON-line representation.
 */
pub type ReplayQueueBatch = Vec<String<768>, REPLAY_BATCH_SIZE>;

/*
 * Result of asking the storage owner for the next replay batch.
 */
pub enum ReplayPrepareResult {
    /*
     * No queue records remain after bounded acknowledged-record cleanup.
     */
    NoPendingRecords,

    /*
     * One bounded batch is ready for transmission.
     */
    Batch(ReplayQueueBatch),

    /*
     * Persistent storage is unavailable.
     */
    StorageUnavailable,
}

/*
 * Result of revalidating replay work immediately before transmission.
 *
 * Replay preparation returns a snapshot of the queue. While that snapshot
 * is waiting for modem access, foreground live telemetry may successfully
 * upload and ACK one or more records from the front of ORBIQ.LOG.
 *
 * Revalidation therefore asks the storage owner for a fresh view of the
 * queue before replay performs its HTTP upload.
 */
pub enum ReplayRevalidateResult {
    /*
     * No pending records remain after acknowledged-record cleanup.
     */
    NoPendingRecords,

    /*
     * A fresh bounded batch is still pending and may be transmitted.
     */
    Batch(ReplayQueueBatch),

    /*
     * Persistent storage is unavailable.
     */
    StorageUnavailable,
}

pub struct ReplayFinalizeRequest {
    pub records: ReplayQueueBatch,
}

pub struct ReplayFinalizeResult {
    pub removed_records: usize,
    pub expected_records: usize,
    pub success: bool,
}

/*
 * GNSS diagnostic records waiting to be written to ORBIGNSS.LOG.
 *
 * The record owns all of its data, so it can safely cross the
 * Embassy task boundary.
 */
static GNSS_DIAGNOSTIC_REQUESTS: Channel<CriticalSectionRawMutex, OwnedGnssDiagnosticRecord, 2> =
    Channel::new();

/*
 * Result of the most recent GNSS diagnostic persistence request.
 *
 * Current runtime behaviour sends one diagnostic request at a time,
 * so one response signal is sufficient.
 */
static GNSS_DIAGNOSTIC_RESPONSE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

/*
 * Called by runtime code when a GNSS diagnostic sample should be
 * persisted.
 *
 * The caller never receives access to the SD card or VolumeManager.
 * It only sends an owned record to the storage owner.
 */
pub async fn request_gnss_diagnostic(record: OwnedGnssDiagnosticRecord) -> bool {
    GNSS_DIAGNOSTIC_REQUESTS.send(record).await;

    STORAGE_WORK_AVAILABLE.signal(());

    GNSS_DIAGNOSTIC_RESPONSE.wait().await
}

static LIVE_ACK_REQUESTS: Channel<CriticalSectionRawMutex, OwnedAckRecord, 2> = Channel::new();

static LIVE_ACK_RESPONSE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

/*
 * Live telemetry records waiting to be persisted to ORBIQ.LOG.
 */
static LIVE_TELEMETRY_REQUESTS: Channel<CriticalSectionRawMutex, OwnedTelemetryRecord, 2> =
    Channel::new();

/*
 * Result of the most recent live telemetry persistence request.
 */
static LIVE_TELEMETRY_RESPONSE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static STORAGE_WORK_AVAILABLE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static REPLAY_PREPARE_REQUESTS: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

static REPLAY_PREPARE_RESPONSE: Signal<CriticalSectionRawMutex, ReplayPrepareResult> =
    Signal::new();

static REPLAY_REVALIDATE_REQUESTS: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

static REPLAY_REVALIDATE_RESPONSE: Signal<CriticalSectionRawMutex, ReplayRevalidateResult> =
    Signal::new();

static REPLAY_FINALIZE_REQUESTS: Channel<CriticalSectionRawMutex, ReplayFinalizeRequest, 1> =
    Channel::new();

static REPLAY_FINALIZE_RESPONSE: Signal<CriticalSectionRawMutex, ReplayFinalizeResult> =
    Signal::new();

/*
 * Dedicated owner of the ORBI SD-card hardware and persistent storage.
 *
 * SPI2, the SD GPIO pins, VolumeManager and filesystem state remain
 * private to this task for the complete runtime lifetime.
 */
#[embassy_executor::task]
pub async fn storage_owner_task(
    spi2: SPI2<'static>,
    miso: GPIO2<'static>,
    mosi: GPIO15<'static>,
    sclk: GPIO14<'static>,
    cs: GPIO13<'static>,
) {
    println!("========================");
    println!("ORBI STORAGE OWNER");
    println!("========================");

    /*
     * Initialize the SD card inside its permanent owner.
     */
    let mut persistent_storage = crate::storage::sdcard::initialize(spi2, miso, mosi, sclk, cs);

    if persistent_storage.is_some() {
        println!("Storage owner initialized persistent SD storage.");
    } else {
        println!("WARNING: Storage owner could not initialize persistent SD storage.");
    }

    loop {
        /*
         * Live telemetry persistence gets first opportunity because
         * durable telemetry is more important than diagnostic logging.
         */
        if let Ok(owned_record) = LIVE_TELEMETRY_REQUESTS.try_receive() {
            let saved = match persistent_storage.as_mut() {
                Some(storage) => {
                    let borrowed_record = owned_record.as_borrowed();

                    storage.append_record(&borrowed_record)
                }

                None => {
                    println!(
                        "Live telemetry persistence failed because SD storage is unavailable."
                    );

                    false
                }
            };

            LIVE_TELEMETRY_RESPONSE.signal(saved);

            continue;
        }

        if let Ok(ack) = LIVE_ACK_REQUESTS.try_receive() {
            let saved = match persistent_storage.as_mut() {
                Some(storage) => storage.append_ack(ack.device_id(), ack.timestamp()),

                None => {
                    println!(
                        "Live telemetry ACK persistence failed because SD storage is unavailable."
                    );

                    false
                }
            };

            LIVE_ACK_RESPONSE.signal(saved);

            continue;
        }

        /*
         * GNSS diagnostics are foreground storage work too,
         * but lower priority than durable live telemetry.
         */
        if let Ok(owned_record) = GNSS_DIAGNOSTIC_REQUESTS.try_receive() {
            let saved = match persistent_storage.as_mut() {
                Some(storage) => {
                    let borrowed_record = owned_record.as_borrowed();

                    storage.append_gnss_diagnostic(&borrowed_record)
                }

                None => {
                    println!(
                        "GNSS diagnostic persistence failed because SD storage is unavailable."
                    );

                    false
                }
            };

            GNSS_DIAGNOSTIC_RESPONSE.signal(saved);

            continue;
        }

        if REPLAY_PREPARE_REQUESTS.try_receive().is_ok() {
            let result = match persistent_storage.as_mut() {
                Some(storage) => {
                    /*
                     * Recover only a bounded number of records that were
                     * already acknowledged but not yet removed.
                     *
                     * The bound prevents replay cleanup from monopolizing
                     * the storage owner.
                     */
                    let mut recovered_records = 0usize;

                    while recovered_records < REPLAY_BATCH_SIZE {
                        let queued_record = match storage.read_first_record() {
                            Some(record) => record,

                            None => {
                                break;
                            }
                        };

                        let (device_id, timestamp) =
                            match crate::telemetry::payload::extract_replay_identity(
                                queued_record.as_str(),
                            ) {
                                Some(identity) => identity,

                                None => {
                                    println!("Invalid record found at the front of ORBIQ.LOG.");
                                    println!("Acknowledged replay cleanup stopped.");

                                    break;
                                }
                            };

                        if !storage.is_acknowledged(device_id, timestamp) {
                            break;
                        }

                        println!("Removing previously acknowledged queued record.");

                        if !storage.remove_first_record() {
                            println!("Failed to remove previously acknowledged queued record.");

                            break;
                        }

                        recovered_records += 1;
                    }

                    if recovered_records > 0 {
                        println!(
                            "Removed {} previously acknowledged queued record(s).",
                            recovered_records
                        );
                    }

                    let queued_records =
                        storage.read_first_records::<REPLAY_BATCH_SIZE>(REPLAY_BATCH_SIZE);

                    if queued_records.is_empty() {
                        ReplayPrepareResult::NoPendingRecords
                    } else {
                        ReplayPrepareResult::Batch(queued_records)
                    }
                }

                None => {
                    println!("Replay preparation failed because SD storage is unavailable.");

                    ReplayPrepareResult::StorageUnavailable
                }
            };

            REPLAY_PREPARE_RESPONSE.signal(result);

            continue;
        }

        /*
         * Revalidate replay work immediately before transmission.
         *
         * A batch returned by replay preparation is only a snapshot.
         * While replay is waiting for modem access, foreground live
         * telemetry may upload and persist ACKs for records at the
         * front of ORBIQ.LOG.
         *
         * Therefore:
         *
         * 1. remove any now-acknowledged records from the queue front;
         * 2. read a fresh bounded batch from persistent storage;
         * 3. return that fresh batch to replay.
         *
         * ORBIQ.LOG remains the authoritative queue state.
         */
        if REPLAY_REVALIDATE_REQUESTS.try_receive().is_ok() {
            let result = match persistent_storage.as_mut() {
                Some(storage) => {
                    let mut recovered_records = 0usize;

                    /*
                     * Keep cleanup bounded for the same reason as replay
                     * preparation: storage work must not monopolize the
                     * Embassy executor.
                     */
                    while recovered_records < REPLAY_BATCH_SIZE {
                        let queued_record = match storage.read_first_record() {
                            Some(record) => record,

                            None => {
                                break;
                            }
                        };

                        let (device_id, timestamp) =
                            match crate::telemetry::payload::extract_replay_identity(
                                queued_record.as_str(),
                            ) {
                                Some(identity) => identity,

                                None => {
                                    println!(
                                        "Invalid record found at the front of ORBIQ.LOG during replay revalidation."
                                    );

                                    println!("Replay revalidation cleanup stopped.");

                                    break;
                                }
                            };

                        if !storage.is_acknowledged(device_id, timestamp) {
                            break;
                        }

                        println!(
                            "Replay revalidation found an already acknowledged queued record."
                        );

                        if !storage.remove_first_record() {
                            println!(
                                "Failed to remove acknowledged record during replay revalidation."
                            );

                            break;
                        }

                        recovered_records += 1;
                    }

                    if recovered_records > 0 {
                        println!(
                            "Replay revalidation removed {} acknowledged queued record(s).",
                            recovered_records
                        );
                    }

                    /*
                     * Do not return the old in-memory replay snapshot.
                     *
                     * Read the queue again so replay receives the current
                     * authoritative front of ORBIQ.LOG.
                     */
                    let queued_records =
                        storage.read_first_records::<REPLAY_BATCH_SIZE>(REPLAY_BATCH_SIZE);

                    if queued_records.is_empty() {
                        ReplayRevalidateResult::NoPendingRecords
                    } else {
                        ReplayRevalidateResult::Batch(queued_records)
                    }
                }

                None => {
                    println!("Replay revalidation failed because SD storage is unavailable.");

                    ReplayRevalidateResult::StorageUnavailable
                }
            };

            REPLAY_REVALIDATE_RESPONSE.signal(result);

            continue;
        }

        if let Ok(request) = REPLAY_FINALIZE_REQUESTS.try_receive() {
            let expected_records = request.records.len();

            let result = match persistent_storage.as_mut() {
                Some(storage) => {
                    /*
                     * Persist ACKs first.
                     *
                     * If any ACK fails, remove nothing from ORBIQ.LOG.
                     */
                    let mut all_acks_persisted = true;

                    for queued_record in request.records.iter() {
                        let (device_id, timestamp) =
                            match crate::telemetry::payload::extract_replay_identity(
                                queued_record.as_str(),
                            ) {
                                Some(identity) => identity,

                                None => {
                                    println!(
                                        "Unable to extract identity from uploaded replay record."
                                    );

                                    all_acks_persisted = false;

                                    break;
                                }
                            };

                        if !storage.append_ack(device_id, timestamp) {
                            println!("Failed to persist replay telemetry ACK.");

                            all_acks_persisted = false;

                            break;
                        }
                    }

                    if !all_acks_persisted {
                        ReplayFinalizeResult {
                            removed_records: 0,
                            expected_records,
                            success: false,
                        }
                    } else {
                        /*
                         * ACKs are safely persisted.
                         *
                         * Remove only the records belonging to this bounded
                         * uploaded batch.
                         */
                        let mut removed_records = 0usize;

                        for _ in 0..expected_records {
                            if !storage.remove_first_record() {
                                println!("Failed while removing acknowledged replay record.");

                                break;
                            }

                            removed_records += 1;
                        }

                        ReplayFinalizeResult {
                            removed_records,
                            expected_records,
                            success: removed_records == expected_records,
                        }
                    }
                }

                None => {
                    println!("Replay finalization failed because SD storage is unavailable.");

                    ReplayFinalizeResult {
                        removed_records: 0,
                        expected_records,
                        success: false,
                    }
                }
            };

            REPLAY_FINALIZE_RESPONSE.signal(result);

            continue;
        }

        /*
         * No storage work is currently queued.
         */
        STORAGE_WORK_AVAILABLE.wait().await;
    }
}

pub async fn request_live_telemetry_persistence(record: OwnedTelemetryRecord) -> bool {
    LIVE_TELEMETRY_REQUESTS.send(record).await;

    STORAGE_WORK_AVAILABLE.signal(());

    LIVE_TELEMETRY_RESPONSE.wait().await
}

pub async fn request_live_ack(ack: OwnedAckRecord) -> bool {
    LIVE_ACK_REQUESTS.send(ack).await;

    STORAGE_WORK_AVAILABLE.signal(());

    LIVE_ACK_RESPONSE.wait().await
}

pub async fn request_replay_prepare() -> ReplayPrepareResult {
    REPLAY_PREPARE_REQUESTS.send(()).await;

    STORAGE_WORK_AVAILABLE.signal(());

    REPLAY_PREPARE_RESPONSE.wait().await
}

pub async fn request_replay_revalidate() -> ReplayRevalidateResult {
    REPLAY_REVALIDATE_REQUESTS.send(()).await;

    STORAGE_WORK_AVAILABLE.signal(());

    REPLAY_REVALIDATE_RESPONSE.wait().await
}

pub async fn request_replay_finalize(records: ReplayQueueBatch) -> ReplayFinalizeResult {
    REPLAY_FINALIZE_REQUESTS
        .send(ReplayFinalizeRequest { records })
        .await;

    STORAGE_WORK_AVAILABLE.signal(());

    REPLAY_FINALIZE_RESPONSE.wait().await
}

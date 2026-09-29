use core::sync::atomic::{AtomicBool, Ordering};

/*
 * Indicates whether a foreground live-telemetry transaction is currently
 * in progress.
 *
 * A live transaction covers the complete durable lifecycle:
 *
 *     persist live telemetry to ORBIQ.LOG
 *              ↓
 *     upload the live telemetry
 *              ↓
 *     persist its ACK to ORBIACK.LOG
 *
 * Replay must not prepare queue records while this transaction is active.
 *
 * This prevents the replay task from reading a newly queued live record
 * after its HTTP upload succeeds but before its ACK has been persisted.
 */
static LIVE_TELEMETRY_TRANSACTION_ACTIVE: AtomicBool = AtomicBool::new(false);

/*
 * Mark the beginning of a live-telemetry transaction.
 */
pub fn begin_live_telemetry_transaction() {
    LIVE_TELEMETRY_TRANSACTION_ACTIVE.store(true, Ordering::Release);
}

/*
 * Mark the end of a live-telemetry transaction.
 */
pub fn end_live_telemetry_transaction() {
    LIVE_TELEMETRY_TRANSACTION_ACTIVE.store(false, Ordering::Release);
}

/*
 * Check whether foreground live telemetry currently has an unfinished
 * durable transaction.
 */
pub fn live_telemetry_transaction_active() -> bool {
    LIVE_TELEMETRY_TRANSACTION_ACTIVE.load(Ordering::Acquire)
}

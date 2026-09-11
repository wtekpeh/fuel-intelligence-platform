use crate::{
    drivers::{gnss::GpsInfo, modem::Modem},
    network::state::NetworkState,
};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};

use heapless::String;

/*
 * A7670 modem ownership and request coordination.
 *
 * This module will become the single runtime owner of the A7670 modem.
 *
 * The modem is shared by several firmware responsibilities:
 *
 * - GNSS acquisition
 * - live telemetry HTTP publishing
 * - heartbeat HTTP publishing
 * - network-state checks
 * - network diagnostics
 * - persistent queue replay
 *
 * These operations must not issue AT commands independently because they all
 * use the same UART-backed modem.
 *
 * The owner will eventually receive requests from other Embassy tasks and
 * execute them one at a time.
 *
 * Replay is intentionally classified separately so that live/runtime work can
 * be given priority between replay batches.
 */

/// Identifies the type of operation that needs exclusive access to the modem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModemRequestKind {
    /// Obtain the latest GNSS fix.
    GnssFix,

    /// Publish newly acquired live telemetry.
    LiveTelemetry,

    /// Publish the independent device heartbeat.
    Heartbeat,

    /// Read SIM/network registration and packet-data state.
    NetworkState,

    /// Run detailed modem/network diagnostics.
    Diagnostics,

    /// Upload one bounded batch from the persistent telemetry queue.
    ReplayBatch,
}

/// Describes whether a modem operation belongs to normal live firmware work
/// or background replay work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModemRequestPriority {
    Foreground,
    Replay,
}

/// Result produced after the modem owner completes a requested operation.
///
/// These are owned values so they can later be transferred safely between
/// Embassy tasks without borrowing data from the modem-owner stack.
pub enum ModemResponse {
    /// Result of an AT+CGNSSINFO request.
    GnssFix(Option<GpsInfo>),

    /// Current SIM / LTE / packet-data state.
    NetworkState(NetworkState),

    /// Result of an HTTP operation such as live telemetry, heartbeat,
    /// or one replay batch.
    HttpResult(bool),

    /// Indicates that a diagnostics request finished.
    DiagnosticsComplete,
}

impl ModemRequestKind {
    /// Return the scheduling class for this modem operation.
    ///
    /// All normal runtime work is currently foreground work.
    /// Queue replay is deliberately background work.
    ///
    /// This gives us the rule we need later:
    ///
    ///     foreground request waiting
    ///              ↓
    ///       service it first
    ///              ↓
    ///     replay another batch only when possible
    pub const fn priority(self) -> ModemRequestPriority {
        match self {
            Self::ReplayBatch => ModemRequestPriority::Replay,

            Self::GnssFix
            | Self::LiveTelemetry
            | Self::Heartbeat
            | Self::NetworkState
            | Self::Diagnostics => ModemRequestPriority::Foreground,
        }
    }
}

/// Command sent to the modem owner.
///
/// The modem owner will eventually receive these commands through Embassy
/// channels and will be the only runtime task allowed to use `&mut Modem`.
pub enum ModemRequest {
    /// Obtain the latest GNSS fix.
    GnssFix,

    /// Read the current SIM / LTE / packet-data state.
    NetworkState,

    /// Run the full modem/network diagnostics sequence.
    Diagnostics,

    /// Send the prepared heartbeat payload.
    Heartbeat,

    /// Send the newest live telemetry payload immediately.
    LiveTelemetry,
}

impl ModemRequest {
    /// Return the scheduling classification for this request.
    pub const fn kind(&self) -> ModemRequestKind {
        match self {
            Self::GnssFix => ModemRequestKind::GnssFix,
            Self::NetworkState => ModemRequestKind::NetworkState,
            Self::Diagnostics => ModemRequestKind::Diagnostics,
            Self::Heartbeat => ModemRequestKind::Heartbeat,
            Self::LiveTelemetry => ModemRequestKind::LiveTelemetry,
        }
    }

    /// Return whether this request belongs to foreground work or replay work.
    pub const fn priority(&self) -> ModemRequestPriority {
        self.kind().priority()
    }
}

/// Foreground requests waiting for access to the A7670 modem.
///
/// This channel is intentionally separate from future replay traffic.
///
/// Keeping foreground work in its own queue means that once one modem
/// transaction finishes, the modem owner can check this queue before
/// allowing another background replay batch to start.
pub static FOREGROUND_MODEM_REQUESTS: Channel<CriticalSectionRawMutex, ModemRequest, 4> =
    Channel::new();

/// Latest GNSS response produced by the modem owner.
///
/// Only the GNSS request path will wait on this signal.
pub static GNSS_RESPONSE: Signal<CriticalSectionRawMutex, Option<GpsInfo>> = Signal::new();

/// Latest network-state response produced by the modem owner.
///
/// Only the network-state request path will wait on this signal.
pub static NETWORK_STATE_RESPONSE: Signal<CriticalSectionRawMutex, NetworkState> = Signal::new();

/// Completion signal for a diagnostics request.
///
/// Diagnostics currently has no returned data; the caller only needs to know
/// when the modem owner has finished running the diagnostics sequence.
pub static DIAGNOSTICS_RESPONSE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Prepared heartbeat payload waiting for the modem owner.
///
/// The payload is kept outside `ModemRequest` so the foreground control
/// channel remains small.
pub static HEARTBEAT_PAYLOAD: Signal<CriticalSectionRawMutex, String<256>> = Signal::new();

/// Result of the most recent heartbeat HTTP transaction.
pub static HEARTBEAT_RESPONSE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

/// Prepared live telemetry payload waiting for the modem owner.
///
/// The payload is kept outside the control request channel so every channel
/// slot does not need to contain a 1024-byte telemetry buffer.
pub static LIVE_TELEMETRY_PAYLOAD: Signal<CriticalSectionRawMutex, String<1024>> = Signal::new();

/// Result of the most recent direct live telemetry HTTP transaction.
pub static LIVE_TELEMETRY_RESPONSE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

/// Wakes the modem owner whenever new foreground or replay work arrives.
///
/// The actual requests remain stored in their respective channels.
/// This signal only prevents the owner from sleeping while work is waiting.
pub static MODEM_WORK_AVAILABLE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Prepared replay batch payload waiting for the modem owner.
///
/// Replay payloads can be much larger than normal control messages, so the
/// 4096-byte JSON buffer is kept outside the control channel.
pub static REPLAY_PAYLOAD: Signal<CriticalSectionRawMutex, String<4096>> = Signal::new();

/// Background replay requests waiting for modem access.
///
/// This channel is intentionally separate from foreground modem requests so
/// the owner can always check foreground work before starting another replay
/// transaction.
pub static REPLAY_MODEM_REQUESTS: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

/// Result of the most recent replay-batch HTTP transaction.
pub static REPLAY_RESPONSE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

/// Execute one foreground modem request.
///
/// This function is the beginning of the single-owner boundary around the
/// A7670 modem. Eventually only the modem-owner Embassy task will call it.
///
/// Each modem operation runs to completion before another request is handled.
/// This prevents different tasks from interleaving AT commands on the same
/// UART-backed modem.
pub async fn execute_foreground_request(modem: &mut Modem<'_>, request: ModemRequest) {
    match request {
        ModemRequest::GnssFix => {
            let gps_fix = crate::drivers::gnss::get_live_fix(modem).await;

            GNSS_RESPONSE.signal(gps_fix);
        }

        ModemRequest::NetworkState => {
            let network_state = crate::network::state::read_network_state(modem).await;

            NETWORK_STATE_RESPONSE.signal(network_state);
        }

        ModemRequest::Diagnostics => {
            crate::network::diagnostics::run_network_diagnostics(modem).await;

            DIAGNOSTICS_RESPONSE.signal(());
        }

        ModemRequest::Heartbeat => {
            let payload = HEARTBEAT_PAYLOAD.wait().await;

            let succeeded = crate::network::http::send_heartbeat(modem, &payload).await;

            HEARTBEAT_RESPONSE.signal(succeeded);
        }

        ModemRequest::LiveTelemetry => {
            let payload = LIVE_TELEMETRY_PAYLOAD.wait().await;

            let succeeded = crate::network::http::send_payload(modem, &payload).await;

            LIVE_TELEMETRY_RESPONSE.signal(succeeded);
        }
    }
}

/// Run the foreground modem-owner loop.
///
/// This loop waits for requests from other Embassy tasks and executes
/// exactly one modem transaction at a time.
///
/// Only this owner path should eventually hold mutable runtime access
/// to the A7670 modem.
pub async fn run_foreground_modem_owner(modem: &mut Modem<'_>) -> ! {
    loop {
        /*
         * Foreground always gets the first opportunity to use the modem.
         *
         * We only begin a replay transaction when no foreground request
         * is already waiting.
         */
        if let Ok(request) = FOREGROUND_MODEM_REQUESTS.try_receive() {
            execute_foreground_request(modem, request).await;

            continue;
        }

        /*
         * Replay processes exactly one bounded batch per owner turn.
         *
         * Once that HTTP transaction finishes, the loop starts again
         * from the foreground check above.
         */
        if REPLAY_MODEM_REQUESTS.try_receive().is_ok() {
            execute_replay_request(modem).await;

            continue;
        }

        /*
         * Nothing is currently queued.
         *
         * Sleep cooperatively until one of the request paths tells us
         * that new work is available.
         */
        MODEM_WORK_AVAILABLE.wait().await;
    }
}

/// Embassy task that becomes the single runtime owner of the A7670 modem.
///
/// The modem is moved into this task rather than shared between tasks.
/// Therefore all runtime AT-command access can eventually be coordinated
/// through the request channels in this module.
#[embassy_executor::task]
pub async fn modem_owner_task(mut modem: Modem<'static>) {
    run_foreground_modem_owner(&mut modem).await;
}

/// Request a GNSS fix from the modem owner.
///
/// The caller does not access the modem directly. It places a request into
/// the foreground queue and waits for the GNSS-specific response signal.
pub async fn request_gnss_fix() -> Option<GpsInfo> {
    FOREGROUND_MODEM_REQUESTS.send(ModemRequest::GnssFix).await;

    MODEM_WORK_AVAILABLE.signal(());

    GNSS_RESPONSE.wait().await
}

/// Ask the modem owner to run the full network diagnostics sequence.
///
/// The caller waits until the modem owner has completed the diagnostics
/// transaction before continuing.
pub async fn request_diagnostics() {
    FOREGROUND_MODEM_REQUESTS
        .send(ModemRequest::Diagnostics)
        .await;

    MODEM_WORK_AVAILABLE.signal(());

    DIAGNOSTICS_RESPONSE.wait().await;
}

/// Ask the modem owner for the current network state.
///
/// The caller sends a small control request and waits for the owner to return
/// the SIM / registration / attachment / IP state.
pub async fn request_network_state() -> NetworkState {
    FOREGROUND_MODEM_REQUESTS
        .send(ModemRequest::NetworkState)
        .await;

    MODEM_WORK_AVAILABLE.signal(());

    NETWORK_STATE_RESPONSE.wait().await
}

/// Send a heartbeat through the modem owner.
///
/// The heartbeat payload is staged separately from the small control request
/// so the foreground request channel remains lightweight.
pub async fn request_heartbeat(payload: String<256>) -> bool {
    HEARTBEAT_PAYLOAD.signal(payload);

    FOREGROUND_MODEM_REQUESTS
        .send(ModemRequest::Heartbeat)
        .await;

    MODEM_WORK_AVAILABLE.signal(());

    HEARTBEAT_RESPONSE.wait().await
}

/// Upload the newest live telemetry payload through the modem owner.
///
/// The payload has already been constructed by the telemetry layer.
/// This function only coordinates access to the shared A7670 modem.
pub async fn request_live_telemetry(payload: String<1024>) -> bool {
    LIVE_TELEMETRY_PAYLOAD.signal(payload);

    FOREGROUND_MODEM_REQUESTS
        .send(ModemRequest::LiveTelemetry)
        .await;

    MODEM_WORK_AVAILABLE.signal(());

    LIVE_TELEMETRY_RESPONSE.wait().await
}

/// Upload one bounded telemetry replay batch through the modem owner.
///
/// Replay traffic uses a separate request path from foreground work.
///
/// This function does not decide which records belong in the batch and does
/// not modify ORBIQ.LOG. Those responsibilities remain in the telemetry
/// replay/publisher layer.
pub async fn request_replay_batch(payload: String<4096>) -> bool {
    REPLAY_PAYLOAD.signal(payload);

    REPLAY_MODEM_REQUESTS.send(()).await;

    MODEM_WORK_AVAILABLE.signal(());

    REPLAY_RESPONSE.wait().await
}

/// Execute one background replay HTTP transaction.
///
/// Replay is deliberately limited to one already-prepared batch.
/// After this transaction finishes, control returns to the owner loop,
/// which checks foreground work again before allowing another replay batch.
async fn execute_replay_request(modem: &mut Modem<'_>) {
    let payload = REPLAY_PAYLOAD.wait().await;

    let succeeded = crate::network::http::send_payload(modem, &payload).await;

    REPLAY_RESPONSE.signal(succeeded);
}

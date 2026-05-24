import { useDeviceHealthStore } from "../../store/deviceHealthStore";

function formatTime(timestamp: string | null | undefined) {
  if (!timestamp) {
    return "N/A";
  }

  const date = new Date(timestamp);

  if (Number.isNaN(date.getTime())) {
    return "N/A";
  }

  return new Intl.DateTimeFormat("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(date);
}

export function DeviceHealthPanel() {
  const { events, onlineCount, staleCount, offlineCount, unknownCount } =
    useDeviceHealthStore();

  return (
    <section className="device-health-panel">
      <div className="device-health-panel__header">
        <div>
          <h2>Device Health</h2>

          <p>Live operational device connectivity monitoring.</p>
        </div>
      </div>

      <div className="device-health-summary">
        <div className="device-health-summary__card device-health-summary__card--online">
          <label>ONLINE</label>

          <strong>{onlineCount}</strong>
        </div>

        <div className="device-health-summary__card device-health-summary__card--stale">
          <label>STALE</label>

          <strong>{staleCount}</strong>
        </div>

        <div className="device-health-summary__card device-health-summary__card--offline">
          <label>OFFLINE</label>

          <strong>{offlineCount}</strong>
        </div>

        <div className="device-health-summary__card device-health-summary__card--unknown">
          <label>UNKNOWN</label>

          <strong>{unknownCount}</strong>
        </div>
      </div>

      <div className="device-health-events">
        {events.slice(0, 8).map((event) => (
          <article key={event.id} className="device-health-event">
            <div className="device-health-event__top">
              <strong>{event.device_id.slice(0, 8)}</strong>

              <span>{formatTime(event.detected_at)}</span>
            </div>

            <div className="device-health-event__middle">
              <span
                className={`device-health-badge device-health-badge--${event.new_status.toLowerCase()}`}
              >
                {event.new_status}
              </span>

              {event.previous_status && (
                <small>from {event.previous_status}</small>
              )}
            </div>

            <p>{event.reason}</p>
          </article>
        ))}

        {events.length === 0 && (
          <div className="device-health-events__empty">
            No device health events available.
          </div>
        )}
      </div>
    </section>
  );
}

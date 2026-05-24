import { useState } from "react";
import { useTelemetryStore } from "../../store/telemetryStore";

function formatTime(timestamp: string) {
  return new Intl.DateTimeFormat("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(new Date(timestamp));
}

export function TelemetryStreamPanel() {
  const readings = useTelemetryStore((state) => state.readings);
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <section className="telemetry-panel">
      <div className="telemetry-panel__header">
        <div>
          <h2>Live Telemetry Stream</h2>

          <p>Read-only operational telemetry feed refreshed every 5 seconds.</p>
        </div>

        <div className="telemetry-panel__header-actions">
          <span className="telemetry-panel__count">
            {readings.length} readings
          </span>

          <button
            type="button"
            className="telemetry-panel__toggle"
            onClick={() => setIsExpanded((current) => !current)}
          >
            {isExpanded ? "Collapse" : "Expand"}
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className="telemetry-stream">
          {readings.map((reading, index) => (
            <article
              key={`${reading.device_id}-${index}`}
              className="telemetry-stream__item"
            >
              <div className="telemetry-stream__top">
                <strong>{reading.device_id.slice(0, 8)}</strong>

                <span>{formatTime(reading.received_at)}</span>
              </div>

              <div className="telemetry-stream__metrics">
                <div>
                  <label>Fuel</label>

                  <strong>{reading.fuel_level_litres.toFixed(1)}L</strong>
                </div>

                <div>
                  <label>Vibration</label>

                  <strong>
                    {reading.vibration_level?.toFixed(2) ?? "N/A"}
                  </strong>
                </div>

                <div>
                  <label>Motion</label>

                  <strong>{reading.motion_detected ? "ACTIVE" : "IDLE"}</strong>
                </div>
              </div>

              <div className="telemetry-stream__gps">
                GPS: {reading.latitude?.toFixed(5)},{" "}
                {reading.longitude?.toFixed(5)}
              </div>
            </article>
          ))}

          {readings.length === 0 && (
            <div className="telemetry-stream__empty">
              Waiting for telemetry...
            </div>
          )}
        </div>
      )}
    </section>
  );
}

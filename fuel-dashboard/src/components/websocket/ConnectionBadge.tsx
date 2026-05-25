import { useConnectionStore } from "../../store/connectionStore";

export function ConnectionBadge() {
  const status = useConnectionStore((state) => state.status);
  const lastHeartbeatAt = useConnectionStore((state) => state.lastHeartbeatAt);

  const statusLabel =
    status === "connected"
      ? "BACKEND ONLINE"
      : status === "connecting"
        ? "BACKEND CONNECTING"
        : "BACKEND OFFLINE";

  return (
    <div className={`connection-badge connection-badge--${status}`}>
      <span className="connection-badge__dot" />

      <div>
        <strong>{statusLabel}</strong>

        <span>
          {lastHeartbeatAt
            ? `Last heartbeat ${new Date(lastHeartbeatAt).toLocaleTimeString()}`
            : "Waiting for heartbeat"}
        </span>
      </div>
    </div>
  );
}

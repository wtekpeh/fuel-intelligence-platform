import { useAlertStore } from "../../store/alertStore";
import { useConnectionStore } from "../../store/connectionStore";
import { StatusCard } from "./StatusCard";

export function StatusGrid() {
  const alerts = useAlertStore((state) => state.alerts);
  const connectionStatus = useConnectionStore((state) => state.status);

  const openAlerts = alerts.filter((alert) => alert.status === "OPEN").length;

  const criticalAlerts = alerts.filter(
    (alert) => alert.severity === "Critical" && alert.status !== "RESOLVED",
  ).length;

  const resolvedAlerts = alerts.filter(
    (alert) => alert.status === "RESOLVED",
  ).length;

  const connectionTone =
    connectionStatus === "connected"
      ? "good"
      : connectionStatus === "connecting"
        ? "warning"
        : "danger";

  return (
    <section className="status-grid">
      <StatusCard
        label="Connection"
        value={connectionStatus}
        hint="Live alert stream"
        tone={connectionTone}
      />

      <StatusCard
        label="Open Alerts"
        value={openAlerts}
        hint="Requires action"
        tone={openAlerts > 0 ? "warning" : "good"}
      />

      <StatusCard
        label="Critical"
        value={criticalAlerts}
        hint="Unresolved critical alerts"
        tone={criticalAlerts > 0 ? "danger" : "good"}
      />

      <StatusCard
        label="Resolved"
        value={resolvedAlerts}
        hint="Closed incidents"
        tone="neutral"
      />
    </section>
  );
}

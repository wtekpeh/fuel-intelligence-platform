import { useAlertStore } from "../../store/alertStore";
import { StatusCard } from "./StatusCard";
import { useFleetStore } from "../../store/fleetStore";

export function StatusGrid() {
  const alerts = useAlertStore((state) => state.alerts);
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const openAlerts = alerts.filter((alert) => alert.status === "OPEN").length;

  const criticalAlerts = alerts.filter(
    (alert) => alert.severity === "Critical" && alert.status !== "RESOLVED",
  ).length;

  const resolvedAlerts = alerts.filter(
    (alert) => alert.status === "RESOLVED",
  ).length;

  return (
    <section className="status-grid">
      <StatusCard
        label="Device Status"
        value={selectedDevice?.device_status ?? "UNKNOWN"}
        hint={selectedDevice?.device_code ?? "No device selected"}
        tone={
          selectedDevice?.device_status === "ONLINE"
            ? "good"
            : selectedDevice?.device_status === "STALE"
              ? "warning"
              : "danger"
        }
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

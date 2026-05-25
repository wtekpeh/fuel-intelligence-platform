import { useAlertsController } from "../services/useAlertsController";
import { StatusGrid } from "../components/status/StatusGrid";
import { AlertTable } from "../components/alerts/AlertTable";
import { AlertDetailsPanel } from "../components/alerts/AlertDetailsPanel";
import { ConnectionBadge } from "../components/websocket/ConnectionBadge";
import { useTelemetryPolling } from "../services/useTelemetryPolling";
import { TelemetryStreamPanel } from "../components/telemetry/TelemetryStreamPanel";
import { useDeviceHealthPolling } from "../services/useDeviceHealthPolling";
import { DeviceHealthPanel } from "../components/device-health/DeviceHealthPanel";
import { DashboardTabs } from "../components/layout/DashboardTabs";
import { useDashboardSectionStore } from "../store/dashboardSectionStore";
import { SelectedDeviceStrip } from "../components/fleet/SelectedDeviceStrip";

export function DashboardPage() {
  const activeSection = useDashboardSectionStore(
    (state) => state.activeSection,
  );

  useAlertsController();
  useTelemetryPolling();
  useDeviceHealthPolling();

  return (
    <main className="dashboard-page">
      <section className="dashboard-header">
        <div>
          <p className="dashboard-eyebrow">Fuel Intelligence Platform</p>

          <h1 className="dashboard-title">Live Operations Dashboard</h1>

          <p className="dashboard-subtitle">
            Real-time fuel alerts, incident lifecycle tracking, and operational
            monitoring.
          </p>
        </div>

        <ConnectionBadge />
      </section>
      <DashboardTabs />
      <SelectedDeviceStrip />
      <StatusGrid />

      {activeSection === "operations" && (
        <>
          <TelemetryStreamPanel />

          <section className="dashboard-main-grid">
            <AlertTable />
            <AlertDetailsPanel />
          </section>
        </>
      )}

      {activeSection === "device-health" && <DeviceHealthPanel />}

      {activeSection === "investigation" && (
        <section className="placeholder-panel">
          <h2>Investigation</h2>
          <p>Timeline replay and forensic review will be added here.</p>
        </section>
      )}

      {activeSection === "analytics" && (
        <section className="placeholder-panel">
          <h2>Analytics</h2>
          <p>
            Trend charts and operational intelligence summaries will be added
            here.
          </p>
        </section>
      )}
    </main>
  );
}

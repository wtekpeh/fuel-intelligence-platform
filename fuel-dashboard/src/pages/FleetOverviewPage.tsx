import { useFleetOverview } from "../services/useFleetOverview";
import { useFleetStore } from "../store/fleetStore";
import { useOrganizationStore } from "../store/organizationStore";
import { useAppViewStore } from "../store/appViewStore";
import FuelLevelGauge from "../components/shared/FuelLevelGauge";
import { useTelemetryStore } from "../store/telemetryStore";

import "../styles/fleet.css";

export function FleetOverviewPage() {
  const selectedOrganization = useOrganizationStore(
    (state) => state.selectedOrganization,
  );

  const selectDevice = useFleetStore((state) => state.selectDevice);

  const showDashboard = useAppViewStore((state) => state.showDashboard);

  const fleetItems = useFleetStore((state) => state.fleetItems);

  const offlineDevices = fleetItems.filter(
    (item) => item.device_status === "OFFLINE",
  );

  const staleDevices = fleetItems.filter(
    (item) => item.device_status === "STALE",
  );

  const devicesWithAlerts = fleetItems.filter(
    (item) => item.open_alert_count > 0,
  );

  const totalOpenAlerts = fleetItems.reduce(
    (totalAlerts, item) => totalAlerts + item.open_alert_count,
    0,
  );

  const fleetRiskLevel =
    offlineDevices.length > 0 || totalOpenAlerts >= 5
      ? "HIGH"
      : staleDevices.length > 0 || totalOpenAlerts > 0
        ? "MODERATE"
        : "HEALTHY";

  const readings = useTelemetryStore((state) => state.readings);

  useFleetOverview(selectedOrganization?.organization_id ?? null);

  return (
    <main className="fleet-page">
      <section className="fleet-header">
        <p className="fleet-eyebrow">Fleet Overview</p>

        <h1>
          {selectedOrganization?.organization_name ?? "Selected Organization"}
        </h1>

        <p>
          Review assets, connected devices, sensors, and operational status
          before opening the live dashboard.
        </p>
      </section>

      <section className="fleet-attention-center">
        <div className="fleet-attention-card">
          <label>Offline Devices</label>
          <strong>{offlineDevices.length}</strong>
        </div>

        <div className="fleet-attention-card">
          <label>Stale Devices</label>
          <strong>{staleDevices.length}</strong>
        </div>

        <div className="fleet-attention-card">
          <label>Devices With Alerts</label>
          <strong>{devicesWithAlerts.length}</strong>
        </div>

        <div className="fleet-attention-card">
          <label>Total Open Alerts</label>
          <strong>{totalOpenAlerts}</strong>
        </div>
      </section>

      <section
        className={`fleet-status-banner fleet-status-banner--${fleetRiskLevel.toLowerCase()}`}
      >
        <label>Fleet Status</label>

        <strong>
          {fleetRiskLevel === "HIGH"
            ? "Operational Risk Elevated"
            : fleetRiskLevel === "MODERATE"
              ? "Moderate Attention Required"
              : "Fleet Healthy"}
        </strong>

        <span>
          {offlineDevices.length} offline device(s), {staleDevices.length} stale
          device(s), {devicesWithAlerts.length} device(s) with alerts,{" "}
          {totalOpenAlerts} open alert(s).
        </span>
      </section>

      <section className="fleet-grid">
        {fleetItems.map((item) => (
          <article key={item.device_id} className="fleet-card">
            <div className="fleet-card__top">
              <div>
                <p>{item.asset_type}</p>
                <h2>{item.asset_name}</h2>
              </div>

              <span
                className={`fleet-status fleet-status--${(item.device_status ?? "unassigned").toLowerCase()}`}
              >
                {item.device_status ?? "UNASSIGNED"}
              </span>
            </div>

            <div className="fleet-card__meta">
              <div>
                <label>Device</label>
                <strong>{item.device_code}</strong>
              </div>

              <div>
                <label>Capacity</label>
                <strong>
                  {item.capacity_litres ? `${item.capacity_litres}L` : "N/A"}
                </strong>
              </div>

              <div>
                <label>Sensors</label>
                <strong>{item.sensor_count}</strong>
              </div>

              <div>
                <label>Open Alerts</label>
                <strong>{item.open_alert_count}</strong>
              </div>
            </div>

            <div className="fleet-card__sensors">
              {item.sensor_types.map((sensorType) => (
                <span key={sensorType}>{sensorType}</span>
              ))}
            </div>

            {(() => {
              const latestReading = readings.find(
                (reading) => reading.device_id === item.device_id,
              );

              if (!latestReading) {
                return (
                  <div className="fleet-card__fuel-empty">
                    <label>Fuel Gauge</label>
                    <strong>No live fuel reading</strong>
                  </div>
                );
              }

              return (
                <FuelLevelGauge
                  value={latestReading.fuel_level_litres}
                  maxValue={item.capacity_litres ?? 100}
                  size="compact"
                  label="Fuel"
                />
              );
            })()}

            <button
              type="button"
              className="fleet-card__button"
              onClick={() => {
                selectDevice(item);
                showDashboard();
              }}
            >
              Open Device Dashboard
            </button>
          </article>
        ))}
      </section>
    </main>
  );
}

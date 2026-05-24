import { acknowledgeAlert, resolveAlert } from "../../api/alertsApi";
import { useAlertStore } from "../../store/alertStore";
import type { AlertResponse } from "../../types";
import { MobileAlertCard } from "./MobileAlertCard";

function formatAlertTime(createdAt: string) {
  return new Intl.DateTimeFormat("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    day: "2-digit",
    month: "short",
  }).format(new Date(createdAt));
}

function getSeverityClass(severity: AlertResponse["severity"]) {
  if (severity === "Critical") {
    return "alert-pill alert-pill--danger";
  }

  if (severity === "Warning") {
    return "alert-pill alert-pill--warning";
  }

  return "alert-pill alert-pill--neutral";
}

function getStatusClass(status: AlertResponse["status"]) {
  if (status === "OPEN") {
    return "alert-pill alert-pill--open";
  }

  if (status === "ACKNOWLEDGED") {
    return "alert-pill alert-pill--acknowledged";
  }

  return "alert-pill alert-pill--resolved";
}

export function AlertTable() {
  const alerts = useAlertStore((state) => state.alerts);
  const updateAlertLifecycle = useAlertStore(
    (state) => state.updateAlertLifecycle,
  );

  const setSelectedAlert = useAlertStore((state) => state.setSelectedAlert);

  const selectedAlert = useAlertStore((state) => state.selectedAlert);

  async function handleAcknowledge(alertId: string) {
    const updatedAlert = await acknowledgeAlert(alertId);
    updateAlertLifecycle(updatedAlert);
  }

  async function handleResolve(alertId: string) {
    const updatedAlert = await resolveAlert(alertId);
    updateAlertLifecycle(updatedAlert);
  }

  return (
    <section className="alert-panel">
      <div className="alert-panel__header">
        <div>
          <h2>Operational Alerts</h2>
          <p>Live incident feed with acknowledgment and resolution workflow.</p>
        </div>

        <span className="alert-panel__count">{alerts.length} alerts</span>
      </div>

      <div className="mobile-alert-list">
        {alerts.map((alert) => (
          <MobileAlertCard key={alert.id} alert={alert} />
        ))}
      </div>

      <div className="alert-table-wrapper">
        <table className="alert-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Type</th>
              <th>Severity</th>
              <th>Status</th>
              <th>Reason</th>
              <th>Actions</th>
            </tr>
          </thead>

          <tbody>
            {alerts.map((alert) => (
              <tr
                key={alert.id}
                onClick={() => setSelectedAlert(alert)}
                className={[
                  "alert-table__row",
                  alert.status === "OPEN"
                    ? "alert-table__row--needs-attention"
                    : "",
                  selectedAlert?.id === alert.id
                    ? "alert-table__row--selected"
                    : "",
                ].join(" ")}
              >
                <td>{formatAlertTime(alert.created_at)}</td>

                <td>
                  <strong>{alert.alert_type}</strong>
                </td>

                <td>
                  <span className={getSeverityClass(alert.severity)}>
                    {alert.severity}
                  </span>
                </td>

                <td>
                  <span className={getStatusClass(alert.status)}>
                    {alert.status}
                  </span>
                </td>

                <td className="alert-table__reason">{alert.reason}</td>

                <td>
                  <div className="alert-actions">
                    <button
                      type="button"
                      onClick={(event) => {
                        event.stopPropagation();
                        handleAcknowledge(alert.id);
                      }}
                      disabled={alert.status !== "OPEN"}
                    >
                      Acknowledge
                    </button>

                    <button
                      type="button"
                      onClick={(event) => {
                        event.stopPropagation();
                        handleResolve(alert.id);
                      }}
                      disabled={alert.status === "RESOLVED"}
                    >
                      Resolve
                    </button>
                  </div>
                </td>
              </tr>
            ))}

            {alerts.length === 0 && (
              <tr>
                <td colSpan={6} className="alert-table__empty">
                  No alerts available yet.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

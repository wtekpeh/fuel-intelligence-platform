import { acknowledgeAlert, resolveAlert } from "../../api/alertsApi";
import { useAlertStore } from "../../store/alertStore";
import { useDashboardSectionStore } from "../../store/dashboardSectionStore";
import { useInvestigationStore } from "../../store/investigationStore";

export function AlertDetailsPanel() {
  const selectedAlert = useAlertStore((state) => state.selectedAlert);

  const updateAlertLifecycle = useAlertStore(
    (state) => state.updateAlertLifecycle,
  );

  const setSelectedAlert = useAlertStore((state) => state.setSelectedAlert);

  const setActiveSection = useDashboardSectionStore(
    (state) => state.setActiveSection,
  );

  const setFocusedFuelEventId = useInvestigationStore(
    (state) => state.setFocusedFuelEventId,
  );

  if (!selectedAlert) {
    return (
      <aside className="alert-details-panel alert-details-panel--empty">
        <p>Select an alert to inspect operational details.</p>
      </aside>
    );
  }

  const activeAlert = selectedAlert;

  async function handleAcknowledge() {
    const updatedAlert = await acknowledgeAlert(activeAlert.id);

    updateAlertLifecycle(updatedAlert);
  }

  async function handleResolve() {
    const updatedAlert = await resolveAlert(activeAlert.id);

    updateAlertLifecycle(updatedAlert);
  }

  return (
    <aside className="alert-details-panel">
      <div className="alert-details-panel__header">
        <div>
          <p className="alert-details-panel__eyebrow">Incident Details</p>

          <h2>{selectedAlert.alert_type}</h2>
        </div>

        <span
          className={`alert-pill ${
            selectedAlert.status === "RESOLVED"
              ? "alert-pill--resolved"
              : selectedAlert.status === "ACKNOWLEDGED"
                ? "alert-pill--acknowledged"
                : "alert-pill--open"
          }`}
        >
          {selectedAlert.status}
        </span>

        <button
          type="button"
          className="alert-details-panel__close"
          onClick={() => setSelectedAlert(null)}
        >
          ×
        </button>
      </div>

      <div className="alert-details-section">
        <label>Severity</label>
        <strong>{selectedAlert.severity}</strong>
      </div>

      <div className="alert-details-section">
        <label>Created</label>
        <strong>{new Date(selectedAlert.created_at).toLocaleString()}</strong>
      </div>

      <div className="alert-details-section">
        <label>Alert ID</label>
        <code>{selectedAlert.id}</code>
      </div>

      <div className="alert-details-section">
        <label>Fuel Event ID</label>
        <code>{selectedAlert.fuel_event_id ?? "N/A"}</code>
      </div>

      <div className="alert-details-section">
        <label>Reason</label>

        <p>{selectedAlert.reason}</p>
      </div>

      <div className="alert-details-actions">
        <button
          type="button"
          onClick={handleAcknowledge}
          disabled={selectedAlert.status !== "OPEN"}
        >
          Acknowledge
        </button>

        <button
          type="button"
          onClick={handleResolve}
          disabled={selectedAlert.status === "RESOLVED"}
        >
          Resolve
        </button>

        <button
          type="button"
          className="alert-details__secondary-button"
          onClick={() => {
            setFocusedFuelEventId(selectedAlert.fuel_event_id);
            setActiveSection("investigation");
          }}
        >
          View Investigation
        </button>
      </div>
    </aside>
  );
}

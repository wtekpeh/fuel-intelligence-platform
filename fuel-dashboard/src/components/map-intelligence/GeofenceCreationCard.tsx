import { useState } from "react";

import { useGeofenceDrawStore } from "../../store/geofenceDrawStore";
import { useGeofenceStore } from "../../store/geofenceStore";
import { useOrganizationStore } from "../../store/organizationStore";

import type { GeofenceType } from "../../types";

const geofenceTypes: GeofenceType[] = [
  "DEPOT",
  "FUELING_STATION",
  "RESTRICTED_ZONE",
  "SAFE_CORRIDOR",
  "CUSTOMER_SITE",
];

function GeofenceCreationCard() {
  const [name, setName] = useState("");
  const [geofenceType, setGeofenceType] = useState<GeofenceType>("DEPOT");
  const [feedbackMessage, setFeedbackMessage] = useState<string | null>(null);
  const [feedbackType, setFeedbackType] = useState<"success" | "error" | null>(
    null,
  );

  const drawnGeojson = useGeofenceDrawStore((state) => state.drawnGeojson);

  const clearDrawnGeojson = useGeofenceDrawStore(
    (state) => state.clearDrawnGeojson,
  );

  const selectedOrganization = useOrganizationStore(
    (state) => state.selectedOrganization,
  );

  const createGeofenceRecord = useGeofenceStore(
    (state) => state.createGeofenceRecord,
  );

  if (!drawnGeojson) {
    return null;
  }

  async function handleSave() {
    if (!selectedOrganization || !drawnGeojson || !name.trim()) {
      return;
    }

    try {
      await createGeofenceRecord({
        organization_id: selectedOrganization.organization_id,
        name: name.trim(),
        geofence_type: geofenceType,
        geojson: drawnGeojson,
      });

      setFeedbackType("success");
      setFeedbackMessage("Geofence saved successfully.");

      setTimeout(() => {
        setName("");
        setGeofenceType("DEPOT");
        clearDrawnGeojson();
        setFeedbackMessage(null);
        setFeedbackType(null);
      }, 900);
    } catch (error) {
      console.error("Failed to save geofence", error);

      setFeedbackType("error");
      setFeedbackMessage("Failed to save geofence. Please try again.");
    }
  }

  function handleCancel() {
    setName("");
    setGeofenceType("DEPOT");
    clearDrawnGeojson();
  }

  return (
    <div className="geofence-modal-overlay">
      <div className="geofence-modal">
        <div className="geofence-modal__header">
          <h3>Save Geofence</h3>

          <button
            type="button"
            className="geofence-modal__close"
            onClick={handleCancel}
          >
            ×
          </button>
        </div>

        <p className="geofence-modal__subtitle">
          Save the drawn operational zone.
        </p>

        <input
          type="text"
          value={name}
          placeholder="Geofence name"
          onChange={(event) => setName(event.target.value)}
        />

        <select
          value={geofenceType}
          onChange={(event) =>
            setGeofenceType(event.target.value as GeofenceType)
          }
        >
          {geofenceTypes.map((type) => (
            <option key={type} value={type}>
              {type.replaceAll("_", " ")}
            </option>
          ))}
        </select>

        {feedbackMessage && (
          <div
            className={`geofence-modal__feedback geofence-modal__feedback--${feedbackType}`}
          >
            {feedbackMessage}
          </div>
        )}

        <div className="geofence-creation-card__actions">
          <button
            type="button"
            className="map-action-button"
            onClick={handleSave}
            disabled={!name.trim()}
          >
            Save Geofence
          </button>

          <button
            type="button"
            className="map-secondary-button"
            onClick={handleCancel}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

export default GeofenceCreationCard;

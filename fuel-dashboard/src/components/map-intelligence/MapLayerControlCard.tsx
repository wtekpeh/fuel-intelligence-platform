import { useState } from "react";
import { useMapLayerStore } from "../../store/mapLayerStore";

export default function MapLayerControlCard() {
  const [isMobileOpen, setIsMobileOpen] = useState(false);

  const {
    showDeviceMarker,
    showInvestigationEvents,
    showFuelEvents,
    showGeofenceTransitions,
    showGeofences,
    showReplayRoute,
    showHotspots,

    toggleDeviceMarker,
    toggleInvestigationEvents,
    toggleFuelEvents,
    toggleGeofenceTransitions,
    toggleGeofences,
    toggleReplayRoute,
    toggleHotspots,
  } = useMapLayerStore();

  const layerContent = (
    <>
      <label>
        <input
          type="checkbox"
          checked={showDeviceMarker}
          onChange={toggleDeviceMarker}
        />
        Device Marker
      </label>

      <label>
        <input
          type="checkbox"
          checked={showInvestigationEvents}
          onChange={toggleInvestigationEvents}
        />
        Investigation Events
      </label>

      <label>
        <input
          type="checkbox"
          checked={showFuelEvents}
          onChange={toggleFuelEvents}
        />
        Fuel Events
      </label>

      <label>
        <input
          type="checkbox"
          checked={showGeofenceTransitions}
          onChange={toggleGeofenceTransitions}
        />
        Geofence Transitions
      </label>

      <label>
        <input
          type="checkbox"
          checked={showGeofences}
          onChange={toggleGeofences}
        />
        Geofences
      </label>

      <label>
        <input
          type="checkbox"
          checked={showReplayRoute}
          onChange={toggleReplayRoute}
        />
        Replay Route
      </label>

      <label>
        <input
          type="checkbox"
          checked={showHotspots}
          onChange={toggleHotspots}
        />
        Hotspots
      </label>
    </>
  );

  return (
    <>
      <div className="map-layer-control-card map-layer-control-card--desktop">
        <h3>Layer Controls</h3>

        {layerContent}
      </div>

      <button
        type="button"
        className="map-layer-control-mobile-trigger"
        onClick={() => setIsMobileOpen(true)}
      >
        Layers ⚙
      </button>

      {isMobileOpen && (
        <div className="map-layer-control-mobile-sheet">
          <div className="map-layer-control-mobile-sheet__content">
            <div className="map-layer-control-mobile-sheet__header">
              <h3>Layer Controls</h3>

              <button type="button" onClick={() => setIsMobileOpen(false)}>
                Close
              </button>
            </div>

            {layerContent}
          </div>
        </div>
      )}
    </>
  );
}

import OperationalMap from "./OperationalMap";

function MapIntelligencePanel() {
  return (
    <section className="map-intelligence-panel">
      <div className="map-intelligence-panel__header">
        <h2>Map Intelligence</h2>
        <p>
          Spatial operational view for selected device telemetry, investigation
          events, and future route replay intelligence.
        </p>
      </div>

      <OperationalMap />
    </section>
  );
}

export default MapIntelligencePanel;

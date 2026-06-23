import { useEffect } from "react";
import { useAnalyticsStore } from "../../store/analyticsStore";

export default function GeofenceUtilizationPanel() {
  const {
    geofenceUtilization,
    loadingGeofenceUtilization,
    geofenceUtilizationError,
    fetchGeofenceUtilization,
    selectedDays,
  } = useAnalyticsStore();

  useEffect(() => {
    fetchGeofenceUtilization(selectedDays);
  }, [fetchGeofenceUtilization, selectedDays]);

  if (loadingGeofenceUtilization) {
    return <p>Loading geofence utilization...</p>;
  }

  if (geofenceUtilizationError) {
    return <p>{geofenceUtilizationError}</p>;
  }

  const zones = geofenceUtilization?.zones ?? [];

  return (
    <section className="fleet-card">
      <h2>Geofence Utilization</h2>

      <div className="table-scroll">
        <table className="analytics-table">
          <thead>
            <tr>
              <th>Zone</th>
              <th>Visits</th>
            </tr>
          </thead>

          <tbody>
            {zones.map((zone) => (
              <tr key={zone.geofence_name}>
                <td>{zone.geofence_name}</td>
                <td>{zone.visits}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

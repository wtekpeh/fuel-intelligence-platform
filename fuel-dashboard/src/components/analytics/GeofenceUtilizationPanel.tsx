import { useGeofenceStore } from "../../store/geofenceStore";

export default function GeofenceUtilizationPanel() {
  const { transitionEvents } = useGeofenceStore();

  const zoneUsageRows = Object.values(
    transitionEvents.reduce<
      Record<string, { zoneName: string; visits: number }>
    >((zoneTotals, event) => {
      if (event.transition_type !== "ENTERED_ZONE") {
        return zoneTotals;
      }

      if (!zoneTotals[event.geofence_name]) {
        zoneTotals[event.geofence_name] = {
          zoneName: event.geofence_name,
          visits: 0,
        };
      }

      zoneTotals[event.geofence_name].visits += 1;

      return zoneTotals;
    }, {}),
  )
    .sort((firstZone, secondZone) => {
      return secondZone.visits - firstZone.visits;
    })
    .slice(0, 5);

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
            {zoneUsageRows.map((zone) => (
              <tr key={zone.zoneName}>
                <td>{zone.zoneName}</td>
                <td>{zone.visits}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

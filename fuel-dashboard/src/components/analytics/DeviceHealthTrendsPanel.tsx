import { useEffect } from "react";
import { useAnalyticsStore } from "../../store/analyticsStore";

export default function DeviceHealthTrendsPanel() {
  const {
    deviceHealthTrends,
    loadingDeviceHealthTrends,
    deviceHealthTrendsError,
    fetchDeviceHealthTrends,
    selectedDays,
  } = useAnalyticsStore();

  useEffect(() => {
    fetchDeviceHealthTrends(selectedDays);
  }, [fetchDeviceHealthTrends, selectedDays]);

  if (loadingDeviceHealthTrends) {
    return <p>Loading device health analytics...</p>;
  }

  if (deviceHealthTrendsError) {
    return <p>{deviceHealthTrendsError}</p>;
  }

  const unreliableDevices = deviceHealthTrends?.devices ?? [];

  return (
    <section className="fleet-card">
      <h2>Most Unreliable Devices</h2>

      <div className="table-scroll">
        <table className="analytics-table">
          <thead>
            <tr>
              <th>Device</th>
              <th>Health Events</th>
            </tr>
          </thead>

          <tbody>
            {unreliableDevices.map((device) => (
              <tr key={device.device_id}>
                <td className="analytics-device-cell">
                  <strong>{device.device_code}</strong>
                </td>
                <td>{device.reliability_issue_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

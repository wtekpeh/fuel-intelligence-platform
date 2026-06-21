import { useDeviceHealthStore } from "../../store/deviceHealthStore";
import { useFleetStore } from "../../store/fleetStore";

export default function DeviceHealthTrendsPanel() {
  const { events } = useDeviceHealthStore();
  const { fleetItems } = useFleetStore();

  const unreliableDevices = Object.values(
    events.reduce<
      Record<
        string,
        {
          deviceId: string;
          deviceCode: string;
          assetName: string;
          eventCount: number;
        }
      >
    >((deviceTotals, event) => {
      const matchingFleetItem = fleetItems.find((fleetItem) => {
        return fleetItem.device_id === event.device_id;
      });

      if (!deviceTotals[event.device_id]) {
        deviceTotals[event.device_id] = {
          deviceId: event.device_id,
          deviceCode: matchingFleetItem?.device_code ?? event.device_id,
          assetName: matchingFleetItem?.asset_name ?? "Unknown Asset",
          eventCount: 0,
        };
      }

      deviceTotals[event.device_id].eventCount += 1;

      return deviceTotals;
    }, {}),
  )
    .sort((firstDevice, secondDevice) => {
      return secondDevice.eventCount - firstDevice.eventCount;
    })
    .slice(0, 5);

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
              <tr key={device.deviceId}>
                <td className="analytics-device-cell">
                  <strong>{device.deviceCode}</strong>
                  <span>{device.assetName}</span>
                </td>
                <td>{device.eventCount}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

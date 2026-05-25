import { useFleetStore } from "../../store/fleetStore";

export function SelectedDeviceStrip() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  if (!selectedDevice) {
    return null;
  }

  return (
    <section className="selected-device-strip">
      <div>
        <p className="selected-device-strip__eyebrow">Selected Device</p>

        <h2>{selectedDevice.device_code}</h2>

        <span>{selectedDevice.asset_name}</span>
      </div>

      <div className="selected-device-strip__meta">
        <span
          className={`fleet-status fleet-status--${selectedDevice.device_status.toLowerCase()}`}
        >
          {selectedDevice.device_status}
        </span>

        <span>{selectedDevice.sensor_count} sensor(s)</span>

        <span>{selectedDevice.open_alert_count} open alert(s)</span>
      </div>
    </section>
  );
}

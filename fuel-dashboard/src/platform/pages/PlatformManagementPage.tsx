import { useEffect, useState } from "react";
import { usePlatformStore } from "../store/platformStore";
import "../styles/platform.css";

export default function PlatformManagementPage() {
  const {
    hardwareProfiles,
    selectedHardwareProfile,
    hardwareProfileSensors,

    devices,
    selectedDevice,
    deviceSensors,

    loadHardwareProfiles,
    selectHardwareProfile,
    loadDevices,
    selectDevice,
  } = usePlatformStore();

  const [mobileProfileOpen, setMobileProfileOpen] = useState(false);
  const [mobileDeviceOpen, setMobileDeviceOpen] = useState(false);

  useEffect(() => {
    loadHardwareProfiles();
    loadDevices();
  }, [loadHardwareProfiles, loadDevices]);

  const selectedProfile =
    selectedHardwareProfile ?? hardwareProfiles[0] ?? null;

  const filteredDevices = selectedProfile
    ? devices.filter(
        (device) => device.hardware_profile_id === selectedProfile.id,
      )
    : devices;

  const activeDevice = selectedDevice ?? filteredDevices[0] ?? null;

  const totalProvisionedSensors = devices.length * 0 + deviceSensors.length;

  const overviewCards = [
    {
      label: "Hardware Profiles",
      value: hardwareProfiles.length.toString(),
      hint: "Supported device kits",
    },
    {
      label: "Registered Devices",
      value: devices.length.toString(),
      hint: "Provisioned devices",
    },
    {
      label: "Provisioned Sensors",
      value: totalProvisionedSensors.toString(),
      hint: "Loaded from selected device",
    },
    {
      label: "Assets",
      value: "-",
      hint: "Linked operational assets",
    },
  ];

  return (
    <main className="platform-page">
      <header className="platform-header">
        <p className="platform-eyebrow">Platform Management</p>
        <h1>Device & Hardware Management</h1>
        <p>
          Register devices, assign hardware profiles, and verify automatically
          provisioned sensors before telemetry ingestion begins.
        </p>
      </header>

      <section className="platform-overview-grid">
        {overviewCards.map((card) => (
          <article key={card.label} className="platform-overview-card">
            <label>{card.label}</label>
            <strong>{card.value}</strong>
            <span>{card.hint}</span>
          </article>
        ))}
      </section>

      <section className="platform-management-grid">
        <div className="platform-panel">
          <div className="platform-panel__header">
            <div>
              <span>Hardware Profiles</span>
              <h2>Available device kits</h2>
            </div>
          </div>

          <div className="platform-list">
            {hardwareProfiles.map((profile) => (
              <button
                key={profile.id}
                type="button"
                onClick={() => {
                  selectHardwareProfile(profile);
                  setMobileProfileOpen(true);
                }}
                className={`platform-list-card ${
                  selectedProfile?.id === profile.id
                    ? "platform-list-card--selected"
                    : ""
                }`}
              >
                <div>
                  <p>{profile.profile_code}</p>
                  <h3>{profile.name}</h3>
                  <span>{profile.description}</span>
                </div>

                <strong>
                  {selectedProfile?.id === profile.id
                    ? hardwareProfileSensors.length
                    : "-"}{" "}
                  sensors
                </strong>
              </button>
            ))}
          </div>
        </div>

        <aside className="platform-panel platform-detail-panel">
          <div className="platform-panel__header">
            <div>
              <span>Selected Profile</span>
              <h2>{selectedProfile?.name ?? "No profile selected"}</h2>
            </div>
          </div>

          <p className="platform-detail-text">
            {selectedProfile?.description ??
              "Select a hardware profile to view details."}
          </p>

          <div className="platform-detail-grid">
            <div>
              <label>Profile Code</label>
              <strong>{selectedProfile?.profile_code ?? "-"}</strong>
            </div>

            <div>
              <label>Used by Devices</label>
              <strong>
                {selectedProfile
                  ? devices.filter(
                      (device) =>
                        device.hardware_profile_id === selectedProfile.id,
                    ).length
                  : "-"}
              </strong>
            </div>
          </div>

          <div className="platform-detail-section">
            <label>Supported Sensors</label>

            <div className="platform-chip-row">
              {hardwareProfileSensors.length > 0 ? (
                hardwareProfileSensors.map((sensor) => (
                  <span key={sensor.id}>{sensor.sensor_type}</span>
                ))
              ) : (
                <span>No sensors loaded</span>
              )}
            </div>
          </div>
        </aside>
      </section>

      <section className="platform-management-grid">
        <div className="platform-panel">
          <div className="platform-panel__header">
            <div>
              <span>Registered Devices</span>
              <h2>Provisioned hardware</h2>
            </div>

            <button className="platform-primary-button" type="button">
              Register Device
            </button>
          </div>

          <div className="platform-list">
            {filteredDevices.map((device) => (
              <button
                key={device.id}
                type="button"
                onClick={() => {
                  selectDevice(device);
                  setMobileDeviceOpen(true);
                }}
                className={`platform-list-card ${
                  activeDevice?.id === device.id
                    ? "platform-list-card--selected"
                    : ""
                }`}
              >
                <div>
                  <p>{device.hardware_profile_code}</p>
                  <h3>{device.device_code}</h3>
                  <span>{device.hardware_profile_name}</span>
                </div>

                <strong>{device.status}</strong>
              </button>
            ))}
          </div>
        </div>

        <aside className="platform-panel platform-detail-panel">
          <div className="platform-panel__header">
            <div>
              <span>Selected Device</span>
              <h2>{activeDevice?.device_code ?? "No device selected"}</h2>
            </div>
          </div>

          <div className="platform-detail-grid">
            <div>
              <label>Status</label>
              <strong>{activeDevice?.status ?? "-"}</strong>
            </div>

            <div>
              <label>Hardware Profile</label>
              <strong>{activeDevice?.hardware_profile_code ?? "-"}</strong>
            </div>
          </div>

          <div className="platform-detail-section">
            <label>Provisioned Sensors</label>

            <div className="platform-sensor-list">
              {deviceSensors.length > 0 ? (
                deviceSensors.map((sensor) => (
                  <div key={sensor.id}>
                    <span>✓</span>
                    <strong>{sensor.sensor_type}</strong>
                  </div>
                ))
              ) : (
                <div>
                  <span>!</span>
                  <strong>Select a device to load sensors</strong>
                </div>
              )}
            </div>
          </div>
        </aside>
      </section>

      {mobileProfileOpen && (
        <div className="platform-mobile-sheet">
          <div className="platform-mobile-sheet__content">
            <div className="platform-panel__header">
              <div>
                <span>Selected Profile</span>
                <h2>{selectedProfile?.name ?? "No profile selected"}</h2>
              </div>

              <button type="button" onClick={() => setMobileProfileOpen(false)}>
                Close
              </button>
            </div>

            <p className="platform-detail-text">
              {selectedProfile?.description ??
                "Select a hardware profile to view details."}
            </p>

            <div className="platform-chip-row">
              {hardwareProfileSensors.map((sensor) => (
                <span key={sensor.id}>{sensor.sensor_type}</span>
              ))}
            </div>
          </div>
        </div>
      )}

      {mobileDeviceOpen && (
        <div className="platform-mobile-sheet">
          <div className="platform-mobile-sheet__content">
            <div className="platform-panel__header">
              <div>
                <span>Selected Device</span>
                <h2>{activeDevice?.device_code ?? "No device selected"}</h2>
              </div>

              <button type="button" onClick={() => setMobileDeviceOpen(false)}>
                Close
              </button>
            </div>

            <div className="platform-detail-grid">
              <div>
                <label>Status</label>
                <strong>{activeDevice?.status ?? "-"}</strong>
              </div>

              <div>
                <label>Hardware Profile</label>
                <strong>{activeDevice?.hardware_profile_code ?? "-"}</strong>
              </div>
            </div>

            <div className="platform-sensor-list">
              {deviceSensors.map((sensor) => (
                <div key={sensor.id}>
                  <span>✓</span>
                  <strong>{sensor.sensor_type}</strong>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </main>
  );
}

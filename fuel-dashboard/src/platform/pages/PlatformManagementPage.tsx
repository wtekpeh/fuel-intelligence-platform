import { useEffect, useState } from "react";
import BottomSheet from "../../components/common/BottomSheet/BottomSheet";
import RegisterDeviceSheet from "../components/RegisterDeviceSheet";
import { useAssetStore } from "../store/assetStore";
import { useDeviceStore } from "../store/deviceStore";
import { useOrganizationStore } from "../store/organizationStore";
import { usePlatformStore } from "../store/platformStore";
import "../styles/platform.css";

type PlatformWorkspace = "business" | "platform";

export default function PlatformManagementPage() {
  const [activeWorkspace, setActiveWorkspace] =
    useState<PlatformWorkspace>("business");

  const [mobileProfileOpen, setMobileProfileOpen] = useState(false);
  const [mobileDeviceOpen, setMobileDeviceOpen] = useState(false);
  const [registerDeviceOpen, setRegisterDeviceOpen] = useState(false);

  const {
    organizations,
    selectedOrganization,
    loadOrganizations,
    selectOrganization,
  } = useOrganizationStore();

  const { assets, selectedAsset, selectedAssetRows, loadAssets, selectAsset } =
    useAssetStore();

  const { devices, selectedDevice, deviceSensors, loadDevices, selectDevice } =
    useDeviceStore();

  const {
    hardwareProfiles,
    selectedHardwareProfile,
    hardwareProfileSensors,
    loadHardwareProfiles,
    selectHardwareProfile,
  } = usePlatformStore();

  useEffect(() => {
    loadOrganizations();
    loadHardwareProfiles();
    loadDevices();
  }, [loadOrganizations, loadHardwareProfiles, loadDevices]);

  useEffect(() => {
    if (selectedOrganization) {
      loadAssets(selectedOrganization.organization_id);
    }
  }, [selectedOrganization, loadAssets]);

  const selectedProfile =
    selectedHardwareProfile ?? hardwareProfiles[0] ?? null;

  const businessDevices = selectedAsset
    ? devices.filter((device) =>
        selectedAssetRows.some((row) => row.device_id === device.id),
      )
    : devices;

  const activeDevice =
    businessDevices.find((device) => device.id === selectedDevice?.id) ??
    businessDevices[0] ??
    null;

  const overviewCards = [
    {
      label: "Organizations",
      value: organizations.length.toString(),
      hint: "Business accounts",
    },
    {
      label: "Assets",
      value: assets.length.toString(),
      hint: "Selected organization assets",
    },
    {
      label: "Devices",
      value: devices.length.toString(),
      hint: "Provisioned hardware",
    },
    {
      label: "Hardware Profiles",
      value: hardwareProfiles.length.toString(),
      hint: "Supported device kits",
    },
  ];

  return (
    <main className="platform-page">
      <header className="platform-header">
        <p className="platform-eyebrow">Platform Management</p>
        <h1>Device & Hardware Management</h1>
        <p>
          Manage organizations, assets, devices, hardware profiles, and
          automatically provisioned sensors before telemetry ingestion begins.
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

      <nav className="platform-workspace-tabs">
        <button
          type="button"
          className={
            activeWorkspace === "business"
              ? "platform-workspace-tabs__button platform-workspace-tabs__button--active"
              : "platform-workspace-tabs__button"
          }
          onClick={() => setActiveWorkspace("business")}
        >
          <strong>Business Hierarchy</strong>
          <span>Organizations, assets, devices, sensors</span>
        </button>

        <button
          type="button"
          className={
            activeWorkspace === "platform"
              ? "platform-workspace-tabs__button platform-workspace-tabs__button--active"
              : "platform-workspace-tabs__button"
          }
          onClick={() => setActiveWorkspace("platform")}
        >
          <strong>Platform Catalogue</strong>
          <span>Hardware profiles and sensor templates</span>
        </button>
      </nav>

      {activeWorkspace === "business" && (
        <section className="platform-business-grid">
          <div className="platform-panel">
            <div className="platform-panel__header">
              <div>
                <span>Organizations</span>
                <h2>Business accounts</h2>
              </div>
            </div>

            <div className="platform-list">
              {organizations.map((organization) => (
                <button
                  key={organization.organization_id}
                  type="button"
                  onClick={() => selectOrganization(organization)}
                  className={`platform-list-card ${
                    selectedOrganization?.organization_id ===
                    organization.organization_id
                      ? "platform-list-card--selected"
                      : ""
                  }`}
                >
                  <div>
                    <p>{organization.industry ?? "Organization"}</p>
                    <h3>{organization.organization_name}</h3>
                    <span>
                      {organization.asset_count} Assets •{" "}
                      {organization.device_count} Devices
                    </span>
                  </div>

                  <strong>{organization.online_device_count} Online</strong>
                </button>
              ))}
            </div>
          </div>

          <div className="platform-panel">
            <div className="platform-panel__header">
              <div>
                <span>Assets</span>
                <h2>Operational assets</h2>
              </div>
            </div>

            <div className="platform-list">
              {assets.map((asset) => (
                <button
                  key={asset.asset_id}
                  type="button"
                  onClick={() => selectAsset(asset)}
                  className={`platform-list-card ${
                    selectedAsset?.asset_id === asset.asset_id
                      ? "platform-list-card--selected"
                      : ""
                  }`}
                >
                  <div>
                    <p>{asset.asset_type}</p>
                    <h3>{asset.asset_name}</h3>
                    <span>
                      {asset.sensor_count} Sensors • {asset.open_alert_count}{" "}
                      Open Alerts
                    </span>
                  </div>

                  <strong>{asset.device_count} Devices</strong>
                </button>
              ))}
            </div>
          </div>

          <div className="platform-panel">
            <div className="platform-panel__header">
              <div>
                <span>Devices</span>
                <h2>Attached hardware</h2>
              </div>

              <button
                type="button"
                className="platform-primary-button"
                onClick={() => setRegisterDeviceOpen(true)}
              >
                Attach Device
              </button>
            </div>

            <div className="platform-list">
              {businessDevices.map((device) => (
                <button
                  key={device.id}
                  type="button"
                  onClick={() => {
                    selectDevice(device);

                    if (window.innerWidth <= 720) {
                      setMobileDeviceOpen(true);
                    }
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
                <span>Details</span>
                <h2>{activeDevice?.device_code ?? "No device selected"}</h2>
              </div>
            </div>

            <div className="platform-detail-grid">
              <div>
                <label>Organization</label>
                <strong>
                  {selectedOrganization?.organization_name ?? "-"}
                </strong>
              </div>

              <div>
                <label>Asset</label>
                <strong>{selectedAsset?.asset_name ?? "-"}</strong>
              </div>

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
      )}

      {activeWorkspace === "platform" && (
        <section className="platform-management-grid">
          <div className="platform-panel">
            <div className="platform-panel__header">
              <div>
                <span>Hardware Profiles</span>
                <h2>Sensor templates</h2>
              </div>
            </div>

            <div className="platform-list">
              {hardwareProfiles.map((profile) => (
                <button
                  key={profile.id}
                  type="button"
                  onClick={() => {
                    selectHardwareProfile(profile);

                    if (window.innerWidth <= 720) {
                      setMobileProfileOpen(true);
                    }
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
      )}

      <BottomSheet
        open={mobileProfileOpen}
        onClose={() => setMobileProfileOpen(false)}
        title={selectedProfile?.name ?? "Selected Profile"}
        size="medium"
      >
        <p className="platform-detail-text">
          {selectedProfile?.description ??
            "Select a hardware profile to view details."}
        </p>

        <div className="platform-chip-row">
          {hardwareProfileSensors.map((sensor) => (
            <span key={sensor.id}>{sensor.sensor_type}</span>
          ))}
        </div>
      </BottomSheet>

      <BottomSheet
        open={mobileDeviceOpen}
        onClose={() => setMobileDeviceOpen(false)}
        title={activeDevice?.device_code ?? "Selected Device"}
        size="medium"
      >
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
      </BottomSheet>

      <RegisterDeviceSheet
        open={registerDeviceOpen}
        onClose={() => setRegisterDeviceOpen(false)}
      />
    </main>
  );
}

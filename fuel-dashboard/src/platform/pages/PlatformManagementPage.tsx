import { useEffect, useState } from "react";
import RegisterDeviceSheet from "../components/RegisterDeviceSheet";
import DeviceOnboardingWizard from "../components/DeviceOnboardingWizard";
import FuelCalibrationPanel from "../components/FuelCalibrationPanel";
import { useAssetStore } from "../store/assetStore";
import { useDeviceStore } from "../store/deviceStore";
import { useDeviceModelStore } from "../store/deviceModelStore";
import { useHardwareStore } from "../store/hardwareStore";
import { useOrganizationStore } from "../store/organizationStore";
import "../styles/platform.css";

export default function PlatformManagementPage() {
  const [registerDeviceOpen, setRegisterDeviceOpen] = useState(false);
  const [deviceOnboardingOpen, setDeviceOnboardingOpen] = useState(false);

  const { organizations, selectedOrganization, loadOrganizations } =
    useOrganizationStore();

  const { assets, selectedAsset, selectedAssetRows, loadAssets, selectAsset } =
    useAssetStore();

  const { devices, selectedDevice, deviceSensors, loadDevices, selectDevice } =
    useDeviceStore();

  const { loadHardwareProfiles } = useHardwareStore();

  const { deviceModels, loadDeviceModels } = useDeviceModelStore();

  useEffect(() => {
    loadOrganizations();
    loadDevices();
    loadDeviceModels();
    loadHardwareProfiles();
  }, [loadOrganizations, loadDevices, loadDeviceModels, loadHardwareProfiles]);

  useEffect(() => {
    /*
     * Organization is the root context for customer-owned resources.
     *
     * Loading or restoring the organization workspace must not clear a
     * persisted device selection. Device validity is reconstructed from
     * fresh backend data by the device and asset stores.
     *
     * Once authenticated tenant switching exists, an explicit tenant
     * switch action should clear organization-scoped selections there.
     */
    if (selectedOrganization) {
      loadAssets(selectedOrganization.organization_id);
    }
  }, [selectedOrganization, loadAssets]);

  const businessDevices = devices.filter((device) => {
    if (!selectedAsset) {
      return false;
    }

    return selectedAssetRows.some((row) => row.device_id === device.id);
  });

  const getDeviceModelName = (deviceModelId?: string | null) => {
    if (!deviceModelId) return "-";

    const model = deviceModels.find((item) => item.id === deviceModelId);

    return model?.modelName ?? "-";
  };

  return (
    <main className="platform-page">
      <header className="platform-header">
        <div>
          <p className="platform-eyebrow">Organization Workspace</p>

          <h1>{selectedOrganization?.organization_name ?? "Organization"}</h1>

          <p>
            Manage your operational assets, connected devices, sensors, and
            calibration workflows from one workspace.
          </p>
        </div>
      </header>

      <section className="platform-organization-context">
        <div className="platform-organization-context__identity">
          <span>Organization Workspace</span>

          <div className="platform-organization-context__title-row">
            <div>
              <h2>
                {selectedOrganization?.organization_name ??
                  "No organization selected"}
              </h2>

              <p>
                {selectedOrganization
                  ? (selectedOrganization.industry ?? "Organization")
                  : "No organization workspace is currently available."}
              </p>
            </div>
          </div>
        </div>

        {selectedOrganization && (
          <div className="platform-organization-context__summary">
            <div>
              <label>Assets</label>
              <strong>{selectedOrganization.asset_count}</strong>
            </div>

            <div>
              <label>Devices</label>
              <strong>{selectedOrganization.device_count}</strong>
            </div>

            <div>
              <label>Online</label>
              <strong>{selectedOrganization.online_device_count}</strong>
            </div>
          </div>
        )}
      </section>

      <section className="platform-workspace-grid">
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
                onClick={() => {
                  selectAsset(asset);
                  selectDevice(null);
                }}
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
                    {asset.sensor_count} Sensors • {asset.open_alert_count} Open
                    Alerts
                  </span>
                </div>

                <strong>{asset.device_count} Devices</strong>
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="platform-management-grid">
        <div className="platform-panel">
          <div className="platform-panel__header">
            <div>
              <span>Asset Devices</span>
              <h2>
                {selectedAsset
                  ? `${selectedAsset.asset_name} devices`
                  : "Select an asset"}
              </h2>
            </div>

            <button
              type="button"
              className="platform-primary-button"
              onClick={() => setDeviceOnboardingOpen(true)}
            >
              Register Device
            </button>
          </div>

          <div className="platform-list">
            {!selectedAsset ? (
              <p className="platform-detail-text">
                Select an operational asset to view its connected devices.
              </p>
            ) : businessDevices.length === 0 ? (
              <p className="platform-detail-text">
                No provisioned devices are attached to this asset.
              </p>
            ) : (
              businessDevices.map((device) => (
                <button
                  key={device.id}
                  type="button"
                  onClick={() => selectDevice(device)}
                  className={`platform-list-card ${
                    selectedDevice?.id === device.id
                      ? "platform-list-card--selected"
                      : ""
                  }`}
                >
                  <div>
                    <p>{device.hardware_profile_code}</p>
                    <h3>{device.device_code}</h3>

                    <span>
                      Model:{" "}
                      {device.device_model_name ??
                        getDeviceModelName(device.device_model_id)}
                    </span>

                    <span>Profile: {device.hardware_profile_name}</span>
                  </div>

                  <strong>{device.status}</strong>
                </button>
              ))
            )}
          </div>
        </div>

        <aside className="platform-panel platform-detail-panel">
          <div className="platform-panel__header">
            <div>
              <span>Selected Device</span>

              <h2>{selectedDevice?.device_code ?? "No device selected"}</h2>
            </div>
          </div>

          {selectedDevice ? (
            <>
              <p className="platform-detail-text">
                Connected to{" "}
                <strong>{selectedAsset?.asset_name ?? "selected asset"}</strong>
                .
              </p>

              <div className="platform-detail-grid">
                <div>
                  <label>Model</label>
                  <strong>
                    {selectedDevice.device_model_name ??
                      getDeviceModelName(selectedDevice.device_model_id)}
                  </strong>
                </div>

                <div>
                  <label>Status</label>
                  <strong>{selectedDevice.status}</strong>
                </div>

                <div>
                  <label>Hardware Profile</label>
                  <strong>{selectedDevice.hardware_profile_name}</strong>
                </div>

                <div>
                  <label>Profile Code</label>
                  <strong>{selectedDevice.hardware_profile_code}</strong>
                </div>
              </div>

              <div className="platform-detail-section">
                <label>Installed Sensors</label>

                <div className="platform-chip-row">
                  {deviceSensors.length > 0 ? (
                    deviceSensors.map((sensor) => (
                      <span key={sensor.id}>
                        {sensor.sensor_type} • {sensor.sensor_code} •{" "}
                        {sensor.unit}
                      </span>
                    ))
                  ) : (
                    <span>No sensors registered for this device</span>
                  )}
                </div>
              </div>
            </>
          ) : (
            <p className="platform-detail-text">
              Select a device attached to this asset to view its configuration,
              sensors, and calibration workflow.
            </p>
          )}
        </aside>
      </section>

      {selectedDevice && (
        <section className="platform-calibration-section">
          <FuelCalibrationPanel />
        </section>
      )}

      <DeviceOnboardingWizard
        open={deviceOnboardingOpen}
        onClose={() => setDeviceOnboardingOpen(false)}
        organizations={organizations}
      />

      <RegisterDeviceSheet
        open={registerDeviceOpen}
        onClose={() => setRegisterDeviceOpen(false)}
      />
    </main>
  );
}

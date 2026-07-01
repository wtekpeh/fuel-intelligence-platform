import { useEffect, useState } from "react";
import RegisterDeviceSheet from "../components/RegisterDeviceSheet";
import { useAssetStore } from "../store/assetStore";
import { useDeviceStore } from "../store/deviceStore";
import { useDeviceModelStore } from "../store/deviceModelStore";
import { useHardwareStore } from "../store/hardwareStore";
import { useOrganizationStore } from "../store/organizationStore";
import "../styles/platform.css";

export default function PlatformManagementPage() {
  const [registerDeviceOpen, setRegisterDeviceOpen] = useState(false);

  const {
    organizations,
    selectedOrganization,
    loadOrganizations,
    selectOrganization,
  } = useOrganizationStore();

  const { assets, selectedAsset, selectedAssetRows, loadAssets, selectAsset } =
    useAssetStore();

  const { devices, selectedDevice, loadDevices, selectDevice } =
    useDeviceStore();

  const {
    hardwareProfiles,
    selectedHardwareProfile,
    hardwareProfileSensors,
    loadHardwareProfiles,
    selectHardwareProfile,
  } = useHardwareStore();

  const { deviceModels, loadDeviceModels } = useDeviceModelStore();

  const [selectedDeviceModelId, setSelectedDeviceModelId] = useState<
    string | null
  >(null);

  useEffect(() => {
    loadOrganizations();
    loadDevices();
    loadDeviceModels();
    loadHardwareProfiles();
  }, [loadOrganizations, loadDevices, loadDeviceModels, loadHardwareProfiles]);

  useEffect(() => {
    if (selectedOrganization) {
      loadAssets(selectedOrganization.organization_id);
    }
  }, [selectedOrganization, loadAssets]);

  const selectedProfile =
    selectedHardwareProfile ?? hardwareProfiles[0] ?? null;

  const businessDevices = devices.filter((device) => {
    const matchesAsset = selectedAsset
      ? selectedAssetRows.some((row) => row.device_id === device.id)
      : true;

    const matchesDeviceModel = selectedDeviceModelId
      ? device.device_model_id === selectedDeviceModelId
      : true;

    return matchesAsset && matchesDeviceModel;
  });

  const getDeviceModelName = (deviceModelId?: string | null) => {
    if (!deviceModelId) return "-";

    const model = deviceModels.find((item) => item.id === deviceModelId);

    return model?.modelName ?? "-";
  };

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
      label: "Device Models",
      value: deviceModels.length.toString(),
      hint: "Supported hardware models",
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
        <div>
          <p className="platform-eyebrow">Platform Management</p>
          <h1>Device & Hardware Management</h1>
          <p>
            Manage organizations, assets, devices, hardware models, hardware
            profiles, and provisioned sensors before telemetry ingestion begins.
          </p>
        </div>
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
                    {asset.sensor_count} Sensors • {asset.open_alert_count} Open
                    Alerts
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
              <span>Device Models</span>
              <h2>Supported hardware</h2>
            </div>
          </div>

          <div className="platform-list">
            {deviceModels.map((model) => (
              <button
                key={model.id}
                type="button"
                onClick={() => setSelectedDeviceModelId(model.id)}
                className={`platform-list-card ${
                  selectedDeviceModelId === model.id
                    ? "platform-list-card--selected"
                    : ""
                }`}
              >
                <div>
                  <p>{model.manufacturer ?? "Manufacturer"}</p>
                  <h3>{model.modelName}</h3>
                  <span>{model.modelCode}</span>
                </div>

                <strong>{model.isActive ? "Active" : "Inactive"}</strong>
              </button>
            ))}
          </div>
        </div>

        <div className="platform-panel">
          <div className="platform-panel__header">
            <div>
              <span>Hardware Profiles</span>
              <h2>Supported capabilities</h2>
            </div>
          </div>

          <div className="platform-list">
            {hardwareProfiles.map((profile) => (
              <button
                key={profile.id}
                type="button"
                onClick={() => selectHardwareProfile(profile)}
                className={`platform-list-card ${
                  selectedProfile?.id === profile.id
                    ? "platform-list-card--selected"
                    : ""
                }`}
              >
                <div>
                  <p>{profile.profileCode}</p>
                  <h3>{profile.name}</h3>
                  <span>{profile.description ?? "Hardware profile"}</span>
                </div>

                <strong>{profile.isActive ? "Active" : "Inactive"}</strong>
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="platform-management-grid">
        <div className="platform-panel">
          <div className="platform-panel__header">
            <div>
              <span>Registered Devices</span>
              <h2>Provisioned hardware</h2>
            </div>

            <button
              type="button"
              className="platform-primary-button"
              onClick={() => setRegisterDeviceOpen(true)}
            >
              Register Device
            </button>
          </div>

          <div className="platform-list">
            {businessDevices.map((device) => (
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
            ))}
          </div>
        </div>

        <aside className="platform-panel platform-detail-panel">
          <div className="platform-panel__header">
            <div>
              <span>Selected Hardware Profile</span>
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
              <strong>{selectedProfile?.profileCode ?? "-"}</strong>
            </div>

            <div>
              <label>Status</label>
              <strong>
                {selectedProfile?.isActive ? "Active" : "Inactive"}
              </strong>
            </div>
          </div>

          <div className="platform-detail-section">
            <label>Supported Sensors</label>

            <div className="platform-chip-row">
              {hardwareProfileSensors.length > 0 ? (
                hardwareProfileSensors.map((sensor) => (
                  <span key={sensor.id}>{sensor.sensorType}</span>
                ))
              ) : (
                <span>No sensors loaded</span>
              )}
            </div>
          </div>
        </aside>
      </section>

      <RegisterDeviceSheet
        open={registerDeviceOpen}
        onClose={() => setRegisterDeviceOpen(false)}
      />
    </main>
  );
}

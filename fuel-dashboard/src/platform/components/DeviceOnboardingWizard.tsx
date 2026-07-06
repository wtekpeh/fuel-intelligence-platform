import { useEffect, useState } from "react";
import { useOrganizationStore } from "../store/organizationStore";
import { useDeviceOnboardingStore } from "../store/deviceOnboardingStore";
import { useAssetStore } from "../store/assetStore";
import { useDeviceCatalogueStore } from "../store/deviceCatalogueStore";

interface WizardOrganization {
  organization_id: string;
  organization_name: string;
  industry?: string | null;
  asset_count: number;
  device_count: number;
  online_device_count: number;
}

interface DeviceOnboardingWizardProps {
  open: boolean;
  onClose: () => void;
  organizations: WizardOrganization[];
}
const steps = ["Organization", "Asset", "Device Model", "Review", "Complete"];

export default function DeviceOnboardingWizard({
  open,
  onClose,
  organizations,
}: DeviceOnboardingWizardProps) {
  const [showCreateOrganization, setShowCreateOrganization] = useState(false);
  const [organizationName, setOrganizationName] = useState("");
  const [industry, setIndustry] = useState("");
  const [showCreateAsset, setShowCreateAsset] = useState(false);
  const [assetName, setAssetName] = useState("");
  const [assetType, setAssetType] = useState("");
  const [registrationNumber, setRegistrationNumber] = useState("");

  const createOrganization = useOrganizationStore(
    (state) => state.createOrganization,
  );
  const {
    assets,
    loadAssets,
    selectedAsset,
    selectAsset,
    addLocalAsset,
    createAsset,
  } = useAssetStore();

  const { models, selectedModel, loadCatalogue, selectModel } =
    useDeviceCatalogueStore();

  const {
    step,
    selectedOrganizationId,
    selectedAssetId,
    selectedDeviceModelId,
    selectOrganization,
    selectAsset: selectAssetForOnboarding,
    selectDeviceModel,
    nextStep,
    previousStep,
  } = useDeviceOnboardingStore();

  useEffect(() => {
    if (step === "asset" && selectedOrganizationId) {
      loadAssets(selectedOrganizationId);
    }
  }, [step, selectedOrganizationId, loadAssets]);

  useEffect(() => {
    if (step === "device-model") {
      loadCatalogue();
    }
  }, [step, loadCatalogue]);

  const canProceed =
    (step === "organization" && selectedOrganizationId !== null) ||
    (step === "asset" && selectedAssetId !== null) ||
    (step === "device-model" && selectedDeviceModelId !== null) ||
    step === "review";

  const stepKeys = [
    "organization",
    "asset",
    "device-model",
    "review",
    "complete",
  ] as const;

  if (!open) {
    return null;
  }

  return (
    <div className="platform-wizard-overlay">
      <section className="platform-wizard">
        <header className="platform-wizard__header">
          <div>
            <p className="platform-eyebrow">Device Onboarding</p>
            <h2>Provision a New Device</h2>
          </div>

          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="platform-wizard__steps">
          {steps.map((label, index) => (
            <div
              key={label}
              className={`platform-wizard__step ${
                stepKeys[index] === step ? "platform-wizard__step--active" : ""
              }`}
            >
              <span>{index + 1}</span>
              <p>{label}</p>
            </div>
          ))}
        </div>

        <div className="platform-wizard__body">
          {step === "organization" && (
            <>
              <p className="platform-eyebrow">Step 1</p>

              <h3>Select Organization</h3>

              <p>Choose the customer account that will own this device.</p>

              <div
                style={{
                  display: "flex",
                  justifyContent: "flex-end",
                  marginTop: "18px",
                  marginBottom: "18px",
                }}
              >
                <button
                  type="button"
                  className="platform-primary-button"
                  onClick={() =>
                    setShowCreateOrganization(!showCreateOrganization)
                  }
                >
                  + Create Organization
                </button>
              </div>

              {showCreateOrganization && (
                <div className="platform-form">
                  <label>
                    Organization Name
                    <input
                      value={organizationName}
                      onChange={(event) =>
                        setOrganizationName(event.target.value)
                      }
                    />
                  </label>

                  <label>
                    Business Type / Sector
                    <input
                      value={industry}
                      onChange={(event) => setIndustry(event.target.value)}
                      placeholder="Example: Transport, Mining, Logistics"
                    />
                  </label>

                  <div
                    style={{
                      display: "flex",
                      justifyContent: "flex-end",
                      gap: "12px",
                    }}
                  >
                    <button
                      type="button"
                      className="platform-secondary-button"
                      onClick={() => setShowCreateOrganization(false)}
                    >
                      Cancel
                    </button>

                    <button
                      type="button"
                      className="platform-primary-button"
                      onClick={async () => {
                        const id = await createOrganization({
                          name: organizationName,
                          industry,
                        });

                        if (id) {
                          selectOrganization(id);
                          setOrganizationName("");
                          setIndustry("");
                          setShowCreateOrganization(false);
                        }
                      }}
                    >
                      Create Organization
                    </button>
                  </div>
                </div>
              )}

              <div className="platform-list" style={{ marginTop: "24px" }}>
                {organizations.map((organization) => (
                  <button
                    key={organization.organization_id}
                    type="button"
                    onClick={() =>
                      selectOrganization(organization.organization_id)
                    }
                    className={`platform-list-card ${
                      selectedOrganizationId === organization.organization_id
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
            </>
          )}

          {step === "asset" && (
            <>
              <p className="platform-eyebrow">Step 2</p>

              <h3>Select Asset</h3>

              <p>Choose the asset that will receive this device.</p>

              <div style={{ marginTop: "24px" }}>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "flex-end",
                    marginBottom: "18px",
                  }}
                >
                  <button
                    type="button"
                    className="platform-primary-button"
                    onClick={() => setShowCreateAsset(!showCreateAsset)}
                  >
                    + Create Asset
                  </button>
                </div>

                {showCreateAsset && (
                  <div
                    className="platform-form"
                    style={{ marginBottom: "24px" }}
                  >
                    <label>
                      Asset Name *
                      <input
                        value={assetName}
                        onChange={(event) => setAssetName(event.target.value)}
                        placeholder="Example: Fuel Truck GH-101"
                      />
                    </label>

                    <label>
                      Asset Type *
                      <input
                        value={assetType}
                        onChange={(event) => setAssetType(event.target.value)}
                        placeholder="Example: Truck, Generator, Excavator"
                      />
                    </label>

                    <label>
                      Registration Number
                      <input
                        value={registrationNumber}
                        onChange={(event) =>
                          setRegistrationNumber(event.target.value)
                        }
                        placeholder="Example: GT-1234-25"
                      />
                    </label>

                    <div
                      style={{
                        display: "flex",
                        justifyContent: "flex-end",
                        gap: "12px",
                      }}
                    >
                      <button
                        type="button"
                        className="platform-secondary-button"
                        onClick={() => setShowCreateAsset(false)}
                      >
                        Cancel
                      </button>

                      <button
                        type="button"
                        className="platform-primary-button"
                        disabled={
                          !assetName.trim() ||
                          !assetType.trim() ||
                          !selectedOrganizationId
                        }
                        onClick={async () => {
                          if (!selectedOrganizationId) {
                            return;
                          }

                          const assetId = await createAsset({
                            organizationId: selectedOrganizationId,
                            name: assetName.trim(),
                            assetType: assetType.trim(),
                            metadata: {
                              registration_number:
                                registrationNumber.trim() || null,
                            },
                          });

                          if (assetId) {
                            const newAsset = {
                              asset_id: assetId,
                              asset_name: assetName.trim(),
                              asset_type: assetType.trim(),
                              capacity_litres: null,
                              device_count: 0,
                              sensor_count: 0,
                              open_alert_count: 0,
                              rows: [],
                            };

                            addLocalAsset(newAsset);
                            selectAssetForOnboarding(assetId);

                            setAssetName("");
                            setAssetType("");
                            setRegistrationNumber("");
                            setShowCreateAsset(false);
                          }
                        }}
                      >
                        Create Asset
                      </button>
                    </div>
                  </div>
                )}

                <div className="platform-list">
                  {assets.map((asset) => (
                    <button
                      key={asset.asset_id}
                      type="button"
                      onClick={() => {
                        selectAsset(asset);
                        selectAssetForOnboarding(asset.asset_id);
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
                          {asset.sensor_count} Sensors •{" "}
                          {asset.open_alert_count} Open Alerts
                        </span>
                      </div>

                      <strong>{asset.device_count} Devices</strong>
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}

          {step === "device-model" && (
            <>
              <p className="platform-eyebrow">Step 3</p>

              <h3>Select Device Model</h3>

              <p>Choose the physical hardware being installed.</p>

              <div className="platform-list" style={{ marginTop: "24px" }}>
                {models.map((model) => {
                  const defaultProfile =
                    model.profiles.find((profile) => profile.isDefault) ??
                    model.profiles[0];

                  return (
                    <button
                      key={model.id}
                      type="button"
                      onClick={() => {
                        selectModel(model);
                        selectDeviceModel(model.id);
                      }}
                      className={`platform-list-card ${
                        selectedModel?.id === model.id
                          ? "platform-list-card--selected"
                          : ""
                      }`}
                    >
                      <div>
                        <p>{model.manufacturer ?? "Device Model"}</p>

                        <h3>{model.modelName}</h3>

                        <span
                          style={{
                            marginTop: "10px",
                            color: "#94a3b8",
                            fontSize: "0.95rem",
                            fontWeight: 500,
                            lineHeight: 1.6,
                            textTransform: "none",
                            letterSpacing: "normal",
                          }}
                        >
                          {model.description}
                        </span>

                        <div
                          className="platform-chip-row"
                          style={{ marginTop: "18px" }}
                        >
                          {defaultProfile?.sensors.map((sensor) => (
                            <span key={sensor.id} className="platform-tag">
                              {sensor.sensorType}
                            </span>
                          ))}
                        </div>

                        <div style={{ marginTop: "16px" }}>
                          <p
                            style={{
                              margin: 0,
                              color: "#64748b",
                              fontSize: "0.7rem",
                              fontWeight: 900,
                              letterSpacing: "0.08em",
                              textTransform: "uppercase",
                            }}
                          >
                            Auto Configuration
                          </p>

                          <span className="platform-tag">
                            {defaultProfile?.name}
                          </span>
                        </div>
                      </div>

                      <div
                        style={{
                          display: "flex",
                          flexDirection: "column",
                          alignItems: "flex-end",
                          gap: "10px",
                        }}
                      >
                        <strong>
                          {model.isActive ? "ACTIVE" : "INACTIVE"}
                        </strong>

                        {selectedModel?.id === model.id && (
                          <span className="platform-tag">✓ Selected</span>
                        )}
                      </div>
                    </button>
                  );
                })}
              </div>
            </>
          )}
        </div>

        <footer className="platform-wizard__footer">
          <button type="button" onClick={onClose}>
            Cancel
          </button>

          <div>
            <button
              type="button"
              className="platform-secondary-button"
              disabled={step === "organization"}
              onClick={previousStep}
            >
              Back
            </button>

            <button
              type="button"
              className="platform-primary-button"
              disabled={!canProceed}
              onClick={nextStep}
            >
              Next
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

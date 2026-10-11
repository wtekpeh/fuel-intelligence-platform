import { useEffect, useState } from "react";
import { useDeviceOnboardingStore } from "../store/deviceOnboardingStore";
import { useAssetStore } from "../store/assetStore";

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
const steps = ["Organization", "Asset", "Verify ORBI Device", "Complete"];

export default function DeviceOnboardingWizard({
  open,
  onClose,
  organizations,
}: DeviceOnboardingWizardProps) {
  const [showCreateAsset, setShowCreateAsset] = useState(false);
  const [assetName, setAssetName] = useState("");
  const [assetType, setAssetType] = useState("");
  const [registrationNumber, setRegistrationNumber] = useState("");

  const {
    assets,
    loadAssets,
    selectedAsset,
    selectAsset,
    addLocalAsset,
    createAsset,
  } = useAssetStore();

  const {
    step,
    selectedOrganizationId,
    selectedAssetId,
    deviceCode,
    verifiedInventoryDevice,
    isVerifyingDevice,
    verificationError,
    isProvisioningDevice,
    provisioningError,
    provisionedDeviceId,
    setDeviceCode,
    verifyDeviceCode,
    provisionVerifiedDevice,
    selectOrganization,
    selectAsset: selectAssetForOnboarding,
    nextStep,
    previousStep,
  } = useDeviceOnboardingStore();

  useEffect(() => {
    if (step === "asset" && selectedOrganizationId) {
      loadAssets(selectedOrganizationId);
    }
  }, [step, selectedOrganizationId, loadAssets]);

  const canProceed =
    (step === "organization" && selectedOrganizationId !== null) ||
    (step === "asset" && selectedAssetId !== null) ||
    step === "review";

  const stepKeys = ["organization", "asset", "review", "complete"] as const;

  if (!open) {
    return null;
  }

  return (
    <div className="platform-wizard-overlay">
      <section className="platform-wizard">
        <header className="platform-wizard__header">
          <div>
            <p className="platform-eyebrow">Device Onboarding</p>
            <h2>Register Your ORBI Device</h2>
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

          {step === "review" && (
            <>
              <p className="platform-eyebrow">Step 3</p>

              <h3>Verify ORBI Device</h3>

              <p>
                Verify the manufactured ORBI device before assigning it to the
                selected asset.
              </p>

              <div className="platform-panel" style={{ marginTop: "24px" }}>
                <p className="platform-eyebrow">
                  {verifiedInventoryDevice
                    ? "Ready to Provision"
                    : "Ready to Verify"}
                </p>

                <h3 style={{ marginTop: "8px" }}>
                  {verifiedInventoryDevice
                    ? "The ORBI device has been verified."
                    : "Verify the ORBI device before provisioning."}
                </h3>

                <span>
                  {verifiedInventoryDevice
                    ? "The platform has confirmed this manufactured device and it is ready to be assigned to the selected asset."
                    : "Enter the ORBI Device Code below to verify the manufactured device."}
                </span>
              </div>

              <div
                style={{
                  display: "grid",
                  gap: "18px",
                  marginTop: "28px",
                }}
              >
                <div className="platform-panel">
                  <p className="platform-eyebrow">Organization</p>

                  <h3 style={{ marginTop: "8px" }}>
                    {organizations.find(
                      (item) => item.organization_id === selectedOrganizationId,
                    )?.organization_name ?? "-"}
                  </h3>
                </div>

                <div className="platform-panel">
                  <p className="platform-eyebrow">Asset</p>

                  <h3 style={{ marginTop: "8px" }}>
                    {selectedAsset?.asset_name ?? "-"}
                  </h3>
                </div>

                <div className="platform-panel">
                  <p className="platform-eyebrow">ORBI Device Verification</p>

                  <label
                    style={{
                      display: "grid",
                      gap: "8px",
                      marginTop: "18px",
                    }}
                  >
                    Device Code
                    <input
                      value={deviceCode}
                      onChange={(event) => setDeviceCode(event.target.value)}
                      placeholder="Example: ORBI-TEST-001"
                    />
                  </label>

                  <div
                    style={{
                      display: "flex",
                      gap: "12px",
                      marginTop: "18px",
                    }}
                  >
                    <button
                      type="button"
                      className="platform-primary-button"
                      disabled={isVerifyingDevice || !deviceCode.trim()}
                      onClick={verifyDeviceCode}
                    >
                      {isVerifyingDevice ? "Verifying..." : "Verify Device"}
                    </button>
                  </div>

                  {verificationError && (
                    <p
                      style={{
                        marginTop: "16px",
                        color: "#dc2626",
                        fontWeight: 600,
                      }}
                    >
                      {verificationError}
                    </p>
                  )}

                  {verifiedInventoryDevice && (
                    <div
                      className="platform-panel"
                      style={{
                        marginTop: "24px",
                        display: "grid",
                        gap: "12px",
                      }}
                    >
                      <p
                        style={{
                          margin: 0,
                          color: "#166534",
                          fontWeight: 700,
                        }}
                      >
                        ✓ ORBI Device Verified
                      </p>

                      <span>
                        <strong>Device Code:</strong>{" "}
                        {verifiedInventoryDevice.device_code}
                      </span>

                      <span>
                        <strong>Assignment:</strong> Confirmed for the selected
                        organization
                      </span>

                      <p style={{ margin: 0, lineHeight: 1.5 }}>
                        This device is assigned to your organization and is
                        eligible for activation. Continue to link it to your
                        selected asset.
                      </p>
                    </div>
                  )}

                  {provisioningError && (
                    <p
                      style={{
                        marginTop: "16px",
                        color: "#dc2626",
                        fontWeight: 600,
                      }}
                    >
                      {provisioningError}
                    </p>
                  )}
                </div>

                <div className="platform-panel">
                  <p className="platform-eyebrow">Device Assignment</p>

                  <h3 style={{ marginTop: "8px" }}>
                    {verifiedInventoryDevice
                      ? "Ready for Activation"
                      : "Awaiting Device Verification"}
                  </h3>

                  <div
                    style={{
                      display: "grid",
                      gap: "12px",
                      marginTop: "18px",
                    }}
                  >
                    <span>
                      <strong>Device Code:</strong>{" "}
                      {verifiedInventoryDevice?.device_code ?? "Not verified"}
                    </span>

                    <span>
                      <strong>Organization:</strong>{" "}
                      {organizations.find(
                        (organization) =>
                          organization.organization_id ===
                          selectedOrganizationId,
                      )?.organization_name ?? "Not selected"}
                    </span>

                    <span>
                      <strong>Activation Status:</strong>{" "}
                      {verifiedInventoryDevice
                        ? "Eligible for activation"
                        : "Verification required"}
                    </span>
                  </div>
                </div>

                <div className="platform-panel">
                  <p className="platform-eyebrow">Activation Summary</p>

                  <div
                    style={{
                      marginTop: "16px",
                      display: "grid",
                      gap: "10px",
                    }}
                  >
                    <span>✓ Register device in the platform</span>

                    <span>✓ Link device to selected asset</span>

                    <span>✓ Apply hardware profile automatically</span>

                    <span>✓ Enable installed sensors</span>

                    <span>✓ Activate operational intelligence</span>
                  </div>
                </div>
              </div>
            </>
          )}

          {step === "complete" && (
            <div
              className="platform-panel"
              style={{
                padding: "28px",
                display: "grid",
                gap: "16px",
                textAlign: "center",
              }}
            >
              <h3 style={{ color: "#166534", margin: 0 }}>
                ✓ ORBI Device Activated Successfully
              </h3>

              <p style={{ margin: 0 }}>
                Your ORBI device has been registered and linked to your selected
                asset.
              </p>

              <p style={{ margin: 0 }}>
                <strong>Device Code:</strong> {deviceCode}
              </p>

              {provisionedDeviceId && (
                <p style={{ margin: 0, overflowWrap: "anywhere" }}>
                  <strong>Operational Device ID:</strong> {provisionedDeviceId}
                </p>
              )}
            </div>
          )}
        </div>

        <footer className="platform-wizard__footer">
          {step === "complete" ? (
            <button
              type="button"
              className="platform-primary-button"
              onClick={onClose}
            >
              Close
            </button>
          ) : (
            <>
              <button type="button" onClick={onClose}>
                Cancel
              </button>

              <div>
                <button
                  type="button"
                  className="platform-secondary-button"
                  disabled={step === "organization" || isProvisioningDevice}
                  onClick={previousStep}
                >
                  Back
                </button>

                <button
                  type="button"
                  className="platform-primary-button"
                  disabled={
                    step === "review"
                      ? !verifiedInventoryDevice || isProvisioningDevice
                      : !canProceed
                  }
                  onClick={async () => {
                    if (step === "organization" || step === "asset") {
                      nextStep();
                      return;
                    }

                    if (step === "review") {
                      await provisionVerifiedDevice();
                    }
                  }}
                >
                  {step === "review"
                    ? isProvisioningDevice
                      ? "Activating..."
                      : "Activate Device"
                    : "Next"}
                </button>
              </div>
            </>
          )}
        </footer>
      </section>
    </div>
  );
}

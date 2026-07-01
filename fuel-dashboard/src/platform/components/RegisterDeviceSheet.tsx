import { useEffect, useState } from "react";
import BottomSheet from "../../components/common/BottomSheet/BottomSheet";
import { useAssetStore } from "../store/assetStore";
import { useDeviceStore } from "../store/deviceStore";
import { useDeviceModelStore } from "../store/deviceModelStore";
import { useHardwareStore } from "../store/hardwareStore";

interface RegisterDeviceSheetProps {
  open: boolean;
  onClose: () => void;
}

export default function RegisterDeviceSheet({
  open,
  onClose,
}: RegisterDeviceSheetProps) {
  const { selectedAsset } = useAssetStore();
  const { createDevice, loading, error, clearError } = useDeviceStore();

  const {
    hardwareProfiles,
    selectedHardwareProfile,
    loadHardwareProfiles,
    selectHardwareProfile,
  } = useHardwareStore();

  const {
    deviceModels,
    selectedDeviceModel,
    loadDeviceModels,
    selectDeviceModel,
  } = useDeviceModelStore();

  const [deviceCode, setDeviceCode] = useState("");
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const hardwareProfileId = selectedHardwareProfile?.id ?? "";
  const deviceModelId = selectedDeviceModel?.id ?? "";

  useEffect(() => {
    if (!open) {
      return;
    }

    loadHardwareProfiles();
    loadDeviceModels();
    clearError();
  }, [open, loadHardwareProfiles, loadDeviceModels, clearError]);

  const canSubmit =
    Boolean(selectedAsset) &&
    deviceCode.trim() !== "" &&
    deviceModelId !== "" &&
    hardwareProfileId !== "" &&
    !loading;

  const handleSubmit = async () => {
    if (!selectedAsset || !canSubmit) {
      return;
    }

    await createDevice({
      asset_id: selectedAsset.asset_id,
      device_model_id: deviceModelId,
      device_code: deviceCode.trim(),
      hardware_profile_id: hardwareProfileId,
    });

    setSuccessMessage(
      "Device registered successfully. Sensors were provisioned automatically.",
    );

    setDeviceCode("");
  };

  return (
    <BottomSheet
      open={open}
      onClose={onClose}
      title="Register Device"
      size="medium"
    >
      <div className="platform-form">
        <label>
          Asset
          <input
            value={selectedAsset?.asset_name ?? "Select an asset first"}
            disabled
          />
        </label>

        <label>
          Device Code
          <input
            value={deviceCode}
            onChange={(event) => {
              setDeviceCode(event.target.value);
              setSuccessMessage(null);
              clearError();
            }}
            placeholder="Example: ORBI-GPS-001"
          />
        </label>

        <label>
          Device Model
          <select
            value={deviceModelId}
            onChange={(event) => {
              const nextModel =
                deviceModels.find((model) => model.id === event.target.value) ??
                null;

              selectDeviceModel(nextModel);
              setSuccessMessage(null);
              clearError();
            }}
          >
            <option value="">Select device model</option>

            {deviceModels.map((model) => (
              <option key={model.id} value={model.id}>
                {model.modelCode} — {model.modelName}
              </option>
            ))}
          </select>
        </label>

        <label>
          Hardware Profile
          <select
            value={hardwareProfileId}
            onChange={(event) => {
              const nextProfile =
                hardwareProfiles.find(
                  (profile) => profile.id === event.target.value,
                ) ?? null;

              selectHardwareProfile(nextProfile);
              setSuccessMessage(null);
              clearError();
            }}
          >
            <option value="">Select hardware profile</option>

            {hardwareProfiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.profileCode} — {profile.name}
              </option>
            ))}
          </select>
        </label>

        {!selectedAsset && (
          <div className="platform-form-error">
            Select an asset before registering a device.
          </div>
        )}

        {error && <div className="platform-form-error">{error}</div>}

        {successMessage && (
          <div className="platform-form-success">{successMessage}</div>
        )}

        <button
          type="button"
          className="platform-primary-button"
          disabled={!canSubmit}
          onClick={handleSubmit}
        >
          {loading ? "Registering..." : "Register Device"}
        </button>
      </div>
    </BottomSheet>
  );
}

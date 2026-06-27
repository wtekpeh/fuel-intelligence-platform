import { useState } from "react";
import BottomSheet from "../../components/common/BottomSheet/BottomSheet";
import { usePlatformStore } from "../store/platformStore";
import { useDeviceStore } from "../store/deviceStore";

interface RegisterDeviceSheetProps {
  open: boolean;
  onClose: () => void;
}

const DEMO_ASSET_ID = "aa76d222-e5e7-44d5-a74e-daa27ed13b7a";

export default function RegisterDeviceSheet({
  open,
  onClose,
}: RegisterDeviceSheetProps) {
  const { hardwareProfiles } = usePlatformStore();

  const { createDevice } = useDeviceStore();

  const [deviceCode, setDeviceCode] = useState("");
  const [hardwareProfileId, setHardwareProfileId] = useState("");
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const canSubmit = deviceCode.trim() !== "" && hardwareProfileId !== "";

  const handleSubmit = async () => {
    if (!canSubmit) {
      return;
    }

    await createDevice({
      asset_id: DEMO_ASSET_ID,
      device_code: deviceCode.trim(),
      hardware_profile_id: hardwareProfileId,
    });

    setSuccessMessage(
      "Device registered successfully. Sensors were provisioned automatically.",
    );

    setDeviceCode("");
    setHardwareProfileId("");
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
          Device Code
          <input
            value={deviceCode}
            onChange={(event) => {
              setDeviceCode(event.target.value);
              setSuccessMessage(null);
            }}
            placeholder="Example: GPS-TRUCK-001"
          />
        </label>

        <label>
          Asset
          <input value="Demo Fuel Truck" disabled />
        </label>

        <label>
          Hardware Profile
          <select
            value={hardwareProfileId}
            onChange={(event) => {
              setHardwareProfileId(event.target.value);
              setSuccessMessage(null);
            }}
          >
            <option value="">Select hardware profile</option>

            {hardwareProfiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.profile_code} — {profile.name}
              </option>
            ))}
          </select>
        </label>

        {successMessage && (
          <div className="platform-form-success">{successMessage}</div>
        )}

        <button
          type="button"
          className="platform-primary-button"
          disabled={!canSubmit}
          onClick={handleSubmit}
        >
          Register Device
        </button>
      </div>
    </BottomSheet>
  );
}

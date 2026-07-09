import { create } from "zustand";
import {
  provisionInventoryDevice,
  verifyInventoryDevice,
} from "../../api/platformApi";

type OnboardingStep = "organization" | "asset" | "review" | "complete";

interface VerifiedInventoryDevice {
  id: string;
  device_code: string;
  serial_number: string;
  imei: string | null;
  device_model_id: string;
  hardware_profile_id: string;
  firmware_version: string | null;
  production_batch: string | null;
  inventory_status: string;
  quality_test_status: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

interface DeviceOnboardingStore {
  step: OnboardingStep;

  selectedOrganizationId: string | null;
  selectedAssetId: string | null;
  selectedDeviceModelId: string | null;
  selectedHardwareProfileId: string | null;

  deviceCode: string;
  verifiedInventoryDevice: VerifiedInventoryDevice | null;

  isVerifyingDevice: boolean;
  verificationError: string | null;

  isProvisioningDevice: boolean;
  provisioningError: string | null;
  provisionedDeviceId: string | null;

  setStep: (step: OnboardingStep) => void;
  nextStep: () => void;
  previousStep: () => void;

  selectOrganization: (organizationId: string) => void;
  selectAsset: (assetId: string) => void;
  selectDeviceModel: (deviceModelId: string) => void;
  selectHardwareProfile: (hardwareProfileId: string | null) => void;

  setDeviceCode: (deviceCode: string) => void;
  verifyDeviceCode: () => Promise<void>;
  provisionVerifiedDevice: () => Promise<boolean>;

  reset: () => void;
}

const stepOrder: OnboardingStep[] = [
  "organization",
  "asset",
  "review",
  "complete",
];

export const useDeviceOnboardingStore = create<DeviceOnboardingStore>(
  (set, get) => ({
    step: "organization",

    selectedOrganizationId: null,
    selectedAssetId: null,
    selectedDeviceModelId: null,
    selectedHardwareProfileId: null,

    deviceCode: "",
    verifiedInventoryDevice: null,

    isVerifyingDevice: false,
    verificationError: null,

    isProvisioningDevice: false,
    provisioningError: null,
    provisionedDeviceId: null,

    setStep: (step) => set({ step }),

    nextStep: () => {
      const currentStep = get().step;
      const currentIndex = stepOrder.indexOf(currentStep);
      const nextIndex = Math.min(currentIndex + 1, stepOrder.length - 1);

      set({ step: stepOrder[nextIndex] });
    },

    previousStep: () => {
      const currentStep = get().step;
      const currentIndex = stepOrder.indexOf(currentStep);
      const previousIndex = Math.max(currentIndex - 1, 0);

      set({ step: stepOrder[previousIndex] });
    },

    selectOrganization: (organizationId) =>
      set({
        selectedOrganizationId: organizationId,
        selectedAssetId: null,
        selectedDeviceModelId: null,
        selectedHardwareProfileId: null,
        deviceCode: "",
        verifiedInventoryDevice: null,
        verificationError: null,
        provisioningError: null,
        provisionedDeviceId: null,
      }),

    selectAsset: (assetId) =>
      set({
        selectedAssetId: assetId,
        selectedDeviceModelId: null,
        selectedHardwareProfileId: null,
        deviceCode: "",
        verifiedInventoryDevice: null,
        verificationError: null,
        provisioningError: null,
        provisionedDeviceId: null,
      }),

    selectDeviceModel: (deviceModelId) =>
      set({
        selectedDeviceModelId: deviceModelId,
        deviceCode: "",
        verifiedInventoryDevice: null,
        verificationError: null,
        provisioningError: null,
        provisionedDeviceId: null,
      }),

    selectHardwareProfile: (hardwareProfileId) =>
      set({
        selectedHardwareProfileId: hardwareProfileId,
        deviceCode: "",
        verifiedInventoryDevice: null,
        verificationError: null,
        provisioningError: null,
        provisionedDeviceId: null,
      }),

    setDeviceCode: (deviceCode) =>
      set({
        deviceCode,
        verifiedInventoryDevice: null,
        verificationError: null,
        provisioningError: null,
        provisionedDeviceId: null,
      }),

    verifyDeviceCode: async () => {
      const deviceCode = get().deviceCode.trim();

      if (!deviceCode) {
        set({
          verificationError: "Enter a device code before verification.",
          verifiedInventoryDevice: null,
        });

        return;
      }

      set({
        isVerifyingDevice: true,
        verificationError: null,
        verifiedInventoryDevice: null,
      });

      try {
        const response = await verifyInventoryDevice(deviceCode);

        if (!response.found || !response.device) {
          set({
            verificationError: "No ORBI inventory device found for this code.",
            verifiedInventoryDevice: null,
          });

          return;
        }

        set({
          verifiedInventoryDevice: response.device,
          verificationError: null,
        });
      } catch {
        set({
          verificationError: "Failed to verify ORBI inventory device.",
          verifiedInventoryDevice: null,
        });
      } finally {
        set({ isVerifyingDevice: false });
      }
    },

    provisionVerifiedDevice: async () => {
      const selectedAssetId = get().selectedAssetId;
      const verifiedInventoryDevice = get().verifiedInventoryDevice;

      if (!selectedAssetId) {
        set({
          provisioningError: "Select an asset before provisioning.",
        });

        return false;
      }

      if (!verifiedInventoryDevice) {
        set({
          provisioningError: "Verify an inventory device before provisioning.",
        });

        return false;
      }

      set({
        isProvisioningDevice: true,
        provisioningError: null,
        provisionedDeviceId: null,
      });

      try {
        const response = await provisionInventoryDevice({
          inventoryDeviceId: verifiedInventoryDevice.id,
          assetId: selectedAssetId,
        });

        set({
          provisionedDeviceId: response.deviceId,
          provisioningError: null,
          step: "complete",
        });

        return true;
      } catch {
        set({
          provisioningError: "Failed to provision inventory device.",
        });

        return false;
      } finally {
        set({ isProvisioningDevice: false });
      }
    },

    reset: () =>
      set({
        step: "organization",
        selectedOrganizationId: null,
        selectedAssetId: null,
        selectedDeviceModelId: null,
        selectedHardwareProfileId: null,
        deviceCode: "",
        verifiedInventoryDevice: null,
        isVerifyingDevice: false,
        verificationError: null,
        isProvisioningDevice: false,
        provisioningError: null,
        provisionedDeviceId: null,
      }),
  }),
);

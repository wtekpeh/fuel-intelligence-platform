import { create } from "zustand";
import {
  activateCustomerDevice,
  listActivationEligibleDevices,
} from "../../api/deviceActivationApi";

type OnboardingStep = "organization" | "asset" | "review" | "complete";

interface VerifiedInventoryDevice {
  id: string;
  device_code: string;
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
      const { deviceCode, selectedOrganizationId } = get();

      const normalizedCode = deviceCode.trim();

      if (!selectedOrganizationId) {
        set({
          verificationError:
            "Select an organization before verifying a device.",
          verifiedInventoryDevice: null,
        });
        return;
      }

      if (!normalizedCode) {
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
        const eligibleDevices = await listActivationEligibleDevices();

        const matchedDevice = eligibleDevices.find(
          (device) =>
            device.organization_id === selectedOrganizationId &&
            device.device_code.toLowerCase() === normalizedCode.toLowerCase(),
        );

        if (!matchedDevice) {
          set({
            verificationError:
              "This device is not assigned to your organization or is not eligible for activation.",
            verifiedInventoryDevice: null,
          });
          return;
        }

        set({
          verifiedInventoryDevice: {
            id: matchedDevice.inventory_device_id,
            device_code: matchedDevice.device_code,
          },
          verificationError: null,
        });
      } catch {
        set({
          verificationError: "Unable to verify the assigned ORBI device.",
          verifiedInventoryDevice: null,
        });
      } finally {
        set({ isVerifyingDevice: false });
      }
    },

    provisionVerifiedDevice: async () => {
      const {
        selectedAssetId,
        selectedOrganizationId,
        verifiedInventoryDevice,
      } = get();

      if (!selectedOrganizationId || !selectedAssetId) {
        set({
          provisioningError:
            "Select an organization and asset before activation.",
        });
        return false;
      }

      if (!verifiedInventoryDevice) {
        set({
          provisioningError:
            "Verify your assigned ORBI device before activation.",
        });
        return false;
      }

      set({
        isProvisioningDevice: true,
        provisioningError: null,
        provisionedDeviceId: null,
      });

      try {
        const operationalDeviceId = await activateCustomerDevice({
          inventory_device_id: verifiedInventoryDevice.id,
          asset_id: selectedAssetId,
        });

        set({
          provisionedDeviceId: operationalDeviceId,
          provisioningError: null,
          step: "complete",
        });

        return true;
      } catch {
        set({
          provisioningError:
            "Device activation failed. Check your organization assignment and try again.",
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

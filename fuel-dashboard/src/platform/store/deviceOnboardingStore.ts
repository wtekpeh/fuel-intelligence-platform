import { create } from "zustand";

type OnboardingStep =
  | "organization"
  | "asset"
  | "device-model"
  | "review"
  | "complete";

interface DeviceOnboardingStore {
  step: OnboardingStep;
  selectedOrganizationId: string | null;
  selectedAssetId: string | null;
  selectedDeviceModelId: string | null;

  setStep: (step: OnboardingStep) => void;
  nextStep: () => void;
  previousStep: () => void;

  selectOrganization: (organizationId: string) => void;
  selectAsset: (assetId: string) => void;
  selectDeviceModel: (deviceModelId: string) => void;

  reset: () => void;
}

const stepOrder: OnboardingStep[] = [
  "organization",
  "asset",
  "device-model",
  "review",
  "complete",
];

export const useDeviceOnboardingStore = create<DeviceOnboardingStore>(
  (set, get) => ({
    step: "organization",
    selectedOrganizationId: null,
    selectedAssetId: null,
    selectedDeviceModelId: null,

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
      }),

    selectAsset: (assetId) =>
      set({
        selectedAssetId: assetId,
        selectedDeviceModelId: null,
      }),

    selectDeviceModel: (deviceModelId) =>
      set({
        selectedDeviceModelId: deviceModelId,
      }),

    reset: () =>
      set({
        step: "organization",
        selectedOrganizationId: null,
        selectedAssetId: null,
        selectedDeviceModelId: null,
      }),
  }),
);

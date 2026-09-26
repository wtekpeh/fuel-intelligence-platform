import { create } from "zustand";

import {
  abandonFuelCalibrationSession,
  activateFuelCalibrationProduction,
  applyFuelCalibrationAnchor,
  captureFuelCalibrationPoint,
  completeFuelCalibrationSession,
  createFuelCalibrationProfile,
  fetchFuelCalibrationProfile,
  pauseFuelCalibrationSession,
  publishFuelCalibrationProfile,
  resumeFuelCalibrationSession,
  startFuelCalibrationSession,
  supersedeFuelCalibrationProfile,
} from "../../api/platformApi";

import type {
  ApplyFuelCalibrationAnchorRequest,
  CaptureFuelCalibrationPointRequest,
  CreateFuelCalibrationProfileRequest,
  FuelCalibrationProfile,
  StartFuelCalibrationSessionRequest,
} from "../types/fuelCalibration";

interface FuelCalibrationStore {
  profile: FuelCalibrationProfile | null;
  selectedSensorId: string | null;

  loading: boolean;
  error: string | null;

  loadProfile: (sensorId: string) => Promise<void>;

  createProfile: (
    sensorId: string,
    request: CreateFuelCalibrationProfileRequest,
  ) => Promise<void>;

  startSession: (request: StartFuelCalibrationSessionRequest) => Promise<void>;

  capturePoint: (
    sessionId: string,
    request: CaptureFuelCalibrationPointRequest,
  ) => Promise<void>;

  pauseSession: (sessionId: string) => Promise<void>;

  resumeSession: (sessionId: string) => Promise<void>;

  applyAnchor: (
    sessionId: string,
    request: ApplyFuelCalibrationAnchorRequest,
  ) => Promise<void>;

  completeSession: (sessionId: string) => Promise<void>;

  abandonSession: (sessionId: string) => Promise<void>;

  publishProfile: () => Promise<void>;

  activateProduction: () => Promise<void>;

  supersedeProfile: () => Promise<void>;

  clearProfile: () => void;

  clearError: () => void;
}

export const useFuelCalibrationStore = create<FuelCalibrationStore>(
  (set, get) => {
    /*
     * Reload the currently selected sensor's calibration profile after
     * a successful workflow mutation.
     *
     * The backend remains the source of truth for:
     *
     * - profile lifecycle status;
     * - confidence;
     * - verified coverage;
     * - session status;
     * - resolved calibration points;
     * - publication state.
     *
     * We therefore avoid reconstructing those transitions manually in
     * the frontend store.
     */
    const reloadCurrentProfile = async () => {
      const sensorId = get().selectedSensorId;

      if (!sensorId) {
        return;
      }

      const profile = await fetchFuelCalibrationProfile(sensorId);

      set({ profile });
    };

    return {
      profile: null,
      selectedSensorId: null,

      loading: false,
      error: null,

      loadProfile: async (sensorId) => {
        set({
          selectedSensorId: sensorId,
          profile: null,
          loading: true,
          error: null,
        });

        try {
          const profile = await fetchFuelCalibrationProfile(sensorId);

          /*
           * Protect against a stale request replacing the state after
           * the installer has already selected another sensor.
           */
          if (get().selectedSensorId !== sensorId) {
            return;
          }

          set({
            profile,
            loading: false,
          });
        } catch {
          if (get().selectedSensorId !== sensorId) {
            return;
          }

          set({
            loading: false,
            error: "Failed to load fuel calibration profile.",
          });
        }
      },

      createProfile: async (sensorId, request) => {
        set({
          selectedSensorId: sensorId,
          loading: true,
          error: null,
        });

        try {
          await createFuelCalibrationProfile(sensorId, request);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to create fuel calibration profile.",
          });
        }
      },

      startSession: async (request) => {
        const profile = get().profile;

        if (!profile) {
          set({
            error: "No fuel calibration profile is selected.",
          });

          return;
        }

        set({
          loading: true,
          error: null,
        });

        try {
          await startFuelCalibrationSession(profile.id, request);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to start fuel calibration session.",
          });
        }
      },

      capturePoint: async (sessionId, request) => {
        set({
          loading: true,
          error: null,
        });

        try {
          await captureFuelCalibrationPoint(sessionId, request);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to capture fuel calibration point.",
          });
        }
      },

      pauseSession: async (sessionId) => {
        set({
          loading: true,
          error: null,
        });

        try {
          await pauseFuelCalibrationSession(sessionId);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to pause fuel calibration session.",
          });
        }
      },

      resumeSession: async (sessionId) => {
        set({
          loading: true,
          error: null,
        });

        try {
          await resumeFuelCalibrationSession(sessionId);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to resume fuel calibration session.",
          });
        }
      },

      applyAnchor: async (sessionId, request) => {
        set({
          loading: true,
          error: null,
        });

        try {
          await applyFuelCalibrationAnchor(sessionId, request);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to apply fuel calibration anchor.",
          });
        }
      },

      completeSession: async (sessionId) => {
        set({
          loading: true,
          error: null,
        });

        try {
          await completeFuelCalibrationSession(sessionId);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to complete fuel calibration session.",
          });
        }
      },

      abandonSession: async (sessionId) => {
        set({
          loading: true,
          error: null,
        });

        try {
          await abandonFuelCalibrationSession(sessionId);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to abandon fuel calibration session.",
          });
        }
      },

      publishProfile: async () => {
        const profile = get().profile;

        if (!profile) {
          set({
            error: "No fuel calibration profile is selected.",
          });

          return;
        }

        set({
          loading: true,
          error: null,
        });

        try {
          await publishFuelCalibrationProfile(profile.id);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to publish fuel calibration profile.",
          });
        }
      },

      activateProduction: async () => {
        const profile = get().profile;

        if (!profile) {
          set({
            error: "No fuel calibration profile is selected.",
          });

          return;
        }

        set({
          loading: true,
          error: null,
        });

        try {
          await activateFuelCalibrationProduction(profile.id);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to activate fuel calibration for production.",
          });
        }
      },

      supersedeProfile: async () => {
        const profile = get().profile;

        if (!profile) {
          set({
            error: "No fuel calibration profile is selected.",
          });

          return;
        }

        set({
          loading: true,
          error: null,
        });

        try {
          await supersedeFuelCalibrationProfile(profile.id);
          await reloadCurrentProfile();

          set({
            loading: false,
          });
        } catch {
          set({
            loading: false,
            error: "Failed to supersede fuel calibration profile.",
          });
        }
      },

      clearProfile: () => {
        set({
          profile: null,
          selectedSensorId: null,
          loading: false,
          error: null,
        });
      },

      clearError: () => {
        set({
          error: null,
        });
      },
    };
  },
);

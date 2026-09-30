import { useEffect, useRef, useState } from "react";

import { useDeviceStore } from "../store/deviceStore";
import { useFuelCalibrationStore } from "../store/fuelCalibrationStore";
import type { FuelCalibrationAutomaticCaptureResponse } from "../types/fuelCalibration";

export default function FuelCalibrationPanel() {
  const { selectedDevice, deviceSensors } = useDeviceStore();
  const [tankCapacityLitres, setTankCapacityLitres] = useState("");
  const [startingLitres, setStartingLitres] = useState("");
  const [startingQuantityKnown, setStartingQuantityKnown] = useState(false);
  const [cumulativeChangeLitres, setCumulativeChangeLitres] = useState("");
  /*
   * Automatic calibration capture is an interactive UI lifecycle.
   *
   * The backend remains authoritative for physical stability. The panel only
   * tracks whether automatic capture is currently running and displays the
   * latest stability evaluation returned by the backend.
   */
  const [automaticCaptureActive, setAutomaticCaptureActive] = useState(false);

  const [automaticCaptureResult, setAutomaticCaptureResult] =
    useState<FuelCalibrationAutomaticCaptureResponse | null>(null);

  /*
   * The polling timer is kept in a ref rather than application state because
   * it is an implementation detail of this component's lifecycle.
   */
  const automaticCaptureTimerRef = useRef<number | null>(null);
  const [supersedeConfirmationOpen, setSupersedeConfirmationOpen] =
    useState(false);

  const {
    profile,
    selectedSensorId,
    latestObservation,
    loading,
    error,
    loadProfile,
    loadLatestObservation,
    createProfile,
    startSession,
    capturePoint,
    pauseSession,
    resumeSession,
    abandonSession,
    applyAnchor,
    completeSession,
    publishProfile,
    activateProduction,
    supersedeProfile,
    clearProfile,
    clearError,
  } = useFuelCalibrationStore();

  /*
   * Fuel calibration belongs to the installed FUEL sensor rather than
   * directly to the device.
   *
   * The device store already loads the provisioned sensor instances for
   * the selected device, so this component only needs to identify the
   * installed FUEL sensor.
   */
  const fuelSensor =
    deviceSensors.find(
      (sensor) => sensor.sensor_type.toUpperCase() === "FUEL",
    ) ?? null;

  const currentSession =
    profile?.sessions.find(
      (session) => session.status === "active" || session.status === "paused",
    ) ?? null;

  const canStartSession =
    profile !== null &&
    profile.status !== "production" &&
    profile.status !== "superseded" &&
    currentSession === null;

  const calibrationStep = (() => {
    /*
     * Step 1:
     * No guided calibration profile exists yet.
     *
     * The installer must first provide the physical tank capacity.
     */
    if (!profile) {
      return "profile";
    }

    /*
     * Terminal lifecycle states.
     */
    if (profile.status === "production") {
      return "production";
    }

    if (profile.status === "superseded") {
      return "superseded";
    }

    /*
     * Once a runtime calibration has been published, the only remaining
     * normal workflow step is explicit production approval.
     */
    if (
      profile.status === "validated" &&
      profile.published_calibration_id !== null
    ) {
      return "production-approval";
    }

    /*
     * An unfinished session takes precedence over historical completed
     * sessions because this is the work the installer must continue.
     */
    if (currentSession) {
      if (currentSession.status === "paused") {
        return "paused";
      }

      if (currentSession.points.length === 0) {
        return "capture-initial-point";
      }

      if (currentSession.anchor_absolute_litres === null) {
        return "capture-points";
      }

      return "review-session";
    }

    /*
     * Completed resolved evidence means the profile can now be converted
     * into the runtime lookup table.
     */
    const hasPublishableSession = profile.sessions.some(
      (session) =>
        session.status === "completed" &&
        session.points.length >= 2 &&
        session.points.every((point) => point.resolved_litres !== null),
    );

    if (hasPublishableSession && profile.published_calibration_id === null) {
      return "publish";
    }

    /*
     * Otherwise the profile exists but there is currently no unfinished
     * session, so the next physical operation is to start one.
     */
    return "start-session";
  })();

  const calibrationStage = (() => {
    switch (calibrationStep) {
      case "profile":
        return 1;

      case "start-session":
      case "capture-initial-point":
      case "capture-points":
      case "paused":
      case "review-session":
        return 2;

      case "publish":
      case "production-approval":
        return 3;

      case "production":
        return 4;

      case "superseded":
        return 1;
    }
  })();

  /*
   * Whenever the selected device's FUEL sensor changes, load the guided
   * calibration profile belonging to that sensor.
   *
   * If no device or no FUEL sensor is available, clear calibration state
   * so a previous device's profile cannot remain visible.
   */
  useEffect(() => {
    if (!selectedDevice || !fuelSensor) {
      clearProfile();
      return;
    }

    if (selectedSensorId !== fuelSensor.id) {
      void loadProfile(fuelSensor.id);
    }
  }, [selectedDevice, fuelSensor, selectedSensorId, loadProfile, clearProfile]);

  useEffect(() => {
    if (!fuelSensor || selectedSensorId !== fuelSensor.id) {
      return;
    }

    /*
     * Load the newest physical KUM observation immediately when the
     * calibration panel becomes ready for the selected fuel sensor.
     */
    void loadLatestObservation(fuelSensor.id);

    /*
     * While this panel remains open, refresh the physical KUM observation
     * every 5 seconds.
     *
     * The firmware uses a 5-second reporting interval while calibration
     * mode is enabled, so matching that cadence keeps the installer-facing
     * calibration measurement close to the physical sensor without
     * continuously hammering the backend.
     *
     * This polling is observational only. Calibration capture remains
     * backend-authoritative: the capture endpoint independently resolves
     * the latest physical fuel observation when a point is captured.
     */
    const observationPollingInterval = window.setInterval(() => {
      void loadLatestObservation(fuelSensor.id);
    }, 5_000);

    /*
     * Stop polling when:
     *
     * - the component unmounts;
     * - another device/sensor is selected; or
     * - this effect is otherwise recreated.
     *
     * This prevents stale calibration polling from continuing in the
     * background for a previously selected sensor.
     */
    return () => {
      window.clearInterval(observationPollingInterval);
    };
  }, [fuelSensor, selectedSensorId, loadLatestObservation]);

  /*
   * Automatic capture polling must never survive this component.
   *
   * Later, when the automatic-capture loop is wired in, this cleanup guarantees
   * that navigating away from Platform Management cannot leave an old timer
   * evaluating a calibration session in the background.
   */
  useEffect(() => {
    return () => {
      if (automaticCaptureTimerRef.current !== null) {
        window.clearTimeout(automaticCaptureTimerRef.current);
        automaticCaptureTimerRef.current = null;
      }
    };
  }, []);

  const handleCreateProfile = async () => {
    if (!fuelSensor) {
      return;
    }

    const capacity = Number(tankCapacityLitres);

    if (!Number.isFinite(capacity) || capacity <= 0) {
      return;
    }

    await createProfile(fuelSensor.id, {
      tank_capacity_litres: capacity,
    });

    setTankCapacityLitres("");
  };

  const handleStartSession = async () => {
    if (!profile || !canStartSession) {
      return;
    }

    if (!startingQuantityKnown) {
      await startSession({
        starting_litres: null,
      });

      setStartingLitres("");
      return;
    }

    const litres = Number(startingLitres);

    if (
      !Number.isFinite(litres) ||
      litres < 0 ||
      litres > profile.tank_capacity_litres
    ) {
      return;
    }

    await startSession({
      starting_litres: litres,
    });

    setStartingLitres("");
  };

  const handleCapturePoint = async () => {
    if (
      !currentSession ||
      currentSession.status !== "active" ||
      automaticCaptureActive
    ) {
      return;
    }

    const cumulativeChange = Number(cumulativeChangeLitres);

    if (!Number.isFinite(cumulativeChange)) {
      return;
    }

    /*
     * Capture the identifiers and requested calibration position at the start
     * of this automatic-capture attempt.
     *
     * The polling loop must continue evaluating exactly this session and
     * cumulative fuel-change position even while React subsequently rerenders.
     */
    const sessionId = currentSession.id;
    const requestedCumulativeChange = cumulativeChange;

    /*
     * Establish the physical-observation boundary once for this entire
     * automatic-capture attempt.
     *
     * Every poll must reuse this exact timestamp so the backend can accumulate
     * only KUM observations produced after the installer pressed
     * "Start Automatic Capture".
     */
    const observationStartedAt = new Date().toISOString();

    setAutomaticCaptureActive(true);
    setAutomaticCaptureResult(null);

    /*
     * Each iteration performs exactly one backend stability evaluation.
     *
     * The backend decides whether the recent physical KUM evidence is stable.
     * The frontend never calculates or overrides that decision.
     */
    const evaluateCapture = async (): Promise<void> => {
      const result = await capturePoint(sessionId, {
        cumulative_change_litres: requestedCumulativeChange,
        observation_started_at: observationStartedAt,
      });

      /*
       * A failed request terminates this automatic-capture attempt.
       *
       * The store has already populated its user-facing error state.
       */
      if (!result) {
        setAutomaticCaptureActive(false);
        automaticCaptureTimerRef.current = null;
        return;
      }

      setAutomaticCaptureResult(result);

      /*
       * Stable evidence has already been persisted by the backend.
       *
       * capturePoint() reloads the authoritative profile when captured=true,
       * so there is nothing further for this polling loop to persist.
       */
      if (result.captured) {
        setAutomaticCaptureActive(false);
        automaticCaptureTimerRef.current = null;
        setCumulativeChangeLitres("");
        return;
      }

      /*
       * The physical measurement is not ready yet.
       *
       * Wait before asking the backend to evaluate the latest observation
       * window again. setTimeout is used rather than setInterval so requests
       * can never overlap.
       */
      automaticCaptureTimerRef.current = window.setTimeout(() => {
        void evaluateCapture();
      }, 5_000);
    };

    await evaluateCapture();
  };

  const handlePauseSession = async () => {
    if (!currentSession || currentSession.status !== "active") {
      return;
    }

    await pauseSession(currentSession.id);
  };

  const handleResumeSession = async () => {
    if (!currentSession || currentSession.status !== "paused") {
      return;
    }

    await resumeSession(currentSession.id);
  };

  const handleAbandonSession = async () => {
    if (!currentSession) {
      return;
    }

    if (
      currentSession.status !== "active" &&
      currentSession.status !== "paused"
    ) {
      return;
    }

    await abandonSession(currentSession.id);
  };

  const handleEstablishFullTankAnchor = async () => {
    if (!profile || !currentSession || currentSession.points.length === 0) {
      return;
    }

    if (
      currentSession.status !== "active" &&
      currentSession.status !== "paused"
    ) {
      return;
    }

    /*
     * The installer has physically confirmed that the tank is FULL.
     *
     * The latest captured KUM observation therefore represents the
     * full-tank position.
     *
     * Because the tank capacity is already known by the calibration
     * profile, that capacity becomes the absolute quantity for the
     * latest captured point.
     *
     * Example:
     *
     * tank capacity             = 200 L
     * latest cumulative change  = +60 L
     *
     * anchor:
     *     +60 L = 200 L
     *
     * The backend then resolves the previous relative calibration
     * observations backwards into absolute litre quantities.
     */
    const finalPoint = currentSession.points[currentSession.points.length - 1];

    await applyAnchor(currentSession.id, {
      cumulative_change_litres: finalPoint.cumulative_change_litres,
      absolute_litres: profile.tank_capacity_litres,
    });
  };

  const handleCompleteSession = async () => {
    if (!currentSession) {
      return;
    }

    if (
      currentSession.status !== "active" &&
      currentSession.status !== "paused"
    ) {
      return;
    }

    if (
      currentSession.points.length < 2 ||
      currentSession.starting_litres === null ||
      currentSession.ending_litres === null ||
      currentSession.points.some((point) => point.resolved_litres === null)
    ) {
      return;
    }

    await completeSession(currentSession.id);
  };

  const handlePublishProfile = async () => {
    if (!profile) {
      return;
    }

    if (
      profile.status === "superseded" ||
      profile.status === "production" ||
      profile.published_calibration_id !== null
    ) {
      return;
    }

    const hasCompletedSession = profile.sessions.some(
      (session) =>
        session.status === "completed" &&
        session.points.length >= 2 &&
        session.points.every((point) => point.resolved_litres !== null),
    );

    if (!hasCompletedSession) {
      return;
    }

    await publishProfile();
  };

  const handleActivateProduction = async () => {
    if (!profile) {
      return;
    }

    if (
      profile.status !== "validated" ||
      profile.confidence === "low" ||
      profile.published_calibration_id === null
    ) {
      return;
    }

    await activateProduction();
  };

  const handleSupersedeProfile = async () => {
    if (!profile || profile.status === "superseded") {
      return;
    }

    const hasUnfinishedSession = profile.sessions.some(
      (session) => session.status === "active" || session.status === "paused",
    );

    if (hasUnfinishedSession) {
      return;
    }

    await supersedeProfile();

    setSupersedeConfirmationOpen(false);
  };

  if (!selectedDevice) {
    return (
      <section className="platform-panel">
        <div className="platform-panel__header">
          <div>
            <span>Fuel Calibration</span>
            <h2>No device selected</h2>
          </div>
        </div>

        <p className="platform-detail-text">
          Select a provisioned device to manage its fuel-sensor calibration.
        </p>
      </section>
    );
  }

  if (!fuelSensor) {
    return (
      <section className="platform-panel">
        <div className="platform-panel__header">
          <div>
            <span>Fuel Calibration</span>
            <h2>Fuel sensor unavailable</h2>
          </div>
        </div>

        <p className="platform-detail-text">
          {selectedDevice.device_code} does not have a provisioned FUEL sensor,
          so guided fuel calibration is not available for this device.
        </p>
      </section>
    );
  }

  return (
    <section className="platform-panel">
      <div className="platform-panel__header">
        <div>
          <span>Fuel Calibration</span>
          <h2>{selectedDevice.device_code}</h2>
        </div>

        {profile && <strong>{profile.status}</strong>}
      </div>

      <div className="platform-detail-grid">
        <div>
          <label>Sensor</label>
          <strong>{fuelSensor.sensor_code}</strong>
        </div>

        <div>
          <label>Unit</label>
          <strong>{fuelSensor.unit}</strong>
        </div>
      </div>

      <div className="platform-calibration-workflow">
        <div
          className={
            calibrationStage === 1
              ? "platform-calibration-workflow__step platform-calibration-workflow__step--active"
              : calibrationStage > 1
                ? "platform-calibration-workflow__step platform-calibration-workflow__step--complete"
                : "platform-calibration-workflow__step"
          }
        >
          <span>1</span>
          <strong>Tank Setup</strong>
        </div>

        <div
          className={
            calibrationStage === 2
              ? "platform-calibration-workflow__step platform-calibration-workflow__step--active"
              : calibrationStage > 2
                ? "platform-calibration-workflow__step platform-calibration-workflow__step--complete"
                : "platform-calibration-workflow__step"
          }
        >
          <span>2</span>
          <strong>Physical Calibration</strong>
        </div>

        <div
          className={
            calibrationStage === 3
              ? "platform-calibration-workflow__step platform-calibration-workflow__step--active"
              : calibrationStage > 3
                ? "platform-calibration-workflow__step platform-calibration-workflow__step--complete"
                : "platform-calibration-workflow__step"
          }
        >
          <span>3</span>
          <strong>Validate</strong>
        </div>

        <div
          className={
            calibrationStage === 4
              ? "platform-calibration-workflow__step platform-calibration-workflow__step--active"
              : "platform-calibration-workflow__step"
          }
        >
          <span>4</span>
          <strong>Production</strong>
        </div>
      </div>

      <p className="platform-calibration-current-step">
        Current step:{" "}
        <strong>
          {calibrationStep === "profile" && "Enter Tank Capacity"}
          {calibrationStep === "start-session" && "Start Calibration Session"}
          {calibrationStep === "capture-initial-point" &&
            "Capture Initial Tank Reading"}
          {calibrationStep === "capture-points" &&
            "Add Fuel and Capture Readings"}
          {calibrationStep === "paused" && "Calibration Paused"}
          {calibrationStep === "review-session" &&
            "Review Resolved Calibration"}
          {calibrationStep === "publish" && "Publish Calibration"}
          {calibrationStep === "production-approval" &&
            "Approve for Production"}
          {calibrationStep === "production" && "Production Calibration Active"}
          {calibrationStep === "superseded" && "Calibration Superseded"}
        </strong>
      </p>

      {loading && (
        <p className="platform-detail-text">
          Loading fuel calibration profile...
        </p>
      )}

      {error && (
        <div className="platform-detail-section">
          <p className="platform-detail-text">{error}</p>

          <button
            type="button"
            className="platform-primary-button"
            onClick={clearError}
          >
            Dismiss
          </button>
        </div>
      )}

      {!loading && !error && !profile && (
        <div className="platform-detail-section">
          <label>Calibration State</label>

          <p className="platform-detail-text">
            This fuel sensor does not yet have a guided calibration profile.
            Enter the known tank capacity to begin calibration.
          </p>

          <div className="platform-form">
            <label>
              Tank Capacity (litres)
              <input
                type="number"
                min="0"
                step="0.1"
                inputMode="decimal"
                placeholder="e.g. 80"
                value={tankCapacityLitres}
                onChange={(event) => setTankCapacityLitres(event.target.value)}
              />
            </label>

            <button
              type="button"
              className="platform-primary-button"
              disabled={
                !Number.isFinite(Number(tankCapacityLitres)) ||
                Number(tankCapacityLitres) <= 0
              }
              onClick={() => void handleCreateProfile()}
            >
              Create Calibration Profile
            </button>
          </div>
        </div>
      )}

      {!loading && !error && profile && (
        <>
          <div className="platform-detail-section">
            <label>Calibration Profile</label>

            <div className="platform-detail-grid">
              <div>
                <label>Status</label>
                <strong>{profile.status}</strong>
              </div>

              <div>
                <label>Confidence</label>
                <strong>{profile.confidence}</strong>
              </div>

              <div>
                <label>Tank Capacity</label>
                <strong>{profile.tank_capacity_litres} L</strong>
              </div>

              <div>
                <label>Coverage</label>
                <strong>{profile.coverage_percentage.toFixed(1)}%</strong>
              </div>
            </div>
          </div>

          <div className="platform-detail-section">
            <label>Latest Physical KUM Observation</label>

            {latestObservation ? (
              <div className="platform-detail-grid">
                <div>
                  <label>Real-Time Distance</label>
                  <strong>
                    {latestObservation.realtime_distance_cm.toFixed(2)} cm
                  </strong>
                </div>

                <div>
                  <label>Smooth Distance</label>
                  <strong>
                    {latestObservation.smooth_distance_cm.toFixed(2)} cm
                  </strong>
                </div>

                <div>
                  <label>Raw Distance</label>
                  <strong>
                    {latestObservation.raw_distance_cm.toFixed(2)} cm
                  </strong>
                </div>

                <div>
                  <label>Recorded At</label>
                  <strong>
                    {new Date(latestObservation.recorded_at).toLocaleString()}
                  </strong>
                </div>
              </div>
            ) : (
              <p className="platform-detail-text">
                No physical KUM observation has been received for this fuel
                sensor yet.
              </p>
            )}
          </div>

          <div className="platform-detail-section">
            <label>Verified Range</label>

            <p className="platform-detail-text">
              {profile.verified_from_litres} L – {profile.verified_to_litres} L
            </p>
          </div>

          <div className="platform-detail-section">
            <label>Calibration Sessions</label>

            <p className="platform-detail-text">
              {profile.sessions.length} guided calibration session
              {profile.sessions.length === 1 ? "" : "s"} recorded.
            </p>
          </div>

          {canStartSession && (
            <div className="platform-detail-section">
              <label>Start Calibration Session</label>

              <p className="platform-detail-text">
                Start a physical calibration session for this tank. The current
                fuel quantity may be supplied when it is independently known, or
                left unknown and established later using an absolute calibration
                anchor.
              </p>

              <div className="platform-form">
                <label>
                  <span>
                    <input
                      type="checkbox"
                      checked={startingQuantityKnown}
                      onChange={(event) => {
                        setStartingQuantityKnown(event.target.checked);

                        if (!event.target.checked) {
                          setStartingLitres("");
                        }
                      }}
                    />
                    Starting fuel quantity is known
                  </span>
                </label>

                {startingQuantityKnown && (
                  <label>
                    Starting Fuel Quantity (litres)
                    <input
                      type="number"
                      min="0"
                      max={profile.tank_capacity_litres}
                      step="0.1"
                      inputMode="decimal"
                      placeholder={`0 – ${profile.tank_capacity_litres}`}
                      value={startingLitres}
                      onChange={(event) =>
                        setStartingLitres(event.target.value)
                      }
                    />
                  </label>
                )}

                <button
                  type="button"
                  className="platform-primary-button"
                  disabled={
                    startingQuantityKnown &&
                    (!Number.isFinite(Number(startingLitres)) ||
                      startingLitres.trim() === "" ||
                      Number(startingLitres) < 0 ||
                      Number(startingLitres) > profile.tank_capacity_litres)
                  }
                  onClick={() => void handleStartSession()}
                >
                  Start Calibration Session
                </button>
              </div>
            </div>
          )}

          {currentSession && (
            <div className="platform-detail-section">
              <label>Current Session</label>

              <div className="platform-detail-grid">
                <div>
                  <label>Status</label>
                  <strong>{currentSession.status}</strong>
                </div>

                <div>
                  <label>Starting Quantity</label>
                  <strong>
                    {currentSession.starting_litres === null
                      ? "Unknown"
                      : `${currentSession.starting_litres} L`}
                  </strong>
                </div>

                <div>
                  <label>Captured Points</label>
                  <strong>{currentSession.points.length}</strong>
                </div>

                <div>
                  <label>Anchor</label>
                  <strong>
                    {currentSession.anchor_absolute_litres === null
                      ? "Not established"
                      : `${currentSession.anchor_absolute_litres} L`}
                  </strong>
                </div>
              </div>

              <div className="platform-calibration-session-actions">
                {currentSession.status === "active" && (
                  <button
                    type="button"
                    className="platform-primary-button"
                    disabled={loading}
                    onClick={() => void handlePauseSession()}
                  >
                    Pause Calibration Session
                  </button>
                )}

                {currentSession.status === "paused" && (
                  <button
                    type="button"
                    className="platform-primary-button"
                    disabled={loading}
                    onClick={() => void handleResumeSession()}
                  >
                    Resume Calibration Session
                  </button>
                )}

                {(currentSession.status === "active" ||
                  currentSession.status === "paused") && (
                  <button
                    type="button"
                    className="platform-primary-button"
                    disabled={loading}
                    onClick={() => void handleAbandonSession()}
                  >
                    Abandon Calibration Session
                  </button>
                )}

                {currentSession.points.length > 0 &&
                  currentSession.anchor_absolute_litres === null && (
                    <button
                      type="button"
                      className="platform-primary-button"
                      disabled={loading}
                      onClick={() => void handleEstablishFullTankAnchor()}
                    >
                      Tank Is Full — Establish Full-Tank Anchor
                    </button>
                  )}

                {currentSession.points.length >= 2 &&
                  currentSession.starting_litres !== null &&
                  currentSession.ending_litres !== null &&
                  currentSession.points.every(
                    (point) => point.resolved_litres !== null,
                  ) && (
                    <button
                      type="button"
                      className="platform-primary-button"
                      disabled={loading}
                      onClick={() => void handleCompleteSession()}
                    >
                      Complete Calibration Session
                    </button>
                  )}
              </div>
            </div>
          )}

          {currentSession?.status === "active" && (
            <div className="platform-detail-section">
              <label>Capture Calibration Point</label>

              {automaticCaptureResult && (
                <div
                  className={`platform-auto-capture ${
                    automaticCaptureResult.captured
                      ? "platform-auto-capture--complete"
                      : "platform-auto-capture--measuring"
                  }`}
                >
                  <div className="platform-auto-capture__visual">
                    {automaticCaptureResult.captured ? (
                      <div
                        className="platform-auto-capture__success"
                        aria-hidden="true"
                      >
                        ✓
                      </div>
                    ) : (
                      <div
                        className="platform-auto-capture__spinner"
                        aria-hidden="true"
                      >
                        <span />
                      </div>
                    )}

                    <span className="platform-auto-capture__eyebrow">
                      ORBI Auto Calibration
                    </span>

                    <strong className="platform-auto-capture__title">
                      {automaticCaptureResult.captured
                        ? "Calibration Point Captured"
                        : automaticCaptureResult.state ===
                            "waiting_for_telemetry"
                          ? "Waiting for Fuel Sensor"
                          : automaticCaptureResult.state === "observing"
                            ? "Measuring Fuel Level"
                            : automaticCaptureResult.state === "settling"
                              ? "Fuel Level Is Settling"
                              : "Checking Fuel Stability"}
                    </strong>

                    <div className="platform-auto-capture__reading">
                      {automaticCaptureResult.captured
                        ? (automaticCaptureResult.capture_distance_cm?.toFixed(
                            2,
                          ) ?? "—")
                        : (latestObservation?.realtime_distance_cm.toFixed(2) ??
                          "—")}

                      <span>cm</span>
                    </div>

                    <p className="platform-auto-capture__message">
                      {automaticCaptureResult.captured
                        ? "ORBI detected a stable fuel level and recorded this calibration point automatically."
                        : "Keep the tank and sensor still. ORBI is continuously measuring the physical fuel level and will capture the point automatically when the reading is stable."}
                    </p>
                  </div>

                  <div className="platform-auto-capture__telemetry">
                    <div>
                      <span>Samples</span>
                      <strong>{automaticCaptureResult.sample_count}</strong>
                    </div>

                    <div>
                      <span>Observing</span>
                      <strong>
                        {automaticCaptureResult.observation_duration_seconds.toFixed(
                          0,
                        )}{" "}
                        s
                      </strong>
                    </div>

                    <div>
                      <span>Variation</span>
                      <strong>
                        {automaticCaptureResult.realtime_range_cm === null
                          ? "—"
                          : `${automaticCaptureResult.realtime_range_cm.toFixed(3)} cm`}
                      </strong>
                    </div>

                    <div>
                      <span>Trend</span>
                      <strong>
                        {automaticCaptureResult.realtime_slope_cm_per_second ===
                        null
                          ? "—"
                          : `${automaticCaptureResult.realtime_slope_cm_per_second.toFixed(
                              4,
                            )} cm/s`}
                      </strong>
                    </div>
                  </div>

                  {!automaticCaptureResult.captured && (
                    <div className="platform-auto-capture__activity">
                      <span />
                      <span />
                      <span />
                      <span />
                      <span />
                    </div>
                  )}
                </div>
              )}

              <div className="platform-form">
                <label>
                  Cumulative Fuel Change (litres)
                  <input
                    type="number"
                    step="0.1"
                    inputMode="decimal"
                    placeholder="e.g. 0, 10, or -10"
                    value={cumulativeChangeLitres}
                    onChange={(event) =>
                      setCumulativeChangeLitres(event.target.value)
                    }
                  />
                </label>

                <button
                  type="button"
                  className="platform-primary-button"
                  disabled={
                    automaticCaptureActive ||
                    !latestObservation ||
                    cumulativeChangeLitres.trim() === "" ||
                    !Number.isFinite(Number(cumulativeChangeLitres))
                  }
                  onClick={() => void handleCapturePoint()}
                >
                  {automaticCaptureActive
                    ? "Waiting for Stable Fuel Level..."
                    : "Start Automatic Capture"}
                </button>
              </div>
            </div>
          )}

          {currentSession && (
            <div className="platform-detail-section">
              <label>Captured Calibration Points</label>

              {currentSession.points.length === 0 ? (
                <p className="platform-detail-text">
                  No calibration points have been captured in this session yet.
                </p>
              ) : (
                <div className="platform-calibration-points">
                  {currentSession.points.map((point, index) => (
                    <div key={point.id} className="platform-calibration-point">
                      <div className="platform-calibration-point__header">
                        <strong>Point {index + 1}</strong>

                        <span>
                          {new Date(point.captured_at).toLocaleString()}
                        </span>
                      </div>

                      <div className="platform-calibration-point__values">
                        <div>
                          <label>KUM Level</label>
                          <strong>{point.level_cm} cm</strong>
                        </div>

                        <div>
                          <label>Cumulative Change</label>
                          <strong>
                            {point.cumulative_change_litres > 0 ? "+" : ""}
                            {point.cumulative_change_litres} L
                          </strong>
                        </div>

                        <div>
                          <label>Resolved Quantity</label>
                          <strong>
                            {point.resolved_litres === null
                              ? "Unresolved"
                              : `${point.resolved_litres} L`}
                          </strong>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {profile.published_calibration_id === null &&
            profile.status !== "superseded" &&
            profile.status !== "production" &&
            profile.sessions.some(
              (session) =>
                session.status === "completed" &&
                session.points.length >= 2 &&
                session.points.every((point) => point.resolved_litres !== null),
            ) && (
              <div className="platform-detail-section">
                <label>Runtime Calibration</label>

                <p className="platform-detail-text">
                  The completed calibration evidence can now be validated and
                  published as a runtime fuel calibration. Publishing does not
                  activate it for live telemetry.
                </p>

                <button
                  type="button"
                  className="platform-primary-button"
                  disabled={loading}
                  onClick={() => void handlePublishProfile()}
                >
                  Publish Calibration
                </button>
              </div>
            )}

          {profile.status === "validated" &&
            profile.confidence !== "low" &&
            profile.published_calibration_id !== null && (
              <div className="platform-detail-section">
                <label>Production Approval</label>

                <p className="platform-detail-text">
                  This calibration has been validated and published. Approving
                  it for production will make it the active fuel calibration
                  used by live telemetry for this sensor.
                </p>

                <button
                  type="button"
                  className="platform-primary-button"
                  disabled={loading}
                  onClick={() => void handleActivateProduction()}
                >
                  Approve for Production
                </button>
              </div>
            )}

          {profile.status !== "superseded" &&
            !profile.sessions.some(
              (session) =>
                session.status === "active" || session.status === "paused",
            ) && (
              <div className="platform-detail-section">
                <label>Recalibration</label>

                <p className="platform-detail-text">
                  Retire this guided calibration profile when a new physical
                  calibration needs to be started. If this profile currently
                  supplies the production calibration, that runtime calibration
                  will remain active until a replacement is approved for
                  production.
                </p>

                <button
                  type="button"
                  className="platform-primary-button"
                  disabled={loading}
                  onClick={() => setSupersedeConfirmationOpen(true)}
                >
                  Supersede Calibration Profile
                </button>

                {supersedeConfirmationOpen && (
                  <div className="platform-confirmation-panel">
                    <div>
                      <strong>Supersede this calibration?</strong>

                      <p className="platform-detail-text">
                        This will retire the current guided calibration profile.
                        Its historical calibration data will be preserved, but a
                        new calibration profile must be created before a
                        replacement physical calibration can be established.
                      </p>

                      {profile.status === "production" && (
                        <p className="platform-detail-text">
                          The calibration currently used at runtime will remain
                          active until a replacement calibration is approved for
                          production.
                        </p>
                      )}
                    </div>

                    <div className="platform-confirmation-actions">
                      <button
                        type="button"
                        className="platform-secondary-button"
                        disabled={loading}
                        onClick={() => setSupersedeConfirmationOpen(false)}
                      >
                        Cancel
                      </button>

                      <button
                        type="button"
                        className="platform-danger-button"
                        disabled={loading}
                        onClick={() => void handleSupersedeProfile()}
                      >
                        Confirm Supersede
                      </button>
                    </div>
                  </div>
                )}
              </div>
            )}
        </>
      )}
    </section>
  );
}

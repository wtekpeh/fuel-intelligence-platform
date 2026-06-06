import { useState } from "react";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useFleetStore } from "../../store/fleetStore";
import { fetchTelemetryHistory } from "../../api/telemetryApi";

function ReplayControls() {
  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);

  const isPlaying = useMapReplayStore((state) => state.isPlaying);

  const playbackSpeedMs = useMapReplayStore((state) => state.playbackSpeedMs);

  const startReplay = useMapReplayStore((state) => state.startReplay);

  const stopReplay = useMapReplayStore((state) => state.stopReplay);

  const play = useMapReplayStore((state) => state.play);

  const pause = useMapReplayStore((state) => state.pause);

  const setPlaybackSpeedMs = useMapReplayStore(
    (state) => state.setPlaybackSpeedMs,
  );

  const setReplayReadings = useMapReplayStore(
    (state) => state.setReplayReadings,
  );

  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const [startDate, setStartDate] = useState("");
  const [startTime, setStartTime] = useState("");

  const [endDate, setEndDate] = useState("");
  const [endTime, setEndTime] = useState("");

  const loadTodayRoute = async () => {
    if (!selectedDevice) {
      return;
    }

    const startTime = new Date();
    startTime.setHours(0, 0, 0, 0);

    const endTime = new Date();

    const readings = await fetchTelemetryHistory(
      selectedDevice.device_id,
      startTime.toISOString(),
      endTime.toISOString(),
    );

    setReplayReadings(readings);
  };

  const loadYesterdayRoute = async () => {
    if (!selectedDevice) {
      return;
    }

    const startTime = new Date();
    startTime.setDate(startTime.getDate() - 1);
    startTime.setHours(0, 0, 0, 0);

    const endTime = new Date(startTime);
    endTime.setHours(23, 59, 59, 999);

    const readings = await fetchTelemetryHistory(
      selectedDevice.device_id,
      startTime.toISOString(),
      endTime.toISOString(),
    );

    setReplayReadings(readings);
  };

  const loadLast7DaysRoute = async () => {
    if (!selectedDevice) {
      return;
    }

    const startTime = new Date();
    startTime.setDate(startTime.getDate() - 7);
    startTime.setHours(0, 0, 0, 0);

    const endTime = new Date();

    const readings = await fetchTelemetryHistory(
      selectedDevice.device_id,
      startTime.toISOString(),
      endTime.toISOString(),
    );

    setReplayReadings(readings);
  };

  const loadCustomRangeRoute = async () => {
    if (!selectedDevice) {
      return;
    }

    if (!startDate || !startTime || !endDate || !endTime) {
      return;
    }

    const startDateTime = new Date(`${startDate}T${startTime}`);

    const endDateTime = new Date(`${endDate}T${endTime}`);

    const readings = await fetchTelemetryHistory(
      selectedDevice.device_id,
      startDateTime.toISOString(),
      endDateTime.toISOString(),
    );

    setReplayReadings(readings);
  };

  return (
    <div className="replay-controls">
      {!isReplayMode ? (
        <>
          <div className="replay-controls__group">
            <span className="replay-controls__label">Quick Ranges</span>

            <button type="button" onClick={loadTodayRoute}>
              Today
            </button>

            <button type="button" onClick={loadYesterdayRoute}>
              Yesterday
            </button>

            <button type="button" onClick={loadLast7DaysRoute}>
              Last 7 Days
            </button>
          </div>

          <div className="replay-controls__field">
            <label>From Date</label>
            <input
              type="date"
              value={startDate}
              onChange={(event) => setStartDate(event.target.value)}
            />
          </div>

          <div className="replay-controls__field">
            <label>From Time</label>
            <input
              type="time"
              value={startTime}
              onChange={(event) => setStartTime(event.target.value)}
            />
          </div>

          <div className="replay-controls__field">
            <label>To Date</label>
            <input
              type="date"
              value={endDate}
              onChange={(event) => setEndDate(event.target.value)}
            />
          </div>

          <div className="replay-controls__field">
            <label>To Time</label>
            <input
              type="time"
              value={endTime}
              onChange={(event) => setEndTime(event.target.value)}
            />
          </div>

          <div className="replay-controls__group">
            <span className="replay-controls__label">Actions</span>

            <button type="button" onClick={loadCustomRangeRoute}>
              Load Range
            </button>

            <button type="button" onClick={startReplay}>
              Start Replay
            </button>
          </div>
        </>
      ) : (
        <>
          <button type="button" onClick={isPlaying ? pause : play}>
            {isPlaying ? "Pause" : "Play"}
          </button>

          <button type="button" onClick={stopReplay}>
            Stop
          </button>

          <select
            value={playbackSpeedMs}
            onChange={(event) => setPlaybackSpeedMs(Number(event.target.value))}
          >
            <option value={2000}>0.5x</option>
            <option value={1000}>1x</option>
            <option value={500}>2x</option>
            <option value={250}>4x</option>
          </select>
        </>
      )}
    </div>
  );
}

export default ReplayControls;

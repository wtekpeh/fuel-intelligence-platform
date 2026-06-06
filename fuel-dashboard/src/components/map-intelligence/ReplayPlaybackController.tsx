import { useEffect, useMemo } from "react";

import { useFleetStore } from "../../store/fleetStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useTelemetryStore } from "../../store/telemetryStore";
import { useInvestigationStore } from "../../store/investigationStore";
import type { FuelEvent } from "../../types";

function ReplayPlaybackController() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);
  const replayHistoryReadings = useMapReplayStore(
    (state) => state.replayReadings,
  );

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const isPlaying = useMapReplayStore((state) => state.isPlaying);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);
  const playbackSpeedMs = useMapReplayStore((state) => state.playbackSpeedMs);
  const setCurrentIndex = useMapReplayStore((state) => state.setCurrentIndex);
  const pause = useMapReplayStore((state) => state.pause);

  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const selectTimelineItem = useInvestigationStore(
    (state) => state.selectTimelineItem,
  );

  const setFocusedFuelEventId = useInvestigationStore(
    (state) => state.setFocusedFuelEventId,
  );

  const sourceReadings =
    replayHistoryReadings.length > 0 ? replayHistoryReadings : readings;

  const replayReadings = useMemo(() => {
    if (!selectedDevice) {
      return [];
    }

    return sourceReadings
      .filter(
        (reading) =>
          reading.device_id === selectedDevice.device_id &&
          reading.latitude !== null &&
          reading.longitude !== null,
      )
      .sort(
        (a, b) =>
          new Date(a.recorded_at).getTime() - new Date(b.recorded_at).getTime(),
      );
  }, [sourceReadings, selectedDevice]);

  useEffect(() => {
    if (!isReplayMode || !isPlaying) {
      return;
    }

    if (replayReadings.length < 2) {
      pause();
      return;
    }

    if (currentIndex >= replayReadings.length - 1) {
      pause();
      return;
    }

    const timeoutId = window.setTimeout(() => {
      setCurrentIndex(currentIndex + 1);
    }, playbackSpeedMs);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [
    currentIndex,
    isPlaying,
    isReplayMode,
    pause,
    playbackSpeedMs,
    replayReadings.length,
    setCurrentIndex,
  ]);

  useEffect(() => {
    if (!isReplayMode || replayReadings.length === 0) {
      return;
    }

    const replayReading = replayReadings[currentIndex];

    if (!replayReading) {
      return;
    }

    const replayTimestamp = new Date(replayReading.recorded_at).getTime();

    const matchedEvent = fuelEvents.find((event) => {
      const eventTimestamp = new Date(event.event_time).getTime();

      return Math.abs(eventTimestamp - replayTimestamp) <= 60_000;
    });

    if (!matchedEvent) {
      return;
    }

    //pause();

    setFocusedFuelEventId(matchedEvent.id);

    selectTimelineItem({
      id: `fuel-${matchedEvent.id}`,
      type: "fuel_event",
      title: matchedEvent.event_type,
      subtitle: matchedEvent.message,
      timestamp: matchedEvent.event_time,
      severity:
        matchedEvent.severity.toLowerCase() === "critical" ||
        matchedEvent.severity.toLowerCase() === "high"
          ? "danger"
          : matchedEvent.severity.toLowerCase() === "medium"
            ? "warning"
            : matchedEvent.severity.toLowerCase() === "low"
              ? "good"
              : "neutral",
      raw: matchedEvent as FuelEvent,
    });
  }, [
    currentIndex,
    fuelEvents,
    isReplayMode,
    replayReadings,
    selectTimelineItem,
    setFocusedFuelEventId,
    pause,
  ]);

  return null;
}

export default ReplayPlaybackController;

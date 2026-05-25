import { useEffect } from "react";

import { fetchRecentTelemetry } from "../api/telemetryApi";
import { useTelemetryStore } from "../store/telemetryStore";
import { useFleetStore } from "../store/fleetStore";

export function useTelemetryPolling() {
  const setReadings = useTelemetryStore((state) => state.setReadings);

  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  useEffect(() => {
    async function loadTelemetry() {
      try {
        const readings = await fetchRecentTelemetry(selectedDevice?.device_id);

        setReadings(readings);
      } catch (error) {
        console.error("[Telemetry] Polling failed.", error);
      }
    }

    loadTelemetry();

    const pollingTimer = window.setInterval(() => {
      loadTelemetry();
    }, 5000);

    return () => {
      window.clearInterval(pollingTimer);
    };
  }, [setReadings, selectedDevice?.device_id]);
}

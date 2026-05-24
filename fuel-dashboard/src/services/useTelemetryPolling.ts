import { useEffect } from "react";

import { fetchRecentTelemetry } from "../api/telemetryApi";
import { useTelemetryStore } from "../store/telemetryStore";

export function useTelemetryPolling() {
  const setReadings = useTelemetryStore((state) => state.setReadings);

  useEffect(() => {
    async function loadTelemetry() {
      try {
        const readings = await fetchRecentTelemetry();

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
  }, [setReadings]);
}

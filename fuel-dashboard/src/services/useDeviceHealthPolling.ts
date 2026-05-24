import { useEffect } from "react";

import { fetchDeviceHealthEvents } from "../api/deviceHealthApi";
import { useDeviceHealthStore } from "../store/deviceHealthStore";

export function useDeviceHealthPolling() {
  const setEvents = useDeviceHealthStore((state) => state.setEvents);

  useEffect(() => {
    async function loadDeviceHealth() {
      try {
        const events = await fetchDeviceHealthEvents();

        setEvents(events);
      } catch (error) {
        console.error("[DeviceHealth] Polling failed.", error);
      }
    }

    loadDeviceHealth();

    const pollingTimer = window.setInterval(() => {
      loadDeviceHealth();
    }, 10000);

    return () => {
      window.clearInterval(pollingTimer);
    };
  }, [setEvents]);
}

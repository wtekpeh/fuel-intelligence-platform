import { useEffect } from "react";

import { fetchDeviceHealthEvents } from "../api/deviceHealthApi";
import { useDeviceHealthStore } from "../store/deviceHealthStore";
import { useFleetStore } from "../store/fleetStore";

export function useDeviceHealthPolling() {
  const setEvents = useDeviceHealthStore((state) => state.setEvents);
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  useEffect(() => {
    async function loadDeviceHealth() {
      try {
        const events = await fetchDeviceHealthEvents(selectedDevice?.device_id);

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
  }, [setEvents, selectedDevice?.device_id]);
}

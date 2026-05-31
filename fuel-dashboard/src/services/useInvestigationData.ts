import { useEffect } from "react";

import {
  fetchDeviceStateEvents,
  fetchFuelEvents,
  fetchSensorHealthEvents,
} from "../api/investigationApi";

import { useFleetStore } from "../store/fleetStore";
import { useInvestigationStore } from "../store/investigationStore";

export function useInvestigationData() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const setFuelEvents = useInvestigationStore((state) => state.setFuelEvents);

  const setDeviceStateEvents = useInvestigationStore(
    (state) => state.setDeviceStateEvents,
  );

  const setSensorHealthEvents = useInvestigationStore(
    (state) => state.setSensorHealthEvents,
  );

  useEffect(() => {
    const selectedDeviceId = selectedDevice?.device_id;

    if (!selectedDeviceId) {
      return;
    }

    async function loadInvestigationData() {
      try {
        const [fuelEvents, deviceStateEvents, sensorHealthEvents] =
          await Promise.all([
            fetchFuelEvents(selectedDeviceId),
            fetchDeviceStateEvents(selectedDeviceId),
            fetchSensorHealthEvents(selectedDeviceId),
          ]);

        setFuelEvents(fuelEvents);
        setDeviceStateEvents(deviceStateEvents);
        setSensorHealthEvents(sensorHealthEvents);
      } catch (error) {
        console.error(
          "[Investigation] Failed to load investigation data.",
          error,
        );
      }
    }

    loadInvestigationData();
  }, [
    selectedDevice?.device_id,
    setFuelEvents,
    setDeviceStateEvents,
    setSensorHealthEvents,
  ]);
}

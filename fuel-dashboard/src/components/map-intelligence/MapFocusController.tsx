import { useEffect } from "react";
import { useMap } from "react-leaflet";

import { useFleetStore } from "../../store/fleetStore";
import { useInvestigationStore } from "../../store/investigationStore";
import { useTelemetryStore } from "../../store/telemetryStore";
import { useGeofenceDrawStore } from "../../store/geofenceDrawStore";
import type { FuelEvent } from "../../types";

function MapFocusController() {
  const map = useMap();

  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);
  const isDrawing = useGeofenceDrawStore((state) => state.isDrawing);

  const selectedTimelineItem = useInvestigationStore(
    (state) => state.selectedTimelineItem,
  );

  const selectedFuelEvent =
    selectedTimelineItem?.type === "fuel_event"
      ? (selectedTimelineItem.raw as FuelEvent)
      : null;

  useEffect(() => {
    if (
      isDrawing ||
      !selectedFuelEvent ||
      selectedFuelEvent.latitude === null ||
      selectedFuelEvent.longitude === null
    ) {
      return;
    }

    map.flyTo([selectedFuelEvent.latitude, selectedFuelEvent.longitude], 18, {
      duration: 1.2,
    });
  }, [isDrawing, selectedFuelEvent, map]);

  useEffect(() => {
    if (isDrawing || !selectedDevice || selectedFuelEvent) {
      return;
    }

    const selectedDeviceReading = readings.find(
      (reading) => reading.device_id === selectedDevice.device_id,
    );

    if (
      !selectedDeviceReading ||
      selectedDeviceReading.latitude === null ||
      selectedDeviceReading.longitude === null
    ) {
      return;
    }

    map.flyTo(
      [selectedDeviceReading.latitude, selectedDeviceReading.longitude],
      15,
      {
        duration: 1.2,
      },
    );
  }, [isDrawing, map, readings, selectedDevice, selectedFuelEvent]);

  return null;
}

export default MapFocusController;

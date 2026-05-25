import { useEffect } from "react";

import { fetchAlerts } from "../api/alertsApi";
import { useAlertStore } from "../store/alertStore";
import { useConnectionStore } from "../store/connectionStore";
import { useFleetStore } from "../store/fleetStore";
import { connectAlertsWebSocket } from "../websocket/alertsWebSocket";

export function useAlertsController() {
  const setAlerts = useAlertStore((state) => state.setAlerts);
  const addAlert = useAlertStore((state) => state.addAlert);
  const updateAlertLifecycle = useAlertStore(
    (state) => state.updateAlertLifecycle,
  );

  const setStatus = useConnectionStore((state) => state.setStatus);
  const setLastHeartbeatAt = useConnectionStore(
    (state) => state.setLastHeartbeatAt,
  );

  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  useEffect(() => {
    let socket: WebSocket | null = null;

    let reconnectTimer: number | null = null;

    let lastReceivedAt: string | undefined;

    let isUnmounted = false;

    const selectedDeviceId = selectedDevice?.device_id;

    async function connectDashboardSocket() {
      if (isUnmounted) {
        return;
      }

      setStatus("connecting");

      try {
        const alerts = await fetchAlerts(selectedDevice?.device_id);

        setAlerts(alerts);

        lastReceivedAt = alerts[0]?.created_at;

        socket = connectAlertsWebSocket({
          since: lastReceivedAt,

          onOpen: () => {
            setStatus("connected");
          },

          onMessage: (message) => {
            if (message.type === "heartbeat") {
              setLastHeartbeatAt(new Date().toISOString());

              return;
            }

            if (message.type === "recovery_alert") {
              if (
                selectedDeviceId &&
                message.data.device_id !== selectedDeviceId
              ) {
                return;
              }

              addAlert(message.data);

              lastReceivedAt = message.data.created_at;

              return;
            }

            if (message.type === "live_alert") {
              if (
                selectedDeviceId &&
                message.data.device_id !== selectedDeviceId
              ) {
                return;
              }

              addAlert(message.data);

              lastReceivedAt = message.data.created_at;

              return;
            }

            if (message.type === "alert_acknowledged") {
              updateAlertLifecycle(message.data);

              return;
            }
          },

          onClose: () => {
            setStatus("disconnected");

            if (isUnmounted) {
              return;
            }

            reconnectTimer = window.setTimeout(() => {
              connectDashboardSocket();
            }, 3000);
          },

          onError: () => {
            setStatus("error");
          },
        });
      } catch (error) {
        console.error("[Dashboard] Bootstrap failed.", error);

        setStatus("error");

        reconnectTimer = window.setTimeout(() => {
          connectDashboardSocket();
        }, 3000);
      }
    }

    connectDashboardSocket();

    return () => {
      isUnmounted = true;

      socket?.close();

      if (reconnectTimer) {
        window.clearTimeout(reconnectTimer);
      }
    };
  }, [
    addAlert,
    setAlerts,
    setLastHeartbeatAt,
    setStatus,
    updateAlertLifecycle,
    selectedDevice,
  ]);
}

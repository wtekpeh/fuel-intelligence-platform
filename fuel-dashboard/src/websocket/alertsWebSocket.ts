import type { AlertWebSocketMessage } from "../types";

interface ConnectAlertsWebSocketOptions {
  since?: string;

  onMessage: (message: AlertWebSocketMessage) => void;

  onOpen?: () => void;

  onClose?: () => void;

  onError?: () => void;
}

export function connectAlertsWebSocket(options: ConnectAlertsWebSocketOptions) {
  const websocketBaseUrl =
    import.meta.env.VITE_WS_BASE_URL ?? "ws://127.0.0.1:8080";

  const websocketUrl = new URL("/ws/alerts", websocketBaseUrl);

  if (options.since) {
    websocketUrl.searchParams.set("since", options.since);
  }

  const socket = new WebSocket(websocketUrl.toString());

  socket.onopen = () => {
    console.log("[WebSocket] Connected to alerts stream.");

    options.onOpen?.();
  };

  socket.onmessage = (event) => {
    try {
      const parsedMessage: AlertWebSocketMessage = JSON.parse(event.data);

      options.onMessage(parsedMessage);
    } catch (error) {
      console.error("[WebSocket] Failed to parse message.", error);
    }
  };

  socket.onerror = () => {
    console.error("[WebSocket] Connection error.");

    options.onError?.();
  };

  socket.onclose = () => {
    console.warn("[WebSocket] Connection closed.");

    options.onClose?.();
  };

  return socket;
}

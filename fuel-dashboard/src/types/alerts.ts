export type AlertStatus = "OPEN" | "ACKNOWLEDGED" | "RESOLVED";

export type AlertSeverity = "Info" | "Warning" | "Critical";

export type AlertType = "THEFT" | "REFILL" | "LEAK";

export interface AlertResponse {
  id: string;
  fuel_event_id: string | null;
  alert_type: AlertType;
  severity: AlertSeverity;
  reason: string;
  is_acknowledged: boolean;
  status: AlertStatus;
  created_at: string;
}

export interface AlertLifecycleResponse {
  id: string;
  alert_type: AlertType;
  severity: AlertSeverity;
  is_acknowledged: boolean;
  status: AlertStatus;
  created_at: string;
}

export type AlertWebSocketMessage =
  | {
      type: "recovery_alert";
      data: AlertResponse;
    }
  | {
      type: "live_alert";
      data: AlertResponse;
    }
  | {
      type: "alert_acknowledged";
      data: AlertLifecycleResponse;
    }
  | {
      type: "heartbeat";
      message: "alerts_ws_alive";
    };

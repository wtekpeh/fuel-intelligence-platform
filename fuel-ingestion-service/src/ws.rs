use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::broadcast::error::RecvError,
    time::{Duration, interval},
};

use crate::{
    models::AlertResponse, repository::list_alerts_since, routes::AppState,
    services::alert_hub::AlertHubMessage,
};

#[derive(Debug, Deserialize)]
pub struct AlertRecoveryQuery {
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum AlertWsMessage {
    #[serde(rename = "recovery_alert")]
    RecoveryAlert { data: AlertResponse },

    #[serde(rename = "live_alert")]
    LiveAlert { data: AlertResponse },

    #[serde(rename = "alert_acknowledged")]
    AlertAcknowledged {
        data: crate::models::AlertAcknowledgementResponse,
    },

    #[serde(rename = "heartbeat")]
    Heartbeat { message: String },
}

pub async fn alerts_ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    Query(query): Query<AlertRecoveryQuery>,
) -> impl IntoResponse {
    let receiver = app_state.alert_hub.subscribe();
    let db_pool = app_state.db_pool.clone();

    ws.on_upgrade(move |socket| handle_alert_socket(socket, receiver, db_pool, query.since))
}

async fn send_ws_message(socket: &mut WebSocket, message: AlertWsMessage) -> bool {
    let payload = match serde_json::to_string(&message) {
        Ok(json) => json,
        Err(_) => return true,
    };

    socket.send(Message::Text(payload)).await.is_ok()
}

async fn handle_alert_socket(
    mut socket: WebSocket,
    mut receiver: tokio::sync::broadcast::Receiver<AlertHubMessage>,
    db_pool: sqlx::PgPool,
    since: Option<chrono::DateTime<chrono::Utc>>,
) {
    if let Some(since_timestamp) = since {
        match list_alerts_since(&db_pool, since_timestamp).await {
            Ok(missed_alerts) => {
                for alert in missed_alerts {
                    let message = AlertWsMessage::RecoveryAlert { data: alert };

                    if !send_ws_message(&mut socket, message).await {
                        return;
                    }
                }
            }

            Err(_) => {
                return;
            }
        }
    }

    let mut heartbeat = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            alert_result = receiver.recv() => {
                match alert_result {
                    Ok(hub_message) => {
                        let message = match hub_message {
                            AlertHubMessage::LiveAlert(alert) => {
                                AlertWsMessage::LiveAlert { data: alert }
                            }

                            AlertHubMessage::AlertAcknowledged(acknowledgement) => {
                                AlertWsMessage::AlertAcknowledged {
                                    data: acknowledgement,
                                }
                            }
                        };

                        if !send_ws_message(&mut socket, message).await {
                            break;
                        }
                    }

                    Err(RecvError::Lagged(_)) => {
                        continue;
                    }

                    Err(RecvError::Closed) => {
                        break;
                    }
                }
            }

            _ = heartbeat.tick() => {
                let message = AlertWsMessage::Heartbeat {
                    message: "alerts_ws_alive".to_string(),
                };

                if !send_ws_message(&mut socket, message).await {
                    break;
                }
            }
        }
    }
}

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use serde::{Deserialize, Serialize};

use tokio::{
    sync::broadcast::error::RecvError,
    time::{Duration, interval},
};

use uuid::Uuid;

use crate::{
    models::{AlertResponse, OrbiUser},
    repository::{can_user_access_alert, consume_websocket_ticket, list_alerts_since},
    routes::AppState,
    services::alert_hub::AlertHubMessage,
};

#[derive(Debug, Deserialize)]
pub struct AlertRecoveryQuery {
    pub ticket: Uuid,
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
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    // Browsers send Origin during the WebSocket handshake.
    // Only explicitly trusted dashboard origins are accepted.
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;

    const ALLOWED_ORIGINS: &[&str] = &[
        "https://vite.williamtekpeh.com",
        "http://localhost:5173",
        "http://127.0.0.1:5173",
    ];

    if !ALLOWED_ORIGINS.contains(&origin) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Consume the ticket before upgrading the HTTP connection.
    let orbi_user = consume_websocket_ticket(&app_state.db_pool, query.ticket)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let receiver = app_state.alert_hub.subscribe();
    let db_pool = app_state.db_pool.clone();

    Ok(ws.on_upgrade(move |socket| {
        handle_alert_socket(socket, receiver, db_pool, query.since, orbi_user)
    }))
}

async fn send_ws_message(socket: &mut WebSocket, message: AlertWsMessage) -> bool {
    let payload = match serde_json::to_string(&message) {
        Ok(json) => json,
        Err(_) => return false,
    };

    socket.send(Message::Text(payload)).await.is_ok()
}

async fn handle_alert_socket(
    mut socket: WebSocket,
    mut receiver: tokio::sync::broadcast::Receiver<AlertHubMessage>,
    db_pool: sqlx::PgPool,
    since: Option<chrono::DateTime<chrono::Utc>>,
    orbi_user: OrbiUser,
) {
    // Historical recovery is tenant-filtered at the database layer.
    if let Some(since_timestamp) = since {
        match list_alerts_since(&db_pool, since_timestamp, &orbi_user).await {
            Ok(missed_alerts) => {
                for alert in missed_alerts {
                    // Recheck authorization immediately before delivery.
                    match can_user_access_alert(&db_pool, alert.id, &orbi_user).await {
                        Ok(true) => {}
                        Ok(false) => continue,
                        Err(_) => return,
                    }

                    if !send_ws_message(&mut socket, AlertWsMessage::RecoveryAlert { data: alert })
                        .await
                    {
                        return;
                    }
                }
            }

            Err(_) => return,
        }
    }

    let mut heartbeat = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            alert_result = receiver.recv() => {
                match alert_result {
                    Ok(hub_message) => {
                        let (alert_id, message) = match hub_message {
                            AlertHubMessage::LiveAlert(alert) => {
                                (
                                    alert.id,
                                    AlertWsMessage::LiveAlert {
                                        data: alert,
                                    },
                                )
                            }

                            AlertHubMessage::AlertAcknowledged(acknowledgement) => {
                                (
                                    acknowledgement.id,
                                    AlertWsMessage::AlertAcknowledged {
                                        data: acknowledgement,
                                    },
                                )
                            }
                        };

                        // Authorization is checked for every event.
                        match can_user_access_alert(
                            &db_pool,
                            alert_id,
                            &orbi_user,
                        )
                        .await
                        {
                            Ok(true) => {}

                            Ok(false) => continue,

                            // Never deliver an event if authorization
                            // cannot be established.
                            Err(_) => break,
                        }

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
                // Check that the user account remains active.
                // Membership authorization is independently checked
                // before every alert delivery.
                let active = sqlx::query_scalar!(
                    r#"
                    SELECT is_active
                    FROM orbi_users
                    WHERE id = $1
                    "#,
                    orbi_user.id
                )
                .fetch_optional(&db_pool)
                .await;

                match active {
                    Ok(Some(true)) => {}
                    _ => break,
                }

                if !send_ws_message(
                    &mut socket,
                    AlertWsMessage::Heartbeat {
                        message: "alerts_ws_alive".to_string(),
                    },
                )
                .await
                {
                    break;
                }
            }
        }
    }
}

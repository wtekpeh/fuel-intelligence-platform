use tokio::sync::broadcast;

use crate::models::{AlertAcknowledgementResponse, AlertResponse};

#[derive(Debug, Clone)]
pub enum AlertHubMessage {
    LiveAlert(AlertResponse),
    AlertAcknowledged(AlertAcknowledgementResponse),
}

#[derive(Clone)]
pub struct AlertHub {
    sender: broadcast::Sender<AlertHubMessage>,
}

impl AlertHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);

        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AlertHubMessage> {
        self.sender.subscribe()
    }

    pub fn broadcast_alert(&self, alert: AlertResponse) {
        let _ = self.sender.send(AlertHubMessage::LiveAlert(alert));
    }

    pub fn broadcast_acknowledgement(&self, acknowledgement: AlertAcknowledgementResponse) {
        let _ = self
            .sender
            .send(AlertHubMessage::AlertAcknowledged(acknowledgement));
    }
}

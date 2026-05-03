use chrono::Utc;

use crate::models::{FuelReading, ReadingBatch};

pub struct SyncQueue {
    device_id: String,
    pending_readings: Vec<FuelReading>,
    batch_size: usize,
}

impl SyncQueue {
    pub fn new(device_id: String, batch_size: usize) -> Self {
        Self {
            device_id,
            pending_readings: Vec::new(),
            batch_size,
        }
    }

    pub fn add_reading(&mut self, reading: FuelReading) {
        self.pending_readings.push(reading);
    }

    pub fn is_ready_to_sync(&self) -> bool {
        self.pending_readings.len() >= self.batch_size
    }

    pub fn create_batch(&mut self) -> ReadingBatch {
        let readings_to_send = self.pending_readings.clone();

        self.pending_readings.clear();

        ReadingBatch {
            device_id: self.device_id.clone(),
            synced_at: Utc::now(),
            readings: readings_to_send,
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending_readings.len()
    }
}

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

    pub fn build_batch(&self) -> ReadingBatch {
        ReadingBatch {
            device_id: self.device_id.clone(),
            synced_at: Utc::now(),
            readings: self.pending_readings.clone(),
        }
    }

    pub fn mark_synced(&mut self) {
        self.pending_readings.clear();
    }

    pub fn pending_count(&self) -> usize {
        self.pending_readings.len()
    }
}

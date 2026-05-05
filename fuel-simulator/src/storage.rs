use anyhow::Result;
use std::{
    fs::{OpenOptions, create_dir_all},
    io::Write,
    path::Path,
};

use crate::models::{FuelReading, ReadingBatch};

pub struct FileStorage {
    readings_file_path: String,
    batches_file_path: String,
}

impl FileStorage {
    pub fn new() -> Result<Self> {
        let data_dir = "data";

        if !Path::new(data_dir).exists() {
            create_dir_all(data_dir)?;
        }

        Ok(Self {
            readings_file_path: format!("{}/readings.jsonl", data_dir),
            batches_file_path: format!("{}/synced_batches.jsonl", data_dir),
        })
    }

    pub fn save_reading(&self, reading: &FuelReading) -> Result<()> {
        let json = serde_json::to_string(reading)?;
        append_line(&self.readings_file_path, &json)
    }

    pub fn save_synced_batch(&self, batch: &ReadingBatch) -> Result<()> {
        let json = serde_json::to_string(batch)?;
        append_line(&self.batches_file_path, &json)
    }
}

fn append_line(file_path: &str, line: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    writeln!(file, "{}", line)?;

    Ok(())
}

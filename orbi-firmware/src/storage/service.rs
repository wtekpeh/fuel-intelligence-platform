use core::fmt::Debug;

use embedded_sdmmc::{BlockDevice, Mode, TimeSource, VolumeIdx, VolumeManager};
use esp_println::println;

use crate::telemetry::record::TelemetryRecord;

use super::record::format_telemetry_record;

pub trait RecordStorage {
    fn append_record(&mut self, record: &TelemetryRecord<'_>) -> bool;
}

pub struct PersistentStorage<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    D::Error: Debug,
{
    volume_manager: VolumeManager<D, T>,
}

impl<D, T> PersistentStorage<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    D::Error: Debug,
{
    pub fn new(volume_manager: VolumeManager<D, T>) -> Self {
        Self { volume_manager }
    }
}

impl<D, T> RecordStorage for PersistentStorage<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    D::Error: Debug,
{
    fn append_record(&mut self, record: &TelemetryRecord<'_>) -> bool {
        let formatted_record = format_telemetry_record(record);

        let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => volume,
            Err(error) => {
                println!("Failed to open SD volume: {:?}", error);
                return false;
            }
        };

        let root_directory = match volume.open_root_dir() {
            Ok(directory) => directory,
            Err(error) => {
                println!("Failed to open SD root: {:?}", error);
                return false;
            }
        };

        let queue_file =
            match root_directory.open_file_in_dir("ORBIQUE.LOG", Mode::ReadWriteCreateOrAppend) {
                Ok(file) => file,
                Err(error) => {
                    println!("Failed to open ORBIQUE.LOG: {:?}", error);
                    return false;
                }
            };

        if let Err(error) = queue_file.write(formatted_record.as_bytes()) {
            println!("Failed to write telemetry to SD: {:?}", error);
            return false;
        }

        if let Err(error) = queue_file.flush() {
            println!("Failed to flush telemetry file: {:?}", error);
            return false;
        }

        println!("Telemetry appended to ORBIQUE.LOG successfully.");

        true
    }
}

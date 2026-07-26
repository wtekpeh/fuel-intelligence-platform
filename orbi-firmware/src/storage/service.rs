use core::fmt::Debug;

use embedded_sdmmc::{BlockDevice, Mode, TimeSource, VolumeIdx, VolumeManager};
use esp_println::println;

use crate::telemetry::record::TelemetryRecord;

use super::record::{format_gnss_diagnostic_record, format_telemetry_record, GnssDiagnosticRecord};

pub trait RecordStorage {
    fn append_record(&mut self, record: &TelemetryRecord<'_>) -> bool;

    fn append_ack(&mut self, device_id: &str, timestamp: &str) -> bool;

    /*
     * Read the first queued telemetry record.
     *
     * Retained for the existing single-record replay and cleanup paths.
     */
    fn read_first_record(&mut self) -> Option<heapless::String<512>>;

    /*
     * Read up to `max_records` complete telemetry records from the beginning
     * of ORBIQ.LOG.
     *
     * This operation is non-destructive. Records remain in the persistent
     * queue until a later acknowledged cleanup operation removes them.
     *
     * `N` is the maximum number of records that can be held in memory.
     * `max_records` allows the publishing policy to request fewer than `N`.
     */
    fn read_first_records<const N: usize>(
        &mut self,
        max_records: usize,
    ) -> heapless::Vec<heapless::String<768>, N>;

    fn is_acknowledged(&mut self, device_id: &str, timestamp: &str) -> bool;

    fn remove_first_record(&mut self) -> bool;

    fn append_gnss_diagnostic(&mut self, record: &GnssDiagnosticRecord<'_>) -> bool;
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
            match root_directory.open_file_in_dir("ORBIQ.LOG", Mode::ReadWriteCreateOrAppend) {
                Ok(file) => file,
                Err(error) => {
                    println!("Failed to open ORBIQ.LOG: {:?}", error);
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

        println!("Telemetry appended to ORBIQ.LOG successfully.");

        true
    }

    fn append_ack(&mut self, device_id: &str, timestamp: &str) -> bool {
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

        let ack_file =
            match root_directory.open_file_in_dir("ORBIACK.LOG", Mode::ReadWriteCreateOrAppend) {
                Ok(file) => file,

                Err(error) => {
                    println!("Failed to open ORBIACK.LOG: {:?}", error);
                    return false;
                }
            };

        let mut line = heapless::String::<128>::new();

        let _ = core::fmt::write(&mut line, format_args!("{},{}\r\n", device_id, timestamp));

        if let Err(error) = ack_file.write(line.as_bytes()) {
            println!("Failed to write ACK: {:?}", error);
            return false;
        }

        if let Err(error) = ack_file.flush() {
            println!("Failed to flush ACK: {:?}", error);
            return false;
        }

        println!("ACK appended to ORBIACK.LOG.");

        true
    }

    fn read_first_record(&mut self) -> Option<heapless::String<512>> {
        let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => volume,

            Err(error) => {
                println!("Failed to open SD volume: {:?}", error);
                return None;
            }
        };

        let root_directory = match volume.open_root_dir() {
            Ok(directory) => directory,

            Err(error) => {
                println!("Failed to open SD root: {:?}", error);
                return None;
            }
        };

        let queue_file = match root_directory.open_file_in_dir("ORBIQ.LOG", Mode::ReadOnly) {
            Ok(file) => file,

            Err(error) => {
                println!("Failed to open ORBIQ.LOG: {:?}", error);
                return None;
            }
        };

        let mut buffer = [0u8; 512];

        let bytes_read = match queue_file.read(&mut buffer) {
            Ok(count) => count,

            Err(error) => {
                println!("Failed to read ORBIQ.LOG: {:?}", error);
                return None;
            }
        };

        if bytes_read == 0 {
            println!("Queue is empty.");
            return None;
        }

        let mut record = heapless::String::<512>::new();

        for byte in &buffer[..bytes_read] {
            if *byte == b'\n' {
                break;
            }

            if *byte != b'\r' {
                let _ = record.push(*byte as char);
            }
        }

        println!("Read first queued record:");
        println!("{}", record);

        Some(record)
    }

    fn read_first_records<const N: usize>(
        &mut self,
        max_records: usize,
    ) -> heapless::Vec<heapless::String<768>, N> {
        let mut records = heapless::Vec::<heapless::String<768>, N>::new();

        /*
         * The compile-time capacity N protects memory usage, while max_records
         * lets the caller choose the batch size.
         */
        let target_records = core::cmp::min(max_records, N);

        if target_records == 0 {
            println!("Batch queue read requested zero records.");
            return records;
        }

        let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => volume,

            Err(error) => {
                println!("Failed to open SD volume for batch queue read: {:?}", error);
                return records;
            }
        };

        let root_directory = match volume.open_root_dir() {
            Ok(directory) => directory,

            Err(error) => {
                println!("Failed to open SD root for batch queue read: {:?}", error);
                return records;
            }
        };

        let queue_file = match root_directory.open_file_in_dir("ORBIQ.LOG", Mode::ReadOnly) {
            Ok(file) => file,

            Err(error) => {
                println!("Failed to open ORBIQ.LOG for batch queue read: {:?}", error);
                return records;
            }
        };

        /*
         * SD data is read in bounded chunks. A telemetry record may cross a
         * chunk boundary, so `current_record` remains alive between reads.
         */
        let mut buffer = [0u8; 512];
        let mut current_record = heapless::String::<768>::new();

        loop {
            let bytes_read = match queue_file.read(&mut buffer) {
                Ok(count) => count,

                Err(error) => {
                    println!("Failed while reading queued telemetry batch: {:?}", error);
                    return records;
                }
            };

            if bytes_read == 0 {
                break;
            }

            for byte in &buffer[..bytes_read] {
                /*
                 * Each ORBIQ.LOG entry is terminated by CRLF. The newline marks
                 * the end of one complete JSON record.
                 */
                if *byte == b'\n' {
                    if current_record.is_empty() {
                        continue;
                    }

                    let completed_record =
                        core::mem::replace(&mut current_record, heapless::String::<768>::new());

                    if records.push(completed_record).is_err() {
                        println!("In-memory telemetry batch capacity reached.");
                        return records;
                    }

                    if records.len() >= target_records {
                        println!(
                            "Read {} queued telemetry record(s) for batching.",
                            records.len()
                        );

                        return records;
                    }

                    continue;
                }

                /*
                 * Ignore the carriage-return component of CRLF.
                 */
                if *byte == b'\r' {
                    continue;
                }

                if current_record.push(*byte as char).is_err() {
                    println!("Queued telemetry record exceeded the 768-byte in-memory capacity.");

                    /*
                     * Never return an incomplete record. Any earlier complete
                     * records remain safe to use.
                     */
                    return records;
                }
            }
        }

        /*
         * Normally every stored record ends with CRLF. This also safely handles
         * a final complete-looking record if the file has no trailing newline.
         */
        if !current_record.is_empty()
            && records.len() < target_records
            && records.push(current_record).is_err()
        {
            println!("Unable to append final queued record to telemetry batch.");
        }

        if records.is_empty() {
            println!("Queue is empty.");
        } else {
            println!(
                "Read {} queued telemetry record(s) for batching.",
                records.len()
            );
        }

        records
    }

    fn is_acknowledged(&mut self, device_id: &str, timestamp: &str) -> bool {
        let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => volume,

            Err(error) => {
                println!("Failed to open SD volume for ACK lookup: {:?}", error);
                return false;
            }
        };

        let root_directory = match volume.open_root_dir() {
            Ok(directory) => directory,

            Err(error) => {
                println!("Failed to open SD root for ACK lookup: {:?}", error);
                return false;
            }
        };

        let ack_file = match root_directory.open_file_in_dir("ORBIACK.LOG", Mode::ReadOnly) {
            Ok(file) => file,

            Err(_) => {
                return false;
            }
        };

        let mut expected = heapless::String::<128>::new();

        if core::fmt::write(&mut expected, format_args!("{},{}", device_id, timestamp)).is_err() {
            println!("Failed to build ACK lookup value.");
            return false;
        }

        let expected_bytes = expected.as_bytes();

        let mut buffer = [0u8; 256];
        let mut matched_bytes = 0usize;

        loop {
            let bytes_read = match ack_file.read(&mut buffer) {
                Ok(count) => count,

                Err(error) => {
                    println!("Failed while reading ORBIACK.LOG: {:?}", error);
                    return false;
                }
            };

            if bytes_read == 0 {
                break;
            }

            for byte in &buffer[..bytes_read] {
                if *byte == expected_bytes[matched_bytes] {
                    matched_bytes += 1;

                    if matched_bytes == expected_bytes.len() {
                        return true;
                    }
                } else if *byte == expected_bytes[0] {
                    matched_bytes = 1;
                } else {
                    matched_bytes = 0;
                }
            }
        }

        false
    }

    fn remove_first_record(&mut self) -> bool {
        println!("Removing completed first record from ORBIQ.LOG...");

        // Stage 1:
        // Copy every byte after the first newline into ORBITMP.LOG.
        {
            let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
                Ok(volume) => volume,

                Err(error) => {
                    println!("Failed to open SD volume for cleanup: {:?}", error);
                    return false;
                }
            };

            let root_directory = match volume.open_root_dir() {
                Ok(directory) => directory,

                Err(error) => {
                    println!("Failed to open SD root for cleanup: {:?}", error);
                    return false;
                }
            };

            let queue_file = match root_directory.open_file_in_dir("ORBIQ.LOG", Mode::ReadOnly) {
                Ok(file) => file,

                Err(error) => {
                    println!("Failed to open ORBIQ.LOG for cleanup: {:?}", error);
                    return false;
                }
            };

            let temporary_file = match root_directory
                .open_file_in_dir("ORBITMP.LOG", Mode::ReadWriteCreateOrTruncate)
            {
                Ok(file) => file,

                Err(error) => {
                    println!("Failed to open ORBITMP.LOG: {:?}", error);
                    return false;
                }
            };

            let mut buffer = [0u8; 512];
            let mut first_line_finished = false;

            loop {
                let bytes_read = match queue_file.read(&mut buffer) {
                    Ok(count) => count,

                    Err(error) => {
                        println!("Failed while reading ORBIQ.LOG: {:?}", error);
                        return false;
                    }
                };

                if bytes_read == 0 {
                    break;
                }

                if first_line_finished {
                    if let Err(error) = temporary_file.write(&buffer[..bytes_read]) {
                        println!("Failed writing ORBITMP.LOG: {:?}", error);
                        return false;
                    }

                    continue;
                }

                if let Some(newline_position) =
                    buffer[..bytes_read].iter().position(|byte| *byte == b'\n')
                {
                    first_line_finished = true;

                    let remaining_start = newline_position + 1;

                    if remaining_start < bytes_read {
                        if let Err(error) =
                            temporary_file.write(&buffer[remaining_start..bytes_read])
                        {
                            println!("Failed writing remaining queue data: {:?}", error);
                            return false;
                        }
                    }
                }
            }

            if let Err(error) = temporary_file.flush() {
                println!("Failed to flush ORBITMP.LOG: {:?}", error);
                return false;
            }
        }

        // Stage 2:
        // Truncate ORBIQ.LOG and copy the remaining records back.
        {
            let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
                Ok(volume) => volume,

                Err(error) => {
                    println!("Failed to reopen SD volume: {:?}", error);
                    return false;
                }
            };

            let root_directory = match volume.open_root_dir() {
                Ok(directory) => directory,

                Err(error) => {
                    println!("Failed to reopen SD root: {:?}", error);
                    return false;
                }
            };

            let temporary_file =
                match root_directory.open_file_in_dir("ORBITMP.LOG", Mode::ReadOnly) {
                    Ok(file) => file,

                    Err(error) => {
                        println!("Failed to read ORBITMP.LOG: {:?}", error);
                        return false;
                    }
                };

            let queue_file = match root_directory
                .open_file_in_dir("ORBIQ.LOG", Mode::ReadWriteCreateOrTruncate)
            {
                Ok(file) => file,

                Err(error) => {
                    println!("Failed to rebuild ORBIQ.LOG: {:?}", error);
                    return false;
                }
            };

            let mut buffer = [0u8; 512];

            loop {
                let bytes_read = match temporary_file.read(&mut buffer) {
                    Ok(count) => count,

                    Err(error) => {
                        println!("Failed reading temporary queue: {:?}", error);
                        return false;
                    }
                };

                if bytes_read == 0 {
                    break;
                }

                if let Err(error) = queue_file.write(&buffer[..bytes_read]) {
                    println!("Failed rebuilding ORBIQ.LOG: {:?}", error);
                    return false;
                }
            }

            if let Err(error) = queue_file.flush() {
                println!("Failed to flush rebuilt ORBIQ.LOG: {:?}", error);
                return false;
            }
        }

        println!("First queued record removed successfully.");

        true
    }

    fn append_gnss_diagnostic(&mut self, record: &GnssDiagnosticRecord<'_>) -> bool {
        let formatted_record = format_gnss_diagnostic_record(record);

        let volume = match self.volume_manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => volume,

            Err(error) => {
                println!("Failed to open SD volume for GNSS diagnostics: {:?}", error);

                return false;
            }
        };

        let root_directory = match volume.open_root_dir() {
            Ok(directory) => directory,

            Err(error) => {
                println!("Failed to open SD root for GNSS diagnostics: {:?}", error);

                return false;
            }
        };

        let diagnostic_file =
            match root_directory.open_file_in_dir("ORBIGNSS.LOG", Mode::ReadWriteCreateOrAppend) {
                Ok(file) => file,

                Err(error) => {
                    println!("Failed to open ORBIGNSS.LOG: {:?}", error);

                    return false;
                }
            };

        if let Err(error) = diagnostic_file.write(formatted_record.as_bytes()) {
            println!("Failed to write GNSS diagnostic record: {:?}", error);

            return false;
        }

        if let Err(error) = diagnostic_file.flush() {
            println!("Failed to flush ORBIGNSS.LOG: {:?}", error);

            return false;
        }

        println!("GNSS diagnostic appended to ORBIGNSS.LOG.");

        true
    }
}

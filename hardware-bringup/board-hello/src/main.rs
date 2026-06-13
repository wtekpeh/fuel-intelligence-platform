#![no_std]
#![no_main]

use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::master::{Config, Spi};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

struct DummyTimeSource;

impl TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    println!("SD write test starting...");

    let mut peripheral_power = Output::new(peripherals.GPIO12, Level::Low, OutputConfig::default());

    peripheral_power.set_high();
    delay.delay_millis(1000);

    let spi = Spi::new(peripherals.SPI2, Config::default())
        .unwrap()
        .with_sck(peripherals.GPIO14)
        .with_miso(peripherals.GPIO2)
        .with_mosi(peripherals.GPIO15);

    let sd_cs = Output::new(peripherals.GPIO13, Level::High, OutputConfig::default());

    let spi_device = ExclusiveDevice::new(spi, sd_cs, delay).unwrap();

    let sdcard = SdCard::new(spi_device, delay);

    match sdcard.num_bytes() {
        Ok(size) => println!("SD detected. Size: {} bytes", size),
        Err(_) => println!("SD detect failed."),
    }

    let volume_manager = VolumeManager::new(sdcard, DummyTimeSource);

    match volume_manager.open_volume(VolumeIdx(0)) {
        Ok(volume0) => {
            println!("Volume opened.");

            match volume0.open_root_dir() {
                Ok(root_dir) => {
                    println!("Root directory opened.");

                    match root_dir.open_file_in_dir("FUEL_LOG.TXT", Mode::ReadWriteCreateOrAppend) {
                        Ok(mut file) => {
                            println!("File opened.");

                            let _ = file.write(b"Fuel Intelligence Platform SD write test\r\n");
                            let _ = file.write(b"Board: LILYGO T-A7670E R2\r\n");
                            let _ = file.write(b"Status: SD logging works\r\n");
                            let _ = file.flush();

                            println!("Write complete.");
                        }
                        Err(_) => println!("Failed to open file."),
                    }
                }
                Err(_) => println!("Failed to open root directory."),
            }
        }
        Err(_) => println!("Failed to open volume."),
    }

    loop {
        println!("SD write test alive...");
        delay.delay_millis(3000);
    }
}

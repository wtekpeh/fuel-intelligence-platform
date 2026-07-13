use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    peripherals::{GPIO13, GPIO14, GPIO15, GPIO2, SPI2},
    spi::{
        self,
        master::{Config as SpiConfig, Spi},
    },
    time::Rate,
};
use esp_println::println;

use super::{PersistentStorage, RecordStorage};

#[derive(Default)]
struct OrbiTimeSource;

impl TimeSource for OrbiTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 6,
            zero_indexed_day: 12,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

pub fn initialize<'d>(
    spi2: SPI2<'d>,
    miso: GPIO2<'d>,
    mosi: GPIO15<'d>,
    sclk: GPIO14<'d>,
    cs: GPIO13<'d>,
) -> Option<impl RecordStorage + 'd> {
    println!("========================");
    println!("ORBI STORAGE");
    println!("========================");
    println!("Initializing persistent SD storage...");

    let spi_bus = match Spi::new(
        spi2,
        SpiConfig::default()
            .with_frequency(Rate::from_khz(400))
            .with_mode(spi::Mode::_0),
    ) {
        Ok(spi) => spi.with_sck(sclk).with_mosi(mosi).with_miso(miso),

        Err(error) => {
            println!("SD SPI initialization failed: {:?}", error);
            return None;
        }
    };

    let sd_cs = Output::new(cs, Level::High, OutputConfig::default());

    let spi_device = match ExclusiveDevice::new(spi_bus, sd_cs, Delay::new()) {
        Ok(device) => device,

        Err(error) => {
            println!("SD SPI device creation failed: {:?}", error);
            return None;
        }
    };

    let sd_card = SdCard::new(spi_device, Delay::new());

    let size_bytes = match sd_card.num_bytes() {
        Ok(size) => size,

        Err(error) => {
            println!("SD card detection failed: {:?}", error);
            return None;
        }
    };

    println!("SD card detected successfully.");
    println!("SD card capacity: {} MB", size_bytes / 1_048_576);

    let volume_manager = VolumeManager::new(sd_card, OrbiTimeSource);

    {
        let volume = match volume_manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => volume,

            Err(error) => {
                println!("Failed to open FAT volume: {:?}", error);
                return None;
            }
        };

        let root_check = volume.open_root_dir();

        match root_check {
            Ok(_) => {
                println!("Persistent SD filesystem ready.");
            }

            Err(error) => {
                println!("Failed to open SD root: {:?}", error);
                return None;
            }
        };
    }

    Some(PersistentStorage::new(volume_manager))
}

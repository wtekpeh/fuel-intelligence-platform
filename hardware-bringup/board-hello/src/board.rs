use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO12, GPIO4, GPIO5};

pub struct BoardPins<'d> {
    pub modem_power_on: Output<'d>,
    pub modem_reset: Output<'d>,
    pub modem_pwrkey: Output<'d>,
}

impl<'d> BoardPins<'d> {
    pub fn new(gpio12: GPIO12<'d>, gpio5: GPIO5<'d>, gpio4: GPIO4<'d>) -> Self {
        let modem_power_on = Output::new(gpio12, Level::Low, OutputConfig::default());
        let modem_reset = Output::new(gpio5, Level::Low, OutputConfig::default());
        let modem_pwrkey = Output::new(gpio4, Level::Low, OutputConfig::default());

        Self {
            modem_power_on,
            modem_reset,
            modem_pwrkey,
        }
    }
}

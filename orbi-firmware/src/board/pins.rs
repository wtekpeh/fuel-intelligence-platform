use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO12, GPIO4, GPIO5};
use esp_println::println;

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

    /// Enables the shared LilyGO peripheral power rail.
    ///
    /// On the T-A7670, GPIO12 supplies power to both:
    ///
    /// - the A7670 modem;
    /// - the onboard microSD card circuit.
    ///
    /// This must therefore happen before either the modem or SD card
    /// is initialized, especially when the board is powered by battery.
    pub fn enable_peripheral_power(&mut self, delay: &Delay) {
        println!("GPIO12 HIGH: enabling shared modem and SD power rail...");

        self.modem_power_on.set_high();

        /*
         * Allow the shared power rail and SD card supply to stabilise
         * before SPI communication begins.
         */
        delay.delay_millis(1_000);

        println!("Shared peripheral power rail is ready.");
    }
}

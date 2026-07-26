use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::peripherals::{GPIO26, GPIO27, UART1};
use esp_hal::uart::{Config, Uart};
use esp_println::{print, println};

const RESPONSE_BUFFER_SIZE: usize = 256;

pub struct Modem<'d> {
    pub uart: Uart<'d, esp_hal::Blocking>,
}

impl<'d> Modem<'d> {
    pub fn new(uart1: UART1<'d>, modem_tx_gpio26: GPIO26<'d>, modem_rx_gpio27: GPIO27<'d>) -> Self {
        let uart = Uart::new(uart1, Config::default())
            .unwrap()
            .with_tx(modem_tx_gpio26)
            .with_rx(modem_rx_gpio27);

        Self { uart }
    }

    pub fn power_on(
        modem_power_on: &mut Output<'d>,
        modem_reset: &mut Output<'d>,
        modem_pwrkey: &mut Output<'d>,
        delay: &Delay,
    ) {
        println!("GPIO12 HIGH: enabling board peripheral power...");
        modem_power_on.set_high();
        delay.delay_millis(1000);

        println!("GPIO5 LOW: modem reset inactive based on LilyGO example...");
        modem_reset.set_low();
        delay.delay_millis(500);

        println!("GPIO4 pulse: modem power key...");
        modem_pwrkey.set_low();
        delay.delay_millis(100);
        modem_pwrkey.set_high();
        delay.delay_millis(100);
        modem_pwrkey.set_low();

        println!("Waiting 10 seconds for modem boot...");
        delay.delay_millis(10000);
    }

    pub fn send_command(&mut self, command: &[u8], label: &str) -> bool {
        match self.uart.write(command) {
            Ok(_) => {
                println!("Sent: {}", label);
                true
            }

            Err(_) => {
                println!("Failed to send: {}", label);
                false
            }
        }
    }

    pub fn read_response(&mut self) -> Option<([u8; RESPONSE_BUFFER_SIZE], usize)> {
        if !self.uart.read_ready() {
            return None;
        }

        let mut buffer = [0u8; RESPONSE_BUFFER_SIZE];

        match self.uart.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => Some((buffer, bytes_read)),

            Ok(_) => None,

            Err(_) => {
                println!("Failed to read modem UART response.");
                None
            }
        }
    }

    fn collect_response(
        &mut self,
        command: &[u8],
        label: &str,
        delay: &Delay,
    ) -> Option<([u8; RESPONSE_BUFFER_SIZE], usize)> {
        if !self.send_command(command, label) {
            return None;
        }

        delay.delay_millis(1000);

        self.read_response()
    }

    fn print_response(buffer: &[u8], bytes_read: usize) {
        for byte in buffer.iter().take(bytes_read) {
            if *byte >= 32 && *byte <= 126 {
                print!("{}", *byte as char);
            } else if *byte == b'\r' {
                print!("\\r");
            } else if *byte == b'\n' {
                println!("\\n");
            } else {
                print!("[{}]", *byte);
            }
        }

        println!();
    }

    pub fn send_command_and_print_response(&mut self, command: &[u8], label: &str, delay: &Delay) {
        match self.collect_response(command, label, delay) {
            Some((buffer, bytes_read)) => {
                println!("Read {} byte(s):", bytes_read);
                Self::print_response(&buffer, bytes_read);
            }

            None => {
                println!("No response yet.");
            }
        }
    }

    pub fn send_command_and_collect_response(
        &mut self,
        command: &[u8],
        label: &str,
        delay: &Delay,
    ) -> Option<([u8; RESPONSE_BUFFER_SIZE], usize)> {
        match self.collect_response(command, label, delay) {
            Some((buffer, bytes_read)) => {
                println!("Collected {} byte(s)", bytes_read);
                Some((buffer, bytes_read))
            }

            None => {
                println!("No response received.");
                None
            }
        }
    }
}

use esp_hal::delay::Delay;
use esp_hal::peripherals::{GPIO21, GPIO22, UART2};
use esp_hal::uart::{Config, Uart};
use esp_println::{print, println};

const RESPONSE_BUFFER_SIZE: usize = 64;

/// Confirmed KUM Modbus RTU query from the successful Python test.
///
/// Slave address:  0x01
/// Function:       0x03
/// Start register: 0x0000
/// Register count: 0x0008
/// CRC:            0x0C44, transmitted low byte first
const READ_LEVEL_REQUEST: [u8; 8] = [0x01, 0x03, 0x00, 0x00, 0x00, 0x08, 0x44, 0x0C];

/// Low-level KUM sensor communication driver.
///
/// UART2 is routed as follows:
///
/// - UART2 TX -> GPIO21 -> XY-485 RXD
/// - UART2 RX <- GPIO22 <- XY-485 TXD
pub struct KumSensor<'d> {
    uart: Uart<'d, esp_hal::Blocking>,
}

impl<'d> KumSensor<'d> {
    /// Creates the KUM UART connection at the sensor's confirmed
    /// serial configuration: 9600 baud, 8 data bits, no parity,
    /// one stop bit.
    pub fn new(uart2: UART2<'d>, kum_tx_gpio21: GPIO21<'d>, kum_rx_gpio22: GPIO22<'d>) -> Self {
        let config = Config::default().with_baudrate(9_600);

        let uart = Uart::new(uart2, config)
            .unwrap()
            .with_tx(kum_tx_gpio21)
            .with_rx(kum_rx_gpio22);

        Self { uart }
    }

    /// Sends the proven Modbus query and prints the raw hexadecimal
    /// response without interpreting it.
    pub fn query_and_print_raw_response(&mut self, delay: &Delay) {
        println!("========================");
        println!("KUM MODBUS RAW TEST");
        println!("========================");

        match self.uart.write(&READ_LEVEL_REQUEST) {
            Ok(_) => {
                println!("KUM request sent:");
                Self::print_hex(&READ_LEVEL_REQUEST);
            }

            Err(_) => {
                println!("ERROR: Failed to write KUM Modbus request.");
                return;
            }
        }

        /*
         * The working Python test waited before reading the response.
         * One second is intentionally generous for the first bring-up.
         */
        delay.delay_millis(1_000);

        if !self.uart.read_ready() {
            println!("No KUM response is currently available.");
            return;
        }

        let mut response = [0u8; RESPONSE_BUFFER_SIZE];

        match self.uart.read(&mut response) {
            Ok(bytes_read) if bytes_read > 0 => {
                println!("Received {} KUM byte(s):", bytes_read);
                Self::print_hex(&response[..bytes_read]);
            }

            Ok(_) => {
                println!("KUM UART read returned zero bytes.");
            }

            Err(_) => {
                println!("ERROR: Failed to read KUM UART response.");
            }
        }
    }

    fn print_hex(bytes: &[u8]) {
        for byte in bytes {
            print!("{:02X} ", byte);
        }

        println!();
    }
}

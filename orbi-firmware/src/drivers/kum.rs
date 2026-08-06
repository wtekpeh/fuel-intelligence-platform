use esp_hal::delay::Delay;
use esp_hal::peripherals::{GPIO21, GPIO22, UART2};
use esp_hal::uart::{Config, Uart};
use esp_println::{print, println};

const RESPONSE_BUFFER_SIZE: usize = 64;

const EXPECTED_SLAVE_ADDRESS: u8 = 0x01;
const READ_HOLDING_REGISTERS_FUNCTION: u8 = 0x03;
const EXPECTED_REGISTER_DATA_BYTES: u8 = 0x10;

/// Confirmed KUM Modbus RTU request.
///
/// - Slave address: 0x01
/// - Function: 0x03
/// - Start register: 0x0000
/// - Register count: 0x0008
/// - CRC: 0x0C44, transmitted low byte first
const READ_LEVEL_REQUEST: [u8; 8] = [
    0x01, // Slave address
    0x03, // Read holding registers
    0x00, // Start register high byte
    0x00, // Start register low byte
    0x00, // Register count high byte
    0x08, // Register count low byte
    0x44, // CRC low byte
    0x0C, // CRC high byte
];

/// Physical values decoded from one KUM response.
///
/// The distance values represent the gap between the ultrasonic
/// sensor and the detected liquid surface.
///
/// They are not yet:
///
/// - fuel height;
/// - fuel volume in litres;
/// - tank percentage.
#[derive(Debug, Clone, Copy)]
pub struct KumMeasurement {
    pub smooth_distance_cm: f32,
    pub realtime_distance_cm: f32,
    pub raw_distance_cm: f32,

    pub status_byte_1: u8,
    pub raw_data_validity: u8,
    pub status_byte_2: u8,

    pub temperature_c: f32,
}

/// Errors that may occur while reading or decoding the KUM sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KumReadError {
    WriteFailed,
    NoResponse,
    ReadFailed,
    ResponseTooShort,
    UnexpectedSlaveAddress,
    UnexpectedFunction,
    UnexpectedByteCount,
}

/// Low-level KUM ultrasonic fuel sensor driver.
///
/// Confirmed wiring for the current ORBI reference hardware:
///
/// - UART2 TX on GPIO21 -> XY-485 TXD
/// - UART2 RX on GPIO22 <- XY-485 RXD
///
/// The current XY-485 module was physically validated using
/// straight-through TXD/RXD labels rather than the usual crossover.
pub struct KumSensor<'d> {
    uart: Uart<'d, esp_hal::Blocking>,
}

impl<'d> KumSensor<'d> {
    /// Creates the KUM UART connection using the sensor's confirmed
    /// serial configuration:
    ///
    /// - 9600 baud
    /// - 8 data bits
    /// - no parity
    /// - 1 stop bit
    pub fn new(uart2: UART2<'d>, kum_tx_gpio21: GPIO21<'d>, kum_rx_gpio22: GPIO22<'d>) -> Self {
        let config = Config::default().with_baudrate(9_600);

        let uart = Uart::new(uart2, config)
            .unwrap()
            .with_tx(kum_tx_gpio21)
            .with_rx(kum_rx_gpio22);

        Self { uart }
    }

    /// Sends one Modbus request and returns a decoded physical
    /// measurement from the KUM sensor.
    pub fn read_measurement(&mut self, delay: &Delay) -> Result<KumMeasurement, KumReadError> {
        self.uart
            .write(&READ_LEVEL_REQUEST)
            .map_err(|_| KumReadError::WriteFailed)?;

        /*
         * The working Python bring-up waited one second before reading.
         *
         * This remains intentionally generous during initial firmware
         * integration. It can be reduced after repeated hardware tests.
         */
        delay.delay_millis(1_000);

        if !self.uart.read_ready() {
            return Err(KumReadError::NoResponse);
        }

        let mut response = [0u8; RESPONSE_BUFFER_SIZE];

        let bytes_read = self
            .uart
            .read(&mut response)
            .map_err(|_| KumReadError::ReadFailed)?;

        decode_level_response(&response[..bytes_read])
    }

    /// Temporary hardware-validation helper.
    ///
    /// This reads one measurement and prints both the decoded values
    /// and any error returned by the driver.
    pub fn read_and_print_measurement(&mut self, delay: &Delay) {
        println!("========================");
        println!("KUM FUEL MEASUREMENT TEST");
        println!("========================");

        println!("Sending KUM request:");
        Self::print_hex(&READ_LEVEL_REQUEST);

        match self.read_measurement(delay) {
            Ok(measurement) => {
                println!("KUM measurement decoded successfully.");
                println!("Smooth distance: {} cm", measurement.smooth_distance_cm);
                println!(
                    "Real-time distance: {} cm",
                    measurement.realtime_distance_cm
                );
                println!("Raw distance: {} cm", measurement.raw_distance_cm);

                println!("Status byte 1: 0x{:02X}", measurement.status_byte_1);
                println!("Raw data validity: 0x{:02X}", measurement.raw_data_validity);
                println!("Status byte 2: 0x{:02X}", measurement.status_byte_2);

                println!("Temperature: {} C", measurement.temperature_c);
            }

            Err(error) => {
                println!("KUM measurement failed: {:?}", error);
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

/// Decodes one unsigned 16-bit big-endian register value.
fn read_u16_be(response: &[u8], start_index: usize) -> Result<u16, KumReadError> {
    let high_byte = *response
        .get(start_index)
        .ok_or(KumReadError::ResponseTooShort)?;

    let low_byte = *response
        .get(start_index + 1)
        .ok_or(KumReadError::ResponseTooShort)?;

    Ok((u16::from(high_byte) << 8) | u16::from(low_byte))
}

/// Decodes the confirmed 21-byte KUM Modbus response.
///
/// Confirmed response layout:
///
/// ```text
/// byte 0      slave address
/// byte 1      function
/// byte 2      register-data byte count
/// bytes 3–4   smooth distance
/// bytes 5–6   real-time distance
/// bytes 7–8   raw distance
/// byte 9      status byte 1
/// byte 13     raw-data validity
/// byte 14     status byte 2
/// byte 15     temperature
/// bytes 19–20 response CRC
/// ```
///
/// Distance registers are expressed in hundredths of a centimetre.
fn decode_level_response(response: &[u8]) -> Result<KumMeasurement, KumReadError> {
    if response.len() < 21 {
        return Err(KumReadError::ResponseTooShort);
    }

    if response[0] != EXPECTED_SLAVE_ADDRESS {
        return Err(KumReadError::UnexpectedSlaveAddress);
    }

    if response[1] != READ_HOLDING_REGISTERS_FUNCTION {
        return Err(KumReadError::UnexpectedFunction);
    }

    if response[2] != EXPECTED_REGISTER_DATA_BYTES {
        return Err(KumReadError::UnexpectedByteCount);
    }

    let smooth_raw = read_u16_be(response, 3)?;
    let realtime_raw = read_u16_be(response, 5)?;
    let raw_raw = read_u16_be(response, 7)?;

    Ok(KumMeasurement {
        smooth_distance_cm: smooth_raw as f32 / 100.0,
        realtime_distance_cm: realtime_raw as f32 / 100.0,
        raw_distance_cm: raw_raw as f32 / 100.0,

        status_byte_1: response[9],
        raw_data_validity: response[13],
        status_byte_2: response[14],

        temperature_c: response[15] as f32,
    })
}

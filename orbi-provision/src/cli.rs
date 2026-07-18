use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "orbi-provision",
    version,
    about = "ORBI device manufacturing and provisioning utility"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate and verify a legacy Version 1 identity record.
    GenerateV1 {
        /// Permanent physical identity assigned to the device.
        #[arg(long)]
        device_code: String,

        /// Path where the identity record will be written.
        #[arg(long, default_value = "identity-v1.bin")]
        output: String,
    },

    /// Generate and verify an authenticated Version 2 identity record.
    GenerateV2 {
        /// Permanent physical identity assigned to the device.
        #[arg(long)]
        device_code: String,

        /// Binary manufacturing key used for HMAC-SHA-256.
        #[arg(long)]
        key_file: String,

        /// Path where the identity record will be written.
        #[arg(long, default_value = "identity-v2.bin")]
        output: String,
    },

    /// Provision a blank connected device with a Version 2 identity.
    ProvisionV2 {
        /// Serial port connected to the ESP32.
        #[arg(long)]
        port: String,

        /// Permanent physical identity assigned to the device.
        #[arg(long)]
        device_code: String,

        /// Binary manufacturing key used for HMAC-SHA-256.
        #[arg(long)]
        key_file: String,

        /// espflash executable to invoke.
        #[arg(long, default_value = "espflash.exe")]
        espflash: String,
    },

    /// Read and decode the identity stored on a connected ORBI device.
    Read {
        /// Serial port connected to the ESP32.
        #[arg(long)]
        port: String,

        /// File used to store the 64-byte flash read-back.
        #[arg(long, default_value = "board-identity.bin")]
        output: String,

        /// espflash executable to invoke.
        #[arg(long, default_value = "espflash.exe")]
        espflash: String,
    },
}

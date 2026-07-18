mod cli;
mod commands;
mod flash;
mod identity;
mod services;

use std::process;

use clap::Parser;

use cli::{Cli, Commands};
use flash::espflash::EspflashProvider;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::GenerateV1 {
            device_code,
            output,
        } => commands::generate::execute_v1(&device_code, &output),

        Commands::GenerateV2 {
            device_code,
            key_file,
            output,
        } => commands::generate::execute_v2(&device_code, &key_file, &output),

        Commands::ProvisionV2 {
            port,
            device_code,
            key_file,
            espflash,
        } => {
            let flash_provider = EspflashProvider::new(espflash);

            commands::provision::execute_v2(&flash_provider, &port, &device_code, &key_file)
        }

        Commands::Read {
            port,
            output,
            espflash,
        } => {
            let flash_provider = EspflashProvider::new(espflash);

            commands::read::execute(&flash_provider, &port, &output)
        }
    };

    if let Err(error) = result {
        eprintln!("[ERROR] {error}");
        process::exit(1);
    }
}

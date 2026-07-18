use std::process::Command;

use super::FlashProvider;

#[derive(Debug)]
pub struct EspflashProvider {
    executable: String,
}

impl EspflashProvider {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
        }
    }
}

impl FlashProvider for EspflashProvider {
    fn read_region(
        &self,
        port: &str,
        address: u32,
        size: usize,
        output_path: &str,
    ) -> Result<(), String> {
        let address_argument = format!("0x{address:X}");
        let size_argument = format!("0x{size:X}");

        println!("Reading identity from connected board...");
        println!("Port: {port}");
        println!("Flash Address: {address_argument}");
        println!("Read Size: {size} bytes");

        let output = Command::new(&self.executable)
            .args([
                "read-flash",
                "--port",
                port,
                "--non-interactive",
                "--skip-update-check",
                &address_argument,
                &size_argument,
                output_path,
            ])
            .output()
            .map_err(|error| format!("Could not start '{}': {}", self.executable, error))?;

        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }

        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            return Err(format!(
                "'{} read-flash' failed with exit status {}.",
                self.executable, output.status
            ));
        }

        Ok(())
    }

    fn write_region(&self, port: &str, address: u32, input_path: &str) -> Result<(), String> {
        let address_argument = format!("0x{address:X}");

        println!("Writing identity to connected board...");
        println!("Port: {port}");
        println!("Flash Address: {address_argument}");
        println!("Input File: {input_path}");

        let output = Command::new(&self.executable)
            .args([
                "write-bin",
                "--port",
                port,
                "--non-interactive",
                "--skip-update-check",
                &address_argument,
                input_path,
            ])
            .output()
            .map_err(|error| format!("Could not start '{}': {}", self.executable, error))?;

        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }

        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            return Err(format!(
                "'{} write-bin' failed with exit status {}.",
                self.executable, output.status
            ));
        }

        Ok(())
    }
}

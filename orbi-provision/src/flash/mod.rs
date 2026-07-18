pub mod espflash;

pub trait FlashProvider {
    fn read_region(
        &self,
        port: &str,
        address: u32,
        size: usize,
        output_path: &str,
    ) -> Result<(), String>;

    fn write_region(&self, port: &str, address: u32, input_path: &str) -> Result<(), String>;
}

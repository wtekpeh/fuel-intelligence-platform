pub fn calculate_checksum(data: &[u8]) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 0x811C_9DC5;
    const FNV_PRIME: u32 = 0x0100_0193;

    let mut checksum = FNV_OFFSET_BASIS;

    for byte in data {
        checksum ^= u32::from(*byte);
        checksum = checksum.wrapping_mul(FNV_PRIME);
    }

    checksum
}

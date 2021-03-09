pub fn u16_from_u8_array(u8_array: &[u8]) -> u16 {
    ((u8_array[0] as u16) << 8) + ((u8_array[1] as u16) << 0)
}

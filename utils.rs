pub fn be_to_le(v: u16) -> u16 {
    return ((v & 0xFF00) >> 8) | ((v & 0x00FF) << 8);
}

pub fn le_to_be(v: u16) -> u16 {
    return be_to_le(v);
}

pub fn bytes_to_le_word(b1: u8, b2: u8) -> u16 {
    return ((b1 as u16) << 8) | (b2 as u16);
}

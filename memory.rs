pub struct Memory {
  rom_banks: std::vec::Vec<std::vec::Vec<u8>>,
  m: std::vec::Vec<u8>,
  current_bank: usize,
}

impl Memory {
  pub fn new(cartridge_data: &std::vec::Vec<u8>) -> Memory {
    let num_banks = cartridge_data.len() / 0x4000;
    let mut banks: std::vec::Vec<std::vec::Vec<u8>> = vec![];

    for i in 0..num_banks {
      let start = i * 0x4000;
      let end = (i + 1) * 0x4000;

      banks.push(cartridge_data[start..end].to_vec());
    }

    return Memory {
      rom_banks: banks, 
      m: vec![0; 0xFFFF - 0x8000 + 1],
      current_bank: 1,
    };
  }

  #[cfg(test)]
  pub fn empty() -> Memory {
    let num_banks = 2;
    let mut banks: std::vec::Vec<std::vec::Vec<u8>> = vec![];

    for i in 0..num_banks {
      banks.push(vec![0; 0x4000]);
    }

    return Memory {
      rom_banks: banks, 
      m: vec![0; 0xFFFF - 0x8000 + 1],
      current_bank: 1,
    };
  }
}

impl std::ops::Index<u16> for Memory {
  type Output = u8;

  fn index(&self, i: u16) -> &Self::Output {
    match i {
      // This is the first, static ROM bank
      0..=0x3FFF => &self.rom_banks[0][i as usize],
      // This is the switchable bank
      0x4000..=0x7FFF => &self.rom_banks[self.current_bank][(i - 0x4000) as usize],
      // 0xC000~0xDDFF is mirrored at 0xE000~0xFDFF
      0xE000..=0xFDFF => &self.m[(i - 0x2000 - 0x8000) as usize],
      _ => &self.m[(i - 0x8000) as usize],
    }
  }
}

impl std::ops::IndexMut<u16> for Memory {
  fn index_mut(&mut self, i: u16) -> &mut Self::Output {
    match i {
      // This is the first, static ROM bank
      0..=0x3FFF => &mut self.rom_banks[0][i as usize],
      // This is the switchable bank
      0x4000..=0x7FFF => &mut self.rom_banks[self.current_bank][(i - 0x4000) as usize],
      // 0xC000~0xDDFF is mirrored at 0xE000~0xFDFF
      0xE000..=0xFDFF => &mut self.m[(i - 0x2000 - 0x8000) as usize],
      _ => &mut self.m[(i - 0x8000) as usize],
    }
  }
}
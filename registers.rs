#[derive(Copy, Clone)]
pub enum RegisterName {
  A,
  F,
  Af,
  B,
  C,
  Bc,
  D,
  E,
  De,
  H,
  L,
  Hl,
  Sp,
  Pc,
  Invalid,
}

pub struct Registers {
  pub af: u16,
  pub bc: u16,
  pub de: u16,
  pub hl: u16,
  pub sp: u16,
  pub pc: u16,
}

impl Registers {
  fn read_first_byte(r: u16) -> u8 {
    return r.to_be_bytes()[0];
  }

  fn read_second_byte(r: u16) -> u8 {
    return r.to_be_bytes()[1];
  }

  fn write_first_byte(r: &mut u16, v: u8) {
    *r = (*r & 0x00FF) | ((v as u16) << 8);
  }

  fn write_second_byte(r: &mut u16, v: u8) {
    *r = (*r & 0xFF00) | (v as u16);
  }

  pub fn z_set(&self) -> bool {
    return (self.af & 0b00000000_10000000) > 0;
  }

  pub fn n_set(&self) -> bool {
    return (self.af & 0b00000000_01000000) > 0;
  }

  pub fn h_set(&self) -> bool {
    return (self.af & 0b00000000_00100000) > 0;
  }

  pub fn c_set(&self) -> bool {
    return (self.af & 0b00000000_00010000) > 0;
  }

  pub fn set_z(&mut self) {
    self.af |= 0b00000000_10000000;
  }

  pub fn set_n(&mut self) {
    self.af |= 0b00000000_01000000;
  }

  pub fn set_h(&mut self) {
    self.af |= 0b00000000_00100000;
  }

  pub fn set_c(&mut self) {
    self.af |= 0b00000000_00010000;
  }

  pub fn reset_z(&mut self) {
    self.af &= 0b11111111_01111111;
  }

  pub fn reset_n(&mut self) {
    self.af &= 0b11111111_10111111;
  }

  pub fn reset_h(&mut self) {
    self.af &= 0b11111111_11011111;
  }

  pub fn reset_c(&mut self) {
    self.af &= 0b11111111_11101111;
  }

  pub fn read_byte(&self, r: RegisterName) -> u8 {
    match r {
      RegisterName::Af |
      RegisterName::Bc |
      RegisterName::De |
      RegisterName::Hl |
      RegisterName::Sp |
      RegisterName::Pc => {
        println!("Trying to read single byte from word register");
        unimplemented!();
      },
      RegisterName::A => return Registers::read_first_byte(self.af),
      RegisterName::B => return Registers::read_first_byte(self.bc),
      RegisterName::D => return Registers::read_first_byte(self.de),
      RegisterName::H => return Registers::read_first_byte(self.hl),

      RegisterName::F => return Registers::read_second_byte(self.af),
      RegisterName::C => return Registers::read_second_byte(self.bc),
      RegisterName::E => return Registers::read_second_byte(self.de),
      RegisterName::L => return Registers::read_second_byte(self.hl),
      _ => { println!("Unknown register name"); unimplemented!(); }
    }
  }

  pub fn read_word(&self, r: RegisterName) -> u16 {
    match r {
      RegisterName::A |
      RegisterName::F |
      RegisterName::B |
      RegisterName::C |
      RegisterName::D |
      RegisterName::E |
      RegisterName::H |
      RegisterName::L => {
        println!("Trying to read a word from byte register");
        unimplemented!();
      },
      RegisterName::Af => return self.af,
      RegisterName::Bc => return self.bc,
      RegisterName::De => return self.de,
      RegisterName::Hl => return self.hl,
      RegisterName::Sp => return self.sp,
      RegisterName::Pc => return self.pc,
      _ => { println!("Unknown register name"); unimplemented!(); }
    }
  }

  pub fn write_byte(&mut self, r: RegisterName, v: u8) {
    match r {
      RegisterName::Af |
      RegisterName::Bc |
      RegisterName::De |
      RegisterName::Hl |
      RegisterName::Sp |
      RegisterName::Pc => {
        panic!("wut");
        println!("Trying to write single byte to word register");
      },
      RegisterName::A => Registers::write_first_byte(&mut self.af, v),
      RegisterName::B => Registers::write_first_byte(&mut self.bc, v),
      RegisterName::D => Registers::write_first_byte(&mut self.de, v),
      RegisterName::H => Registers::write_first_byte(&mut self.hl, v),

      RegisterName::F => Registers::write_second_byte(&mut self.af, v),
      RegisterName::C => Registers::write_second_byte(&mut self.bc, v),
      RegisterName::E => Registers::write_second_byte(&mut self.de, v),
      RegisterName::L => Registers::write_second_byte(&mut self.hl, v),
      _ => { println!("Unknown register name"); unimplemented!(); }
    }
  }

  pub fn write_word(&mut self, r: RegisterName, v: u16) {
    match r {
      RegisterName::A |
      RegisterName::F |
      RegisterName::B |
      RegisterName::C |
      RegisterName::D |
      RegisterName::E |
      RegisterName::H |
      RegisterName::L => {
        println!("Trying to write a word to byte register");
      },
      RegisterName::Af => self.af = v,
      RegisterName::Bc => self.bc = v,
      RegisterName::De => self.de = v,
      RegisterName::Hl => self.hl = v,
      RegisterName::Sp => self.sp = v,
      RegisterName::Pc => self.pc = v,
      _ => { println!("Unknown register name"); unimplemented!(); }
    }
  }
}
use crate::registers::RegisterName;
use crate::registers::Registers;
use crate::utils;
use crate::memory::Memory;

pub struct Location {
  immediate: u16,
  address: u16,
  is_immediate: bool,
  is_register: bool,
  register: RegisterName,
}

impl Location {
    pub fn from_immediate_byte(v: u8) -> Location {
    return Location { 
      immediate: (v as u16) << 8,
      address: 0,
      is_immediate: true,
      is_register: false,
      register: RegisterName::Invalid,
    };
  }

  pub fn from_immediate(v: u16) -> Location {
    return Location { 
      immediate: utils::be_to_le(v),
      address: 0,
      is_immediate: true,
      is_register: false,
      register: RegisterName::Invalid,
    };
  }

  pub fn from_address(a: u16) -> Location {
    return Location {
      immediate: 0,
      address: a,
      is_immediate: false,
      is_register: false,
      register: RegisterName::Invalid,
    };
  }

  pub fn from_high_address(a: u8) -> Location {
    return Location {
      immediate: 0,
      address: 0xFF00 | (a as u16),
      is_immediate: false,
      is_register: false,
      register: RegisterName::Invalid,
    };
  }

  pub fn from_immediate_register(r: RegisterName) -> Location {
    return Location {
      immediate: 0,
      address: 0,
      is_immediate: false,
      is_register: true,
      register: r,
    };
  }

  pub fn read_byte(&self, memory: &Memory, registers: &Registers) -> u8 {
    if self.is_immediate {
      return self.immediate.to_be_bytes()[0];
    } else if self.is_register {
      return registers.read_byte(self.register);
    } else {
      return memory[self.address].to_be_bytes()[0];
    }
  }

  pub fn read_word(&self, memory: &Memory, registers: &Registers) -> u16 {
    // Values are stored in LE order, so we need to read 2 u8s from the location and swap them.
    if self.is_immediate {
      return utils::be_to_le(self.immediate);
    } else if self.is_register {
      return registers.read_word(self.register);
    } else {
      return ((memory[self.address + 1] as u16) << 8) | (memory[self.address] as u16);
    }
  }

  pub fn write_byte(&mut self, memory: &mut Memory, registers: &mut Registers, v: u8) {
    if self.is_register {
      registers.write_byte(self.register, v);
    } else {
      memory.set(self.address, v);
    }
  }

  pub fn write_word(&mut self, memory: &mut Memory, registers: &mut Registers, v: u16) {
    if self.is_register {
      registers.write_word(self.register, v);
    } else {
      // LE means we need to swap the bytes before writing them
      memory.set(self.address, v.to_be_bytes()[1]);
      memory.set(self.address + 1, v.to_be_bytes()[0]);
    }
  }
}
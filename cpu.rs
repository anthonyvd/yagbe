use crate::registers::Registers;
use crate::registers::RegisterName;
use crate::memory_utils::Source;
use crate::memory_utils::Dest;
use crate::utils;
use crate::memory::Memory;

pub struct Cpu {
  pub registers: Registers,
  pub ime: bool,
}

impl Cpu {
  pub fn new() -> Cpu {
    return Cpu {
      registers: Registers {
        af: 0x01B0,
        bc: 0x0013,
        de: 0x00D8,
        hl: 0x014D,
        sp: 0xFFFE,
        pc: 0x0100,
      },
      ime: true
    };
  }

  fn dump_registers(&self) {
    println!("AF: {:04X}", self.registers.af);
    println!("BC: {:04X}", self.registers.bc);
    println!("DE: {:04X}", self.registers.de);
    println!("HL: {:04X}", self.registers.hl);
    println!("SP: {:04X}", self.registers.sp);
    println!("PC: {:04X}", self.registers.pc);
  }

  // Reads the byte at (|pc|) then increments |pc|
  fn pc_read(&mut self, memory: &Memory) -> u8 {
    let ret = memory[self.registers.pc];
    self.registers.pc += 1;
    return ret;
  }

  fn pc_read_word(&mut self, memory: &Memory) -> u16 {
    let b1 = self.pc_read(memory);
    let b2 = self.pc_read(memory);

    return utils::le_to_be(utils::bytes_to_le_word(b1, b2));
  }

  fn get_ld_arithmetic_bit_source(&self, instr: u8) -> Source {
    return match (instr & 0x0F) % 0x08 {
      0x00 => Source::from_register(RegisterName::B),
      0x01 => Source::from_register(RegisterName::C),
      0x02 => Source::from_register(RegisterName::D),
      0x03 => Source::from_register(RegisterName::E),
      0x04 => Source::from_register(RegisterName::H),
      0x05 => Source::from_register(RegisterName::L),
      0x06 => Source::from_address(self.registers.hl),
      0x07 => Source::from_register(RegisterName::A),
      _ => unimplemented!(),
    };
  }

  fn get_ld_arithmetic_bit_dest(&self, instr: u8) -> Dest {
    return match (instr & 0x0F) % 0x08 {
      0x00 => Dest::from_register(RegisterName::B),
      0x01 => Dest::from_register(RegisterName::C),
      0x02 => Dest::from_register(RegisterName::D),
      0x03 => Dest::from_register(RegisterName::E),
      0x04 => Dest::from_register(RegisterName::H),
      0x05 => Dest::from_register(RegisterName::L),
      0x06 => Dest::from_address(self.registers.hl),
      0x07 => Dest::from_register(RegisterName::A),
      _ => unimplemented!(),
    };
  }

  fn push_byte(&mut self, b: u8, memory: &mut Memory) {
    //println!("push_byte {:04X}", self.registers.sp);
    self.registers.sp = self.registers.sp.wrapping_sub(1);
    Dest::from_address(self.registers.sp).write_byte(memory, &mut self.registers, b);
  }

  fn pop_byte(&mut self, memory: &mut Memory) -> u8 {
    //println!("pop_byte {:04X}", self.registers.sp);
    let res = Source::from_address(self.registers.sp).read_byte(memory, &self.registers);
    self.registers.sp = self.registers.sp.wrapping_add(1);
    return res;
  }

  fn push_word(&mut self, w: u16, memory: &mut Memory) {
    let h = ((w & 0xFF00) >> 8) as u8;
    let l = (w & 0x00FF) as u8;
    self.push_byte(h, memory);
    self.push_byte(l, memory);
  }

  fn pop_word(&mut self, memory: &mut Memory) -> u16 {
    let l = self.pop_byte(memory);
    let h = self.pop_byte(memory);
    return ((h as u16) << 8) | (l as u16);
  }

  pub fn tick(&mut self, memory: &mut Memory) {
    // This currently runs each instruction in a single simulated TCycle, which isn't accurate.

    /*
    if self.registers.pc == 0x27DA {
      self.dump_registers();
      for i in 0x8000..0x8000 + 16 {
        println!("byte {:04X}", memory[i]);
      }
    }*/
    // println!("Executing instruction at 0x{:04X}", self.registers.pc);
    let instr: u8 = self.pc_read(memory);


    // If it's the CB prefix byte, fetch the next one
    if instr == 0xCB {
      let arg = self.pc_read(memory);
      self.exec_pref(arg, memory);
    } else {
      self.exec_unpref(instr, memory);
    }
  }

  fn exec_pref(&mut self, instr: u8, memory: &mut Memory) {
    let src: Source = self.get_ld_arithmetic_bit_source(instr);
    let mut dest: Dest = self.get_ld_arithmetic_bit_dest(instr);
    match instr {/*
      0x00..=0x07 => {

      },
      0x08..=0x0F => {

      },
      0x10..=0x17 => {

      },
      0x18..=0x1F => {

      },
      0x20..=0x27 => {

      },
      0x28..=0x2F => {

      },*/
      0x30..=0x37 => {
        // SWAP
        let byte = src.read_byte(memory, &self.registers);
        let h = byte & 0xF0;
        let l = byte & 0x0F;

        let res = (h >> 4) | (l << 4);
        dest.write_byte(memory, &mut self.registers, res);

        if res == 0 { self.registers.set_z(); }
        self.registers.reset_n();
        self.registers.reset_h();
        self.registers.reset_c();
      },/*
      0x38..=0x3F => {

      },*/
      0x40..=0x7F => {
        // BIT
        let index = (instr - 0x40) / 0x08;
        let pattern = 0b00000001 << index;
        let set: bool = (src.read_byte(memory, &self.registers) & pattern) > 0;

        if !set { self.registers.set_z(); }
        self.registers.reset_n();
        self.registers.set_h();
      },
      0x80..=0xBF => {
        // RES
        let index = (instr - 0x80) / 0x08;
        let pattern: u8 = !(0b00000001 << index);
        let byte = src.read_byte(memory, &self.registers);
        dest.write_byte(memory, &mut self.registers, byte & pattern);
      },
      0xC0..=0xFF => {
        // SET
        let index = (instr - 0xC0) / 0x08;
        let pattern: u8 = 0b00000001 << index;
        let byte = src.read_byte(memory, &self.registers);
        dest.write_byte(memory, &mut self.registers, byte | pattern);
      },
      _ =>  {
        println!("Unknown CB-prefixed Instruction: 0x{:02X}", instr); 
        unimplemented!();
      },
    };
  }

  fn exec_unpref(&mut self, instr: u8, memory: &mut Memory) {
    match instr {
      0x00 => self.nop(),
      0x01 => {
        let arg = self.pc_read_word(memory);
        self.ld16(Dest::from_register(RegisterName::Bc), Source::from_immediate(arg), memory);
      },
      0x02 => self.ld(Dest::from_address(self.registers.bc), Source::from_register(RegisterName::A), memory),
      0x03 => self.inc16(Dest::from_register(RegisterName::Bc), Source::from_register(RegisterName::Bc), memory),
      0x04 => self.inc(Dest::from_register(RegisterName::B), Source::from_register(RegisterName::B), memory),
      0x05 => self.dec(Dest::from_register(RegisterName::B), Source::from_register(RegisterName::B), memory),
      0x06 => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_register(RegisterName::B), Source::from_immediate_byte(arg), memory);
      },
      0x07 => {
        self.rlc(Dest::from_register(RegisterName::A), Source::from_register(RegisterName::A), memory);
      },

      0x0B => self.dec16(Dest::from_register(RegisterName::Bc), Source::from_register(RegisterName::Bc), memory),
      0x0C => self.inc(Dest::from_register(RegisterName::C), Source::from_register(RegisterName::C), memory),
      0x0D => self.dec(Dest::from_register(RegisterName::C), Source::from_register(RegisterName::C), memory),
      0x0E => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_register(RegisterName::C), Source::from_immediate_byte(arg), memory);
      },
      0x11 => {
        let arg = self.pc_read_word(memory);
        self.ld16(Dest::from_register(RegisterName::De), Source::from_immediate(arg), memory);
      },
      0x12 => {
        self.ld(Dest::from_address(self.registers.de), Source::from_register(RegisterName::A), memory);
      },
      0x13 => self.inc16(Dest::from_register(RegisterName::De), Source::from_register(RegisterName::De), memory),
      0x14 => self.inc(Dest::from_register(RegisterName::D), Source::from_register(RegisterName::D), memory),
      0x15 => self.dec(Dest::from_register(RegisterName::D), Source::from_register(RegisterName::D), memory),
      0x16 => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_register(RegisterName::D), Source::from_immediate_byte(arg), memory);
      },

      0x18 => {
        let arg = self.pc_read(memory);
        self.jr(Source::from_immediate_byte(arg), memory);
      },
      0x19 => {
        self.add16(Dest::from_register(RegisterName::Hl),
          Source::from_register(RegisterName::Hl), Source::from_register(RegisterName::De), memory);
      },
      0x1A => {
        self.ld(Dest::from_register(RegisterName::A), Source::from_address(self.registers.de), memory);
      },
      0x1B => self.dec16(Dest::from_register(RegisterName::De), Source::from_register(RegisterName::De), memory),
      0x1C => self.inc(Dest::from_register(RegisterName::D), Source::from_register(RegisterName::D), memory),
      0x1D => self.dec(Dest::from_register(RegisterName::D), Source::from_register(RegisterName::D), memory),

      0x20 => {
        let arg = self.pc_read(memory);
        if !self.registers.z_set() {
          self.jr(Source::from_immediate_byte(arg), memory);
        }
      },
      0x21 => {
        let arg = self.pc_read_word(memory);
        self.ld16(Dest::from_register(RegisterName::Hl), Source::from_immediate(arg), memory);
      },
      0x22 => {
        self.ld(Dest::from_address(self.registers.hl), Source::from_register(RegisterName::A), memory);
        self.registers.hl = self.registers.hl.wrapping_add(1);
      },
      0x23 => self.inc16(Dest::from_register(RegisterName::Hl), Source::from_register(RegisterName::Hl), memory),
      0x24 => self.inc(Dest::from_register(RegisterName::H), Source::from_register(RegisterName::H), memory),
      0x25 => self.dec(Dest::from_register(RegisterName::H), Source::from_register(RegisterName::H), memory),
      0x28 => {
        let arg = self.pc_read(memory);
        if self.registers.z_set() {
          self.jr(Source::from_immediate_byte(arg), memory);
        }
      },
      0x2A => {
        self.ld(Dest::from_register(RegisterName::A), Source::from_address(self.registers.hl), memory);
        self.registers.hl = self.registers.hl.wrapping_add(1);
      },

      0x2B => self.dec16(Dest::from_register(RegisterName::Hl), Source::from_register(RegisterName::Hl), memory),
      0x2C => self.inc(Dest::from_register(RegisterName::L), Source::from_register(RegisterName::L), memory),
      0x2D => self.dec(Dest::from_register(RegisterName::L), Source::from_register(RegisterName::L), memory),

      0x2F => self.cpl(),
      0x31 => {
        let arg = self.pc_read_word(memory);
        self.ld16(Dest::from_register(RegisterName::Sp), Source::from_immediate(arg), memory);
      },
      0x32 => {
        self.ld(Dest::from_address(self.registers.hl), Source::from_register(RegisterName::A), memory);
        self.registers.hl = self.registers.hl.wrapping_sub(1);
      },
      0x33 => self.inc16(Dest::from_register(RegisterName::Sp), Source::from_register(RegisterName::Sp), memory),
      0x34 => self.inc(Dest::from_address(self.registers.hl), Source::from_address(self.registers.hl), memory),
      0x35 => self.dec(Dest::from_address(self.registers.hl), Source::from_address(self.registers.hl), memory),
      0x36 => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_address(self.registers.hl), Source::from_immediate_byte(arg), memory);
      },

      0x3B => self.dec16(Dest::from_register(RegisterName::Sp), Source::from_register(RegisterName::Sp), memory),
      0x3C => self.inc(Dest::from_register(RegisterName::A), Source::from_register(RegisterName::A), memory),
      0x3D => self.dec(Dest::from_register(RegisterName::A), Source::from_register(RegisterName::A), memory),
      0x3E => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_register(RegisterName::A), Source::from_immediate_byte(arg), memory);
      },

      0x76 => self.halt(),
      0x40..=0x75 | 0x77..=0x7F => {
        let dest: Dest = match instr {
          0x40..=0x47 => Dest::from_register(RegisterName::B),
          0x48..=0x4F => Dest::from_register(RegisterName::C),
          0x50..=0x57 => Dest::from_register(RegisterName::D),
          0x58..=0x5F => Dest::from_register(RegisterName::E),
          0x60..=0x67 => Dest::from_register(RegisterName::H),
          0x68..=0x6F => Dest::from_register(RegisterName::L),
          0x70..=0x77 => Dest::from_address(self.registers.hl),
          0x78..=0x7F => Dest::from_register(RegisterName::A),
          _ => unimplemented!(),
        };
        let src: Source = self.get_ld_arithmetic_bit_source(instr);

        self.ld(dest, src, memory);
      },

      0x80..=0xB7 => {
        let dest = Dest::from_register(RegisterName::A);
        let src_a = Source::from_register(RegisterName::A);
        let src = self.get_ld_arithmetic_bit_source(instr);
        match instr {
          0x80..=0x87 => {
            self.add(dest, src_a, src, memory);
          },
          0x88..=0x8F => {
            self.adc(dest, src_a, src, memory);
          },
          0x90..=0x97 => {
            self.sub(dest, src_a, src, memory);
          },
          0x98..=0x9F => {
            self.sbc(dest, src_a, src, memory);
          },
          0xA0..=0xA7 => {
            self.and(dest, src_a, src, memory);
          },
          0xA8..=0xAF => {
            self.xor(dest, src_a, src, memory);
          },
          0xB0..=0xB7 => {
            self.or(dest, src_a, src, memory);
          },
          _ => unimplemented!(),
        };
      },
      0xB8..=0xBF => {
        let src_a = Source::from_register(RegisterName::A);
        let src = self.get_ld_arithmetic_bit_source(instr);
        self.cp(src_a, src, memory);
      },
      0xC1 => {
        self.registers.bc = self.pop_word(memory);
      }
      0xC3 => { 
        let arg = self.pc_read_word(memory);
        self.jp(Source::from_immediate(arg), memory);
      },
      0xC5 => {
        self.push_word(self.registers.bc, memory);
      },
      0xC8 => {
        if self.registers.z_set() {
          self.ret(memory);
        }
      },
      0xC9 => self.ret(memory),
      0xCA => {
        let arg = self.pc_read_word(memory);
        if self.registers.z_set() {
          self.jp(Source::from_immediate(arg), memory);
        }
      },
      0xCD => {
        let arg = self.pc_read_word(memory);
        self.call(arg, memory);
      },
      0xD1 => {
        self.registers.de = self.pop_word(memory);
      },
      0xD5 => {
        self.push_word(self.registers.de, memory);
      },

      0xE0 => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_address(0xFF00 | (arg as u16)), Source::from_register(RegisterName::A), memory);
      },
      0xE1 => {
        self.registers.hl = self.pop_word(memory);
      },
      0xE2 => {
        self.ld(Dest::from_address(0xFF00 | (self.registers.bc & 0x00FF)), Source::from_register(RegisterName::A), memory);
      },
      0xE4 => {
        println!("Illegal instruction 0xE4 at {:04X}-ish", self.registers.pc);
      },
      0xE5 => {
        self.push_word(self.registers.hl, memory);
      },
      0xE6 => {
        let arg = self.pc_read(memory);
        self.and(Dest::from_register(RegisterName::A), Source::from_register(RegisterName::A), Source::from_immediate_byte(arg), memory);
      },
      0xE9 => {
        self.jp(Source::from_register(RegisterName::Hl), memory);
      },
      0xEA => {
        let arg = self.pc_read_word(memory);
        self.ld(Dest::from_address(arg), Source::from_register(RegisterName::A), memory);
      },
      0xEF => self.rst(0x0028, memory),
      0xF0 => {
        let arg = self.pc_read(memory);
        self.ld(Dest::from_register(RegisterName::A), Source::from_address(0xFF00 | (arg as u16)), memory);
      },
      0xF1 => {
        self.registers.af = self.pop_word(memory);
      },
      0xF3 => self.di(),
      0xF5 => {
        self.push_word(self.registers.af, memory);
      },
      0xFA => {
        let arg = self.pc_read_word(memory);
        self.ld(Dest::from_register(RegisterName::A), Source::from_address(arg), memory);
      },
      0xFB => self.ei(),
      0xFE => {
        let arg = self.pc_read(memory);
        self.cp(Source::from_register(RegisterName::A), Source::from_immediate_byte(arg), memory);
      },
      0xFF => self.rst(0x0038, memory),
      _ => {
        println!("Unknown Instruction: 0x{:02X}, SP: {:04X}, PC {:04X}", instr, self.registers.sp, self.registers.pc); 
        unimplemented!();
      }
    }
  }


  // -- 8bit arithmetic + logic

  fn adc(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &self.registers)
      .wrapping_add(s2.read_byte(memory, &self.registers))
      .wrapping_add(if self.registers.c_set() { 1 } else { 0 });

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    // TODO: carry flags
    d.write_byte(memory, &mut self.registers, res);
  }

  fn add(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &self.registers)
      .wrapping_add(s2.read_byte(memory, &self.registers));
      
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    // TODO: carry flags
    d.write_byte(memory, &mut self.registers, res);
  }

  fn add16(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_word(memory, &self.registers)
      .wrapping_add(s2.read_word(memory, &self.registers));

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    // TODO: carry flags
    d.write_word(memory, &mut self.registers, res);
  }

  fn and(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &mut self.registers) & s2.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.set_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  // CP doesn't store results so it doesn't have a destination
  fn cp(&mut self, s1: Source, s2: Source, memory: &mut Memory) {
    let first = s1.read_byte(memory, &self.registers);
    let second = s2.read_byte(memory, &self.registers);
    if first == second { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    // TODO: carry flags
  }

  // |s1| and |d| should represent the same location
  fn dec(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let res = s.read_byte(memory, &self.registers).wrapping_sub(1);
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    // TODO: there should be a possible H flag set here

    d.write_byte(memory, &mut self.registers, res);
  }

  // |s1| and |d| should represent the same location
  fn inc(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let res = s.read_byte(memory, &self.registers).wrapping_add(1);
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    // TODO: there should be a possible H flag set here

    d.write_byte(memory, &mut self.registers, res);
  }

  fn or(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &self.registers) | s2.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  fn sbc(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &self.registers)
      .wrapping_sub(s2.read_byte(memory, &self.registers))
      .wrapping_sub(if self.registers.c_set() { 1 } else { 0 });
      
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    // TODO: carry flags
    d.write_byte(memory, &mut self.registers, res);

  }

  fn sub(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &self.registers)
      .wrapping_sub(s2.read_byte(memory, &self.registers));
      
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    // TODO: carry flags
    d.write_byte(memory, &mut self.registers, res);

  }

  fn xor(&mut self, mut d: Dest, s1: Source, s2: Source, memory: &mut Memory) {
    let res = s1.read_byte(memory, &self.registers) ^ s2.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  // -- 16 bit arithmetic

  fn dec16(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let res = s.read_word(memory, &self.registers).wrapping_sub(1);
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    // TODO: there should be a possible H flag set here

    d.write_word(memory, &mut self.registers, res);
  }

  fn inc16(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let res = s.read_word(memory, &self.registers).wrapping_add(1);
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    // TODO: there should be a possible H flag set here

    d.write_word(memory, &mut self.registers, res);
  }

  // -- bit shifts
  fn rlc(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let val = s.read_byte(memory, &self.registers);
    let msb = val & 0b10000000;

    self.registers.reset_z();
    self.registers.reset_n();
    self.registers.reset_h();
    if msb > 0 {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }

    let res = (val << 1) | (msb >> 7);
    d.write_byte(memory, &mut self.registers, res);
  }

  // -- loads

  fn ld(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let byte = s.read_byte(memory, &self.registers);
    d.write_byte(memory, &mut self.registers, byte);
  }

  fn ld16(&mut self, mut d: Dest, s: Source, memory: &mut Memory) {
    let word = s.read_word(memory, &self.registers);
    d.write_word(memory, &mut self.registers, word);
  }

  // -- jumps and subroutines

  fn jp(&mut self, s: Source, memory: &mut Memory) {
    self.registers.pc = s.read_word(memory, &self.registers);
  }

  fn jr(&mut self, s: Source, memory: &mut Memory) {
    let byte = s.read_byte(memory, &self.registers);

    let add = (byte & 0b10000000) == 0;
    let mut jump = 0; 
    if add { jump = byte as u16; } else { jump = (!byte).wrapping_add(1) as u16; };

    if add {
      self.registers.pc = self.registers.pc.wrapping_add(jump);
    } else {
      self.registers.pc = self.registers.pc.wrapping_sub(jump);
    }
  }

  fn call(&mut self, addr: u16, memory: &mut Memory) {
    let pc = self.registers.pc;
    self.push_word(pc, memory);
    self.registers.pc = addr;
  }

  fn ret(&mut self, memory: &mut Memory) {
    self.registers.pc = self.pop_word(memory);
  }

  fn rst(&mut self, addr: u16, memory: &mut Memory) {
    // TODO: eventually this will probably need to do something different to emulate that RST is faster than CALL
    self.call(addr, memory);
  }

  // -- stack ops

  // -- misc ops

  fn nop(&self) {}

  fn cpl(&mut self) {
    let a = ((self.registers.af & 0xFF00) >> 8) as u8;
    let c = ((!a) as u16) << 8;

    self.registers.af = (self.registers.af & 0x00FF) | c;
  }

  fn ei(&mut self) {
    self.ime = true;
  }

  fn di(&mut self) {
    self.ime = false;
  }

  fn halt(&self) { 
    // TODO we don't have interrupts yet so....
  }
}
use crate::registers::Registers;
use crate::registers::RegisterName;
use crate::memory_utils::Location;
use crate::utils;
use crate::memory::Memory;
use crate::opcodes;

pub struct Cpu {
  pub registers: Registers,
  pub ime: bool,
  pub cycles_stalled: u8,
  pub halted: bool,
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
      ime: false,
      cycles_stalled: 0,
      halted: false,
    };
  }

  // Reads the byte at (|pc|) then increments |pc|
  pub fn pc_read(&mut self, memory: &Memory) -> u8 {
    let ret = memory[self.registers.pc];
    self.registers.pc += 1;
    return ret;
  }

  pub fn pc_read_word(&mut self, memory: &Memory) -> u16 {
    let b1 = self.pc_read(memory);
    let b2 = self.pc_read(memory);

    return utils::le_to_be(utils::bytes_to_le_word(b1, b2));
  }

  fn get_ld_arithmetic_bit_source(&self, instr: u8) -> Location {
    return match (instr & 0x0F) % 0x08 {
      0x00 => Location::from_immediate_register(RegisterName::B),
      0x01 => Location::from_immediate_register(RegisterName::C),
      0x02 => Location::from_immediate_register(RegisterName::D),
      0x03 => Location::from_immediate_register(RegisterName::E),
      0x04 => Location::from_immediate_register(RegisterName::H),
      0x05 => Location::from_immediate_register(RegisterName::L),
      0x06 => Location::from_address(self.registers.hl),
      0x07 => Location::from_immediate_register(RegisterName::A),
      _ => unimplemented!(),
    };
  }

  fn get_ld_arithmetic_bit_dest(&self, instr: u8) -> Location {
    return match (instr & 0x0F) % 0x08 {
      0x00 => Location::from_immediate_register(RegisterName::B),
      0x01 => Location::from_immediate_register(RegisterName::C),
      0x02 => Location::from_immediate_register(RegisterName::D),
      0x03 => Location::from_immediate_register(RegisterName::E),
      0x04 => Location::from_immediate_register(RegisterName::H),
      0x05 => Location::from_immediate_register(RegisterName::L),
      0x06 => Location::from_address(self.registers.hl),
      0x07 => Location::from_immediate_register(RegisterName::A),
      _ => unimplemented!(),
    };
  }

  fn push_byte(&mut self, b: u8, memory: &mut Memory) {
    self.registers.sp = self.registers.sp.wrapping_sub(1);
    Location::from_address(self.registers.sp).write_byte(memory, &mut self.registers, b);
  }

  fn pop_byte(&mut self, memory: &mut Memory) -> u8 {
    let res = Location::from_address(self.registers.sp).read_byte(memory, &self.registers);
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

  pub fn tick(&mut self, memory: &mut Memory, stall: bool) -> bool {
    if self.halted && (memory[0xFF0F] & memory[0xFFFF] == 0) {
      return false;
    }

    if stall && self.cycles_stalled > 0 {
      self.cycles_stalled = self.cycles_stalled - 1;
      return false;
    }

    if self.ime {
      // TODO: The interrupt handling routing wastes some cycles
      if (memory[0xFF0F] & 0x01 != 0) &&
         (memory[0xFFFF] & 0x01 != 0) {
        // VBLANK
        memory[0xFF0F] = memory[0xFF0F] & 0xFE;
        self.ime = false;
        self.call(Location::from_immediate(0x40), memory, true, true);
      } else if (memory[0xFF0F] & 0b10 != 0) &&
                (memory[0xFFFF] & 0b10 != 0) {
        // STAT
        memory[0xFF0F] = memory[0xFF0F] & !0b10;
        self.ime = false;
        self.call(Location::from_immediate(0x48), memory, true, true);
      } else if (memory[0xFF0F] & 0b100 != 0) &&
                (memory[0xFFFF] & 0b100 != 0) {
        // TIMER
        memory[0xFF0F] = memory[0xFF0F] & !0b100;
        self.ime = false;
        self.call(Location::from_immediate(0x50), memory, true, true);
      } else if (memory[0xFF0F] & 0b1000 != 0) &&
                (memory[0xFFFF] & 0b1000 != 0) {
        // SERIAL
        memory[0xFF0F] = memory[0xFF0F] & !0b1000;
        self.ime = false;
        self.call(Location::from_immediate(0x58), memory, true, true);
      } else if (memory[0xFF0F] & 0b10000 != 0) &&
                (memory[0xFFFF] & 0b10000 != 0) {
        // JOYPAD
        memory[0xFF0F] = memory[0xFF0F] & !0b10000;
        self.ime = false;
        self.call(Location::from_immediate(0x60), memory, true, true);
      }
    }

    let instr: u8 = self.pc_read(memory);

    // If it's the CB prefix byte, fetch the next one
    if instr == 0xCB {
      let arg = self.pc_read(memory);
      self.exec_pref(arg, memory);
      if stall {
        self.cycles_stalled = 8 + 4; // 4 for the prefix, 8 for the instruction
      }
    } else {
      let stall_len = opcodes::exec_unpref(instr, memory, self);
      if stall {
        self.cycles_stalled = stall_len;
      }
    }

    return true;
  }

  pub fn call(&mut self, arg: Location, memory: &mut Memory, cond: bool, _is_16: bool) {
    if !cond {
      return;
    }

    let pc = self.registers.pc;
    self.push_word(pc, memory);
    self.registers.pc = arg.read_word(memory, &self.registers);
  }

  pub fn rst(&mut self, arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("rst can't have cond"); }

    // No cond check because 1/ RST has no conditional variant and 2/ CALL checks it anyway
    // TODO: eventually this will probably need to do something different to emulate that RST is faster than CALL
    self.call(arg, memory, cond, is_16);
  }

  pub fn push(&mut self, arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("push has a false cond"); }
    if !is_16 { panic!("push is 8 bits"); }

    self.push_word(arg.read_word(memory, &self.registers), memory);
  }

  pub fn pop(&mut self, mut arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("pop has a false cond"); }
    if !is_16 { panic!("pop is 8 bits"); }

    let word = self.pop_word(memory);
    arg.write_word(memory, &mut self.registers, word);
  }

  pub fn cp(&mut self, s1: Location, s2: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("CP has a false cond"); }
    if is_16 { panic!("CP is 16 bits"); }

    let first = s1.read_byte(memory, &self.registers);
    let second = s2.read_byte(memory, &self.registers);

    if first == second { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    if second > first { self.registers.set_c(); } else { self.registers.reset_c(); }
    if (second & 0x0F) > (first & 0x0F) {
      self.registers.set_h();
    } else {
      self.registers.reset_h();
    }
  }

  pub fn or(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("OR has a false cond"); }
    if is_16 { panic!("OR is 16 bits"); }

    let res = d.read_byte(memory, &self.registers) | s.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn and(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("AND has a false cond"); }
    if is_16 { panic!("AND is 16 bits"); }

    let res = d.read_byte(memory, &mut self.registers) & s.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.set_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn add(&mut self, mut d: Location, s: Location, memory: &mut Memory, _cond: bool, is_16: bool) {
    if is_16 {
      let left = d.read_word(memory, &self.registers);
      let right = s.read_word(memory, &self.registers);

      let res = left.wrapping_add(right);

      // Z flag not affected in 16 bit adds
      self.registers.reset_n();
      if (left & 0xFFF) + (right & 0xFFF) > 0xFFF {
        self.registers.set_h();
      } else {
        self.registers.reset_h();
      }

      if (left as u32) + (right as u32) > 0xFFFF {
        self.registers.set_c();
      } else {
        self.registers.reset_c();
      }

      d.write_word(memory, &mut self.registers, res);
    } else {

      let left = d.read_byte(memory, &self.registers);
      let right = s.read_byte(memory, &self.registers);

      let res = left.wrapping_add(right);
        
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.reset_n();
      if (left & 0x0F) + (right & 0x0F) > 0x0F {
        self.registers.set_h();
      } else {
        self.registers.reset_h();
      }

      if 0xFF - left < right {
        self.registers.set_c();
      } else {
        self.registers.reset_c();
      }

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn sub(&mut self, mut d: Location, s: Location, memory: &mut Memory, _cond: bool, is_16: bool) {
    if is_16 {
      unimplemented!();
    } else {
      let left = d.read_byte(memory, &self.registers);
      let right = s.read_byte(memory, &self.registers);

      let res = left.wrapping_sub(right);
        
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.set_n();
      if right > left { self.registers.set_c(); } else { self.registers.reset_c(); }
      if (right & 0x0F) > (left & 0x0F) {
        self.registers.set_h();
      } else {
        self.registers.reset_h();
      }

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn adc(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("ADC can't have cond"); }
    if is_16 { panic!("ADC can't be 16 bits"); }

    let left = d.read_byte(memory, &self.registers);
    let right = s.read_byte(memory, &self.registers);
    let c = if self.registers.c_set() { 1 } else { 0 };

    let mut res = left.wrapping_add(right);
    let overflow = res < left || (res == 0xFF && c == 1);

    res = res.wrapping_add(c);

    self.registers.reset_n();
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    if overflow { self.registers.set_c(); } else { self.registers.reset_c(); }
    if (left & 0x0F) + (right & 0x0F) + c > 0x0F {
        self.registers.set_h();
      } else {
        self.registers.reset_h();
      }

    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn sbc(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("SBC can't have cond"); }
    if is_16 { panic!("SBC can't be 16 bits"); }

    let left = d.read_byte(memory, &self.registers);
    let right = s.read_byte(memory, &self.registers);
    let c = if self.registers.c_set() { 1 } else { 0 };
    let _underflow = false;

    let mut res = left.wrapping_sub(right);

    let underflow = res > left || (res == 0 && c == 1);

    res = res.wrapping_sub(c);
      
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    if underflow { self.registers.set_c(); } else { self.registers.reset_c(); }
    self.registers.set_n();

    if (right & 0x0F) > (left & 0x0F) || ((right & 0x0F) == (left & 0x0F) && c == 1) {
      self.registers.set_h();
    } else {
      self.registers.reset_h();
    }

    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn dec(&mut self, mut d: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("DEC has a false cond"); }

    if is_16 {
      let res = d.read_word(memory, &self.registers).wrapping_sub(1);
      d.write_word(memory, &mut self.registers, res);
    } else {
      let byte = d.read_byte(memory, &self.registers);
      let res = byte.wrapping_sub(1);

      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.set_n();
      if byte & 0x0F == 0x00 { self.registers.set_h(); } else { self.registers.reset_h(); }

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn inc(&mut self, mut d: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("INC has a false cond"); }

    if is_16 {
      let res = d.read_word(memory, &self.registers).wrapping_add(1);
      d.write_word(memory, &mut self.registers, res);
    } else {
      let byte = d.read_byte(memory, &self.registers);
      let res = byte.wrapping_add(1);

      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.reset_n();
      if byte & 0x0F == 0x0F { self.registers.set_h(); } else { self.registers.reset_h(); }

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn ret(&mut self, memory: &mut Memory, cond: bool, _is_16: bool) {
    if !cond {
      return;
    }

    self.registers.pc = self.pop_word(memory);
  }

  pub fn reti(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("RETI has a false cond"); }

    self.ei(memory, true, false);
    self.ret(memory, cond, is_16);
  }

  pub fn halt(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    // TODO: halt bug, see pandocs
    self.halted = true;
  }

  pub fn stop(&mut self, _d: Location, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    // d is ignored
    unimplemented!();
  }

  pub fn ccf(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    if self.registers.c_set() {
      self.registers.reset_c();
    } else {
      self.registers.set_c();
    }
    self.registers.reset_n();
    self.registers.reset_h();
  }

  pub fn scf(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.registers.set_c();
    self.registers.reset_n();
    self.registers.reset_h();
  }

  pub fn cpl(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    let byte = self.registers.read_byte(RegisterName::A);
    self.registers.write_byte(RegisterName::A, byte ^ 0xFF);
    self.registers.set_n();
    self.registers.set_h();
  }

  pub fn daa(&mut self, _memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("daa has cond"); }
    if is_16 { panic!("daa is 16 bits"); }

    let mut a = self.registers.read_byte(RegisterName::A) as u16;
    let c_set = self.registers.c_set();
    self.registers.reset_c();

    if !self.registers.n_set() {
      if self.registers.h_set() || a & 0xF > 9 {
        a = a.wrapping_add(0x06);
      }

      if self.registers.c_set() || a > 0x9F {
        a = a.wrapping_add(0x60);
      }
    } else {
      if self.registers.h_set() {
        a = (a.wrapping_sub(6)) & 0xFF;
      }

      if self.registers.c_set() {
        a = a.wrapping_sub(0x60);
      }
    }

    self.registers.reset_h();

    if a & 0x100 == 0x100 {
      self.registers.set_c();
    }

    a = a & 0xFF;

    if a == 0 {
      self.registers.set_z();
    } else {
      self.registers.reset_z();
    }

    self.registers.write_byte(RegisterName::A, a as u8);
  }

  pub fn rra(&mut self, memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.rr(Location::from_immediate_register(RegisterName::A), memory, true, false, true);
  }

  pub fn rla(&mut self, memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.rl(Location::from_immediate_register(RegisterName::A), memory, true, false, true);
  }

  pub fn rrca(&mut self, memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.rr(Location::from_immediate_register(RegisterName::A), memory, false, false, true);
  }

  pub fn rlca(&mut self, memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.rl(Location::from_immediate_register(RegisterName::A), memory, false, false, true);
  }

  pub fn nop(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    // Might burn cycles
  }

  pub fn ei(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.ime = true;
  }

  pub fn di(&mut self, _memory: &mut Memory, _cond: bool, _is_16: bool) {
    self.ime = false;
  }

  pub fn ld(&mut self, d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("LD is cond"); }

    if is_16 {
      self.ld16(d, s, memory)
    } else {
      self.ld8(d, s, memory)
    }
  }

  pub fn ldh(&mut self, d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("LDH is cond"); }
    if is_16 { panic!("LDH is 16"); }

    self.ld8(d, s, memory);
  }

  pub fn xor(&mut self, mut d: Location, s: Location, memory: &mut Memory, _cond: bool, _is_16: bool) {
    let res = d.read_byte(memory, &self.registers) ^ s.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn jp(&mut self, arg: Location, memory: &mut Memory, cond: bool, _is_16: bool) {
    if !cond {
      return;
    }

    let addr = arg.read_word(memory, &self.registers);
    // println!("Jumping to {:04X}", addr);

    self.registers.pc = addr;
  }

  pub fn to16bits_signed_offset(&mut self, arg: u8) -> u16 {
    let add = (arg & 0b10000000) == 0;
    let mask: u16 = if add { 0x0000 } else { 0xFF00 };

    return mask | (arg as u16);
  }

  pub fn jr(&mut self, arg: Location, memory: &mut Memory, cond: bool, _is_16: bool) {
    if !cond {
      return;
    }

    let byte = arg.read_byte(memory, &self.registers);
    self.registers.pc = self.registers.pc.wrapping_add(self.to16bits_signed_offset(byte));
  }

  pub fn adda(&mut self, sp: Location, arg: Location, memory: &mut Memory, _cond: bool, _is_16: bool) {
    let sp_val = sp.read_word(memory, &self.registers);
    let offset = arg.read_byte(memory, &self.registers);
    let val = self.to16bits_signed_offset(offset);

    let res = self.registers.sp.wrapping_add(val);

    if (sp_val & 0x0F) + (val & 0x0F) > 0xF {
      self.registers.set_h();
    } else {
      self.registers.reset_h();
    }

    if (sp_val & 0xFF) + (val & 0xFF) > 0xFF {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }

    self.registers.reset_z();
    self.registers.reset_n();

    self.registers.sp = res;
  }

  pub fn lda(&mut self, d: Location, s1: Location, s2: Location, memory: &mut Memory, _cond: bool, _is_16: bool) {
    let sp_val = s1.read_word(memory, &self.registers);
    let offset = s2.read_byte(memory, &self.registers);
    let val = self.to16bits_signed_offset(offset);

    self.registers.reset_z();
    self.registers.reset_n();

    let res = self.registers.sp.wrapping_add(val);

    if (sp_val & 0x0F) + (val & 0x0F) > 0xF {
      self.registers.set_h();
    } else {
      self.registers.reset_h();
    }

    if (sp_val & 0xFF) + (val & 0xFF) > 0xFF {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }

    self.ld16(d, Location::from_immediate(res), memory);
  }

  fn ld8(&mut self, mut d: Location, s: Location, memory: &mut Memory) {
    let byte = s.read_byte(memory, &self.registers);
    d.write_byte(memory, &mut self.registers, byte);
  }

  fn ld16(&mut self, mut d: Location, s: Location, memory: &mut Memory) {
    let word = s.read_word(memory, &self.registers);
    d.write_word(memory, &mut self.registers, word);
  }

  pub fn rr(&mut self, mut loc: Location, memory: &mut Memory, through_carry: bool, a_shift: bool, force_reset_z: bool) {
    let byte = loc.read_byte(memory, &self.registers);

    let mask = if a_shift {
      if byte & 0x80 != 0 { 0x80 } else { 0x00 }
    } else if (through_carry && self.registers.c_set()) || 
              (!through_carry && (byte & 0x01) != 0) {
      0x80
    } else {
      0x00
    };

    let res = (byte >> 1) | mask;

    loc.write_byte(memory, &mut self.registers, res);

    if res == 0 && !force_reset_z { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    if byte & 0x01 != 0 {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }
  }

  pub fn rl(&mut self, mut loc: Location, memory: &mut Memory, through_carry: bool, a_shift: bool, force_reset_z: bool) {
    let byte = loc.read_byte(memory, &self.registers);

    let mask = if !a_shift && 
      (through_carry && self.registers.c_set() || !through_carry && (byte & 0x80) != 0) { 
      0x01
    } else { 
      0x00 
    };

    let res = (byte << 1) | mask;

    loc.write_byte(memory, &mut self.registers, res);

    if res == 0 && !force_reset_z { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    if byte & 0x80 != 0 {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }
  }

  fn srl(&mut self, mut loc: Location, memory: &mut Memory) {
    let byte = loc.read_byte(memory, &self.registers);

    let res = byte >> 1;
    loc.write_byte(memory, &mut self.registers, res);

    self.registers.reset_n();
    self.registers.reset_h();
    if res == 0 {
      self.registers.set_z();
    } else {
      self.registers.reset_z();
    }
    if byte & 0x01 == 0x01 {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }
  }

  fn exec_pref(&mut self, instr: u8, memory: &mut Memory) {
    let src: Location = self.get_ld_arithmetic_bit_source(instr);
    let mut dest: Location = self.get_ld_arithmetic_bit_dest(instr);
    match instr {
      0x00..=0x07 => {
        // RLC
        self.rl(dest, memory, false, false, false);
      },
      0x08..=0x0F => {
        // RRC
        self.rr(dest, memory, false, false, false);
      },
      0x10..=0x17 => {
        // RL through carry
        self.rl(dest, memory, true, false, false);
      },
      0x18..=0x1F => {
        // RR through carry
        self.rr(dest, memory, true, false, false);
      },
      0x20..=0x27 => {
        // SLA
        self.rl(dest, memory, false, true, false);
      },
      0x28..=0x2F => {
        // SRA
        self.rr(dest, memory, false, true, false);
      },
      0x30..=0x37 => {
        // SWAP
        let byte = src.read_byte(memory, &self.registers);
        let h = byte & 0xF0;
        let l = byte & 0x0F;

        let res = (h >> 4) | (l << 4);
        dest.write_byte(memory, &mut self.registers, res);

        if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
        self.registers.reset_n();
        self.registers.reset_h();
        self.registers.reset_c();
      },
      0x38..=0x3F => {
        // SRL
        self.srl(dest, memory);
      },
      0x40..=0x7F => {
        // BIT
        let index = (instr - 0x40) / 0x08;
        let pattern = 0b00000001 << index;
        let byte = src.read_byte(memory, &self.registers);
        let set: bool = (byte & pattern) != 0;

        if !set { self.registers.set_z(); } else { self.registers.reset_z(); }
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
    };
  }
}

#[cfg(test)]
mod tests {
  use crate::cpu::Cpu;
  use crate::registers::RegisterName;
  use crate::registers::Registers;
  use crate::memory::Memory;
  use crate::memory_utils::Location;
  use crate::opcodes;

  #[test]
  fn ld8() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    assert_eq!(cpu.registers.pc, 0x0100);

    assert_eq!(0x01, cpu.registers.read_byte(RegisterName::A));
    assert_eq!(0x13, cpu.registers.read_byte(RegisterName::C));

    cpu.ld(Location::from_immediate_register(RegisterName::A), Location::from_immediate_register(RegisterName::C), &mut memory, true, false);

    assert_eq!(0x13, cpu.registers.read_byte(RegisterName::A));

    cpu.ld(Location::from_address(0x00), Location::from_immediate_register(RegisterName::C), &mut memory, true, false);
    assert_eq!(0x13, memory[0x00]);

    assert_eq!(0x00, memory[0x01]);
    cpu.ld(Location::from_immediate_register(RegisterName::A), Location::from_address(0x01), &mut memory, true, false);
    assert_eq!(0x00, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;

    memory[0x00] = 0x46;
    cpu.registers.pc = 0x00;

    cpu.tick(&mut memory, false);

    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::B));

    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;
    cpu.registers.pc = 0x00;
    memory[0x00] = 0x2A;

    cpu.tick(&mut memory, false);
    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::A));
    assert_eq!(0x1235, cpu.registers.hl);

    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;
    cpu.registers.pc = 0x00;
    memory[0x00] = 0x3A;

    cpu.tick(&mut memory, false);
    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::A));
    assert_eq!(0x1233, cpu.registers.hl);

    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;
    cpu.registers.pc = 0x00;
    memory[0x00] = 0x32;
    cpu.registers.af = 0x00;

    cpu.tick(&mut memory, false);
    assert_eq!(0x00, memory[0x1234]);
    assert_eq!(0x1233, cpu.registers.hl);
  }

  #[test]
  fn push_pop() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.sp = 0x10;

    cpu.registers.bc = 0x1234;
    cpu.push(Location::from_immediate_register(RegisterName::Bc), &mut memory, true, true);

    assert_eq!(0x0E, cpu.registers.sp);
    assert_eq!(0x34, memory[0x0E]);
    assert_eq!(0x12, memory[0x0F]);

    cpu.registers.de = 0x00;
    cpu.pop(Location::from_immediate_register(RegisterName::De), &mut memory, true, true);
    assert_eq!(0x1234, cpu.registers.de);
    assert_eq!(0x10, cpu.registers.sp);

    cpu.registers.sp = 0x80;
    cpu.registers.bc = 0x1200;
    cpu.push(Location::from_immediate_register(RegisterName::Bc), &mut memory, true, true);
    cpu.pop(Location::from_immediate_register(RegisterName::Af), &mut memory, true, true);
    cpu.push(Location::from_immediate_register(RegisterName::Af), &mut memory, true, true);
    cpu.pop(Location::from_immediate_register(RegisterName::De), &mut memory, true, true);

    cpu.registers.pc = 0x00;
    memory[0x00] = 0x79;
    cpu.tick(&mut memory, false);
    assert_eq!(0x01, cpu.registers.pc);
    memory[0x01] = 0xE6;
    memory[0x02] = 0xF0;
    cpu.tick(&mut memory, false);
    assert_eq!(0x03, cpu.registers.pc);
    memory[0x03] = 0xBB;
    cpu.tick(&mut memory, false);
    assert_eq!(0x04, cpu.registers.pc);

    assert!(cpu.registers.z_set());
  }

  #[test]
  fn add() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.af = 0x0312;
    assert_eq!(0x03, cpu.registers.read_byte(RegisterName::A));
    cpu.add(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0x02), &mut memory, true, false);
    assert_eq!(0x05, cpu.registers.read_byte(RegisterName::A));
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.add(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0xFB), &mut memory, true, false);
    assert_eq!(0x00, cpu.registers.read_byte(RegisterName::A));
    assert!(cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());

    cpu.adc(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0xFA), &mut memory, true, false);
    assert_eq!(0xFB, cpu.registers.read_byte(RegisterName::A));
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.registers.set_c();
    cpu.adc(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0x04), &mut memory, true, false);
    assert_eq!(0x00, cpu.registers.read_byte(RegisterName::A));
    assert!(cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());
  }

  #[test]
  fn sub() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.af = 0x0312;
    assert_eq!(0x03, cpu.registers.read_byte(RegisterName::A));
    cpu.sub(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0x02), &mut memory, true, false);
    assert_eq!(0x01, cpu.registers.read_byte(RegisterName::A));
    assert!(!cpu.registers.z_set());
    assert!(cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.sub(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0x02), &mut memory, true, false);
    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::A));
    assert!(!cpu.registers.z_set());
    assert!(cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());

    cpu.sbc(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0xFE), &mut memory, true, false);
    assert_eq!(0x00, cpu.registers.read_byte(RegisterName::A));
    assert!(cpu.registers.z_set());
    assert!(cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.registers.set_c();
    cpu.sbc(Location::from_immediate_register(RegisterName::A), Location::from_immediate(0x00), &mut memory, true, false);
    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::A));
    assert!(!cpu.registers.z_set());
    assert!(cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());
  }

  #[test]
  fn call_ret() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.sp = 0xFF00;
    cpu.registers.pc = 0x1234;
    memory[0x1234] = 0xCD;
    memory[0x1235] = 0x78;
    memory[0x1236] = 0x56;
    memory[0x5678] = 0xC9;

    cpu.tick(&mut memory, false);
    assert_eq!(0x5678, cpu.registers.pc);
    assert_eq!(0xFEFE, cpu.registers.sp);

    cpu.tick(&mut memory, false);
    assert_eq!(0x1237, cpu.registers.pc);
    assert_eq!(0xFF00, cpu.registers.sp);
  }

  #[test]
  fn call_reti() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.ime = false;
    cpu.registers.sp = 0xFF00;
    cpu.registers.pc = 0x1234;
    memory[0x1234] = 0xCD;
    memory[0x1235] = 0x78;
    memory[0x1236] = 0x56;
    memory[0x5678] = 0xD9;

    cpu.tick(&mut memory, false);
    assert_eq!(0x5678, cpu.registers.pc);
    assert_eq!(0xFEFE, cpu.registers.sp);

    cpu.tick(&mut memory, false);
    assert_eq!(0x1237, cpu.registers.pc);
    assert_eq!(0xFF00, cpu.registers.sp);
    assert!(cpu.ime);
  }

  #[test]
  fn jp() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    memory[0x1234] = 0xC3;
    memory[0x1235] = 0xAD;
    memory[0x1236] = 0xDE;
    cpu.registers.pc = 0x1234;

    cpu.tick(&mut memory, false);
    assert_eq!(0xDEAD, cpu.registers.pc);

    cpu.registers.set_z();
    memory[0x1234] = 0xC2;
    memory[0x1235] = 0xAD;
    memory[0x1236] = 0xDE;
    cpu.registers.pc = 0x1234;

    cpu.tick(&mut memory, false);
    assert_eq!(0x1237, cpu.registers.pc);
  }

  #[test]
  fn jr() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    memory[0x0480] = 0x18;
    memory[0x0481] = 0x03;
    cpu.registers.pc = 0x0480;

    cpu.tick(&mut memory, false);
    assert_eq!(0x0485, cpu.registers.pc);

    memory[0x0480] = 0x18;
    memory[0x0481] = 0b11111101;
    cpu.registers.pc = 0x0480;

    cpu.tick(&mut memory, false);
    assert_eq!(0x047F, cpu.registers.pc);
  }

  #[test]
  fn rr() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.set_c();
    cpu.registers.write_byte(RegisterName::A, 0b00010111);
    cpu.rra(&mut memory, true, false);

    assert!(cpu.registers.c_set());
    assert_eq!(0b10001011, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.reset_c();
    cpu.registers.write_byte(RegisterName::A, 0b00010111);
    cpu.rra(&mut memory, true, false);

    assert!(cpu.registers.c_set());
    assert_eq!(0b00001011, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.reset_c();
    cpu.registers.write_byte(RegisterName::A, 0b11011101);
    cpu.rra(&mut memory, true, false);

    assert!(cpu.registers.c_set());
    assert_eq!(0b01101110, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.set_c();
    cpu.registers.write_byte(RegisterName::A, 0b11011101);
    cpu.rra(&mut memory, true, false);

    assert!(cpu.registers.c_set());
    assert_eq!(0b11101110, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.set_c();
    cpu.registers.write_byte(RegisterName::A, 0b10111000);
    cpu.rr(Location::from_immediate_register(RegisterName::A), &mut memory, false, true, false);

    assert!(!cpu.registers.c_set());
    assert_eq!(0b11011100, cpu.registers.read_byte(RegisterName::A));
  }

  #[test]
  fn rl() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.reset_c();
    cpu.registers.write_byte(RegisterName::A, 0b10001000);
    cpu.rlca(&mut memory, true, false);

    assert!(cpu.registers.c_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert_eq!(0b00010001, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.reset_c();
    cpu.registers.write_byte(RegisterName::A, 0b10110001);
    cpu.rl(Location::from_immediate_register(RegisterName::A), &mut memory, false, true, false);

    assert!(cpu.registers.c_set());
    assert_eq!(0b01100010, cpu.registers.read_byte(RegisterName::A));
  }

  #[test]
  fn swap() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.write_byte(RegisterName::A, 0x1F);
    cpu.exec_pref(0x37, &mut memory);

    assert_eq!(0xF1, cpu.registers.read_byte(RegisterName::A));
    assert!(!cpu.registers.z_set());

    cpu.registers.write_byte(RegisterName::A, 0x00);
    cpu.exec_pref(0x37, &mut memory);

    assert_eq!(0x00, cpu.registers.read_byte(RegisterName::A));
    assert!(cpu.registers.z_set());
  }

  #[test]
  fn srl() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.set_c();
    cpu.registers.write_byte(RegisterName::A, 0b10001111);
    cpu.srl(Location::from_immediate_register(RegisterName::A), &mut memory);

    assert!(cpu.registers.c_set());
    assert_eq!(0b01000111, cpu.registers.read_byte(RegisterName::A));

    cpu.registers.reset_c();
    cpu.registers.write_byte(RegisterName::A, 0b10001111);
    cpu.srl(Location::from_immediate_register(RegisterName::A), &mut memory);

    assert!(cpu.registers.c_set());
    assert_eq!(0b01000111, cpu.registers.read_byte(RegisterName::A));
  }

  #[test]
  fn bit() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.set_z();
    cpu.registers.write_byte(RegisterName::A, 0b0100);
    cpu.exec_pref(0x57, &mut memory);
    assert!(!cpu.registers.z_set());

    cpu.registers.reset_z();
    cpu.exec_pref(0x4F, &mut memory);
    assert!(cpu.registers.z_set());
  }

  #[test]
  fn set() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.set_z();
    cpu.registers.write_byte(RegisterName::A, 0b0100);
    cpu.exec_pref(0xCF, &mut memory);
    assert_eq!(0b0110, cpu.registers.read_byte(RegisterName::A));

    cpu.exec_pref(0xCF, &mut memory);
    assert_eq!(0b0110, cpu.registers.read_byte(RegisterName::A));
  }

  #[test]
  fn res() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.set_z();
    cpu.registers.write_byte(RegisterName::A, 0b0100);
    cpu.exec_pref(0x97, &mut memory);
    assert_eq!(0b0000, cpu.registers.read_byte(RegisterName::A));

    cpu.exec_pref(0x97, &mut memory);
    assert_eq!(0b0000, cpu.registers.read_byte(RegisterName::A));
  }

  #[test]
  fn daa() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.reset_n();
    cpu.registers.reset_c();
    cpu.registers.reset_h();
    cpu.registers.write_byte(RegisterName::A, 0b00111100);
    cpu.daa(&mut memory, true, false);

    assert_eq!(0b01000010, cpu.registers.read_byte(RegisterName::A));
  }

  #[test]
  fn inc() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.sp = 0xFFFF;
    cpu.inc(Location::from_immediate_register(RegisterName::Sp), &mut memory, true, true);

    assert_eq!(0x0000, cpu.registers.sp);

    cpu.registers.sp = 0x1000;
    cpu.inc(Location::from_immediate_register(RegisterName::Sp), &mut memory, true, true);

    assert_eq!(0x1001, cpu.registers.sp);
  }

  #[test]
  fn lda() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDEAD;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate(0x01),
            &mut memory, true, true);
    assert_eq!(0xDEAE, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDEAD;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate(0x03),
            &mut memory, true, true);
    assert_eq!(0xDEB0, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDEFE;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate_byte(0x02),
            &mut memory, true, true);
    assert_eq!(0xDF00, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDEAD;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate_byte(0xFF),
            &mut memory, true, true);
    assert_eq!(0xDEAC, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDE11;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate_byte(0b11101111),
            &mut memory, true, true);
    assert_eq!(0xDE00, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(cpu.registers.h_set());
    assert!(cpu.registers.c_set());

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDE20;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate_byte(0b11111011),
            &mut memory, true, true);
    assert_eq!(0xDE1B, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(cpu.registers.c_set());

    cpu.registers.hl = 0x1234;
    cpu.registers.sp = 0xDE00;
    cpu.lda(Location::from_immediate_register(RegisterName::Hl),
            Location::from_immediate_register(RegisterName::Sp),
            Location::from_immediate_byte(0b11111111),
            &mut memory, true, true);
    assert_eq!(0xDDFF, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.registers.hl = 0x0000;
    cpu.registers.sp = 0x0000;
    cpu.registers.pc = 0x1234;
    memory[0x1234] = 0xF8;
    memory[0x1235] = 0x01;
    memory[0x1236] = 0x00;
    cpu.tick(&mut memory, false);
    assert_eq!(0x0001, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());

    cpu.registers.hl = 0x0000;
    cpu.registers.sp = 0x0000;
    cpu.registers.pc = 0x1234;
    memory[0x1234] = 0xF8;
    memory[0x1235] = 0xFF;
    memory[0x1236] = 0x00;
    cpu.tick(&mut memory, false);
    assert_eq!(0xFFFF, cpu.registers.hl);
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
    assert!(!cpu.registers.h_set());
    assert!(!cpu.registers.c_set());
  }

  #[test]
  fn adda() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::empty();

    cpu.registers.sp = 0xFFFF;
    cpu.adda(Location::from_immediate_register(RegisterName::Sp), 
             Location::from_immediate_byte(0xFF), 
             &mut memory, true, true);

    assert_eq!(0xFFFE, cpu.registers.sp);
    assert!(cpu.registers.c_set());
    assert!(cpu.registers.h_set());
    assert!(!cpu.registers.z_set());
    assert!(!cpu.registers.n_set());
  }
}
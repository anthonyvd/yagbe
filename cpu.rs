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
    //println!("push_byte {:04X}", self.registers.sp);
    self.registers.sp = self.registers.sp.wrapping_sub(1);
    Location::from_address(self.registers.sp).write_byte(memory, &mut self.registers, b);
  }

  fn pop_byte(&mut self, memory: &mut Memory) -> u8 {
    //println!("pop_byte {:04X}", self.registers.sp);
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

  pub fn tick(&mut self, memory: &mut Memory) {
    if self.cycles_stalled > 0 {
      self.cycles_stalled = self.cycles_stalled - 1;
      return;
    }
    // This currently runs each instruction in a single simulated TCycle, which isn't accurate.
/*
    if self.registers.pc == 0x27CD || self.registers.pc == 0x27CE || self.registers.pc == 0x27CF || self.registers.pc == 0x27D0 {
      self.dump_registers();
      for i in 0x8000..0x8000 + 16 {
        println!("byte {:04X}", memory[i]);
      }
    }
*/
/*
    // Also see pandocs about timing
    if (memory[0xFF0F] | 0x01 == 1) && self.ime && (memory[0xFFFF] & 0x01 == 1) {
      memory[0xFF0F] = memory[0xFF0F] & 0xFE;
      self.ime = false;
      println!("INTV {:04X}", Location::from_address(0x40).read_word(memory, &self.registers));
      self.call(Location::from_immediate(0x40), memory, true, true);
    }
*/
/*
    println!("{:04X} -> {:02X} {:02X}, {:02X} ({:02X}, {:04X}, {:02X}, {:02X} {})", 
      self.registers.pc, memory[self.registers.pc], memory[self.registers.pc + 1], memory[self.registers.pc + 2],
      self.registers.read_byte(RegisterName::A), self.registers.read_word(RegisterName::Hl),
      self.registers.read_byte(RegisterName::B), self.registers.read_byte(RegisterName::C),
      self.registers.z_set());
*/
    let instr: u8 = self.pc_read(memory);
    
    // If it's the CB prefix byte, fetch the next one
    if instr == 0xCB {
      let arg = self.pc_read(memory);
      self.exec_pref(arg, memory);
      self.cycles_stalled = 8;
    } else {
      self.cycles_stalled = opcodes::exec_unpref(instr, memory, self);
    }
  }

  pub fn call(&mut self, arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond {
      return;
    }

    let pc = self.registers.pc;
    self.push_word(pc, memory);
    self.registers.pc = arg.read_word(memory, &self.registers);
  }

  pub fn rst(&mut self, arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
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
    // TODO: carry flags
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

  pub fn add(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if is_16 {
      let left = d.read_word(memory, &self.registers);
      let right = s.read_word(memory, &self.registers);

      let res = left.wrapping_add(right);

      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.reset_n();
      // TODO: H flag      
      if 0xFFFF - left < right {
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
      // TODO: H flag
      if 0xFF - left < right {
        self.registers.set_c();
      } else {
        self.registers.reset_c();
      }

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn sub(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if is_16 {
      unimplemented!();
    } else {
      let left = d.read_byte(memory, &self.registers);
      let right = s.read_byte(memory, &self.registers);

      let res = left.wrapping_sub(right);
        
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.set_n();
      // TODO: carry flags
      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn adc(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    let res = d.read_byte(memory, &self.registers)
      .wrapping_add(s.read_byte(memory, &self.registers))
      .wrapping_add(if self.registers.c_set() { 1 } else { 0 });

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    // TODO: carry flags
    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn sbc(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    let res = d.read_byte(memory, &self.registers)
      .wrapping_sub(s.read_byte(memory, &self.registers))
      .wrapping_sub(if self.registers.c_set() { 1 } else { 0 });
      
    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.set_n();
    // TODO: carry flags
    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn dec(&mut self, mut d: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("DEC has a false cond"); }

    if is_16 {
      let res = d.read_word(memory, &self.registers).wrapping_sub(1);
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.set_n();
      // TODO: there should be a possible H flag set here

      d.write_word(memory, &mut self.registers, res);
    } else {
      let res = d.read_byte(memory, &self.registers).wrapping_sub(1);
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.set_n();
      // TODO: there should be a possible H flag set here

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn inc(&mut self, mut d: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("INC has a false cond"); }

    if is_16 {
      let res = d.read_word(memory, &self.registers).wrapping_add(1);
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.reset_n();
      // TODO: there should be a possible H flag set here

      d.write_word(memory, &mut self.registers, res);
    } else {
      let res = d.read_byte(memory, &self.registers).wrapping_add(1);
      if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
      self.registers.reset_n();
      // TODO: there should be a possible H flag set here

      d.write_byte(memory, &mut self.registers, res);
    }
  }

  pub fn ret(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond {
      return;
    }

    self.registers.pc = self.pop_word(memory);
  }

  pub fn reti(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("RETI has a false cond"); }

    self.ime = true;
    self.ret(memory, cond, is_16)
  }

  pub fn halt(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    // TODO: interrupts?!
  }

  pub fn stop(&mut self, d: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    // d is ignored
    unimplemented!();
  }

  pub fn ccf(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    if self.registers.c_set() {
      self.registers.reset_c();
    } else {
      self.registers.set_c();
    }
  }

  pub fn scf(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    self.registers.set_c();
  }

  pub fn cpl(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    let byte = self.registers.read_byte(RegisterName::A);
    self.registers.write_byte(RegisterName::A, byte ^ 0xFF);
    self.registers.set_n();
    self.registers.set_h();
  }

  pub fn daa(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    unimplemented!();
  }

  pub fn rra(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    
    let mask = if self.registers.c_set() { 0x00 } else { 0x80 };

    let byte = self.registers.read_byte(RegisterName::A);
    if byte & 0x01 != 0 {
      self.registers.set_c();
    } else {
      self.registers.reset_c();
    }

    let res = (byte >> 1) | mask;

    self.registers.write_byte(RegisterName::A, res);

    self.registers.reset_z();
    self.registers.reset_n();
    self.registers.reset_h();
  }

  pub fn rla(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    unimplemented!();
  }

  pub fn rrca(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    unimplemented!();
  }

  pub fn rlca(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    unimplemented!();
  }

  pub fn nop(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    // Might burn cycles
  }

  pub fn ei(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    self.ime = true;
  }

  pub fn di(&mut self, memory: &mut Memory, cond: bool, is_16: bool) {
    self.ime = false;
  }

  pub fn ld(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("LD is cond"); }

    if is_16 {
      self.ld16(d, s, memory)
    } else {
      self.ld8(d, s, memory)
    }
  }

  pub fn ldh(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond { panic!("LDH is cond"); }
    if is_16 { panic!("LDH is 16"); }

    self.ld8(d, s, memory);
  }

  pub fn xor(&mut self, mut d: Location, s: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    let res = d.read_byte(memory, &self.registers) ^ s.read_byte(memory, &self.registers);

    if res == 0 { self.registers.set_z(); } else { self.registers.reset_z(); }
    self.registers.reset_n();
    self.registers.reset_h();
    self.registers.reset_c();

    d.write_byte(memory, &mut self.registers, res);
  }

  pub fn jp(&mut self, arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond {
      return;
    }

    let addr = arg.read_word(memory, &self.registers);
    // println!("Jumping to {:04X}", addr);

    self.registers.pc = addr;
  }

  pub fn jr(&mut self, arg: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    if !cond {
      return;
    }

    let byte = arg.read_byte(memory, &self.registers);

    let new_pc = self.signed_offset16(byte, self.registers.pc);

    self.registers.pc = new_pc;
  }

  pub fn lda(&mut self, mut d: Location, s1: Location, s2: Location, memory: &mut Memory, cond: bool, is_16: bool) {
    let sp_val = s1.read_word(memory, &self.registers);
    let val = self.signed_offset16(s2.read_byte(memory, &self.registers), sp_val);

    self.ld16(d, Location::from_immediate(val), memory)
  }

  fn ld8(&mut self, mut d: Location, s: Location, memory: &mut Memory) {
    let byte = s.read_byte(memory, &self.registers);
    d.write_byte(memory, &mut self.registers, byte);
  }

  fn ld16(&mut self, mut d: Location, s: Location, memory: &mut Memory) {
    let word = s.read_word(memory, &self.registers);
    d.write_word(memory, &mut self.registers, word);
  }

  fn signed_offset16(&self, byte: u8, val: u16) -> u16 {
    let add = (byte & 0b10000000) == 0;
    let off: u16;
    if add { off = byte as u16; } else { off = (!byte).wrapping_add(1) as u16; };

    if add {
      return val.wrapping_add(off);
    } else {
      return val.wrapping_sub(off);
    }
  }

  fn exec_pref(&mut self, instr: u8, memory: &mut Memory) {
    let src: Location = self.get_ld_arithmetic_bit_source(instr);
    let mut dest: Location = self.get_ld_arithmetic_bit_dest(instr);
    match instr {/*
      0x00..=0x07 => {

      },
      0x08..=0x0F => {

      },
      0x10..=0x17 => {

      },*/
      0x18..=0x1F => {
        // RR through carry
        let mask = if self.registers.c_set() { 0x00 } else { 0x80 };

        let byte = src.read_byte(memory, &self.registers);
        if byte & 0x01 != 0 {
          self.registers.set_c();
        } else {
          self.registers.reset_c();
        }

        let res = (byte >> 1) | mask;

        dest.write_byte(memory, &mut self.registers, res);
        self.registers.reset_n();
        self.registers.reset_h();
        if res == 0 {
          self.registers.set_z();
        } else {
          self.registers.reset_z();
        }
      },
      /*
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
      },
      0x38..=0x3F => {
        // SRL
        let byte = src.read_byte(memory, &self.registers);

        let res = byte >> 1;
        dest.write_byte(memory, &mut self.registers, res);

        if res == 0 { self.registers.set_z(); }
        self.registers.reset_n();
        self.registers.reset_h();
        if res == 0 {
          self.registers.set_z();
        } else {
          self.registers.reset_z();
        }

        // TODO: carry flag?
      },
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
}

#[cfg(test)]
mod tests {
  use crate::Cpu;
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

    cpu.tick(&mut memory);

    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::B));


    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;
    cpu.registers.pc = 0x00;
    memory[0x00] = 0x2A;

    cpu.tick(&mut memory);
    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::A));
    assert_eq!(0x1235, cpu.registers.hl);

    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;
    cpu.registers.pc = 0x00;
    memory[0x00] = 0x3A;

    cpu.tick(&mut memory);
    assert_eq!(0xFF, cpu.registers.read_byte(RegisterName::A));
    assert_eq!(0x1233, cpu.registers.hl);

    cpu.registers.hl = 0x1234;
    memory[0x1234] = 0xFF;
    cpu.registers.pc = 0x00;
    memory[0x00] = 0x32;
    cpu.registers.af = 0x00;

    cpu.tick(&mut memory);
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
    assert!(!cpu.registers.h_set());
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

    cpu.tick(&mut memory);
    assert_eq!(0x5678, cpu.registers.pc);
    assert_eq!(0xFEFE, cpu.registers.sp);

    cpu.tick(&mut memory);
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

    cpu.tick(&mut memory);
    assert_eq!(0x5678, cpu.registers.pc);
    assert_eq!(0xFEFE, cpu.registers.sp);

    cpu.tick(&mut memory);
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

    cpu.tick(&mut memory);
    assert_eq!(0xDEAD, cpu.registers.pc);
  }
}
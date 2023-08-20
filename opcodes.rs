use crate::registers::RegisterName;
use crate::memory_utils::Location;
use crate::memory::Memory;
use crate::cpu::Cpu;

pub fn exec_unpref(instr: u8, memory: &mut Memory, cpu: &mut Cpu) -> u8 {
	match instr {
		0x00 => {
			cpu.nop(
				memory, true, false);
			return 4;
		},
		0x01 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::Bc),
				Location::from_immediate(w0),
				memory, true, true);
			return 12;
		},
		0x02 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Bc)),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 8;
		},
		0x03 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::Bc),
				memory, true, true);
			return 8;
		},
		0x04 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x05 => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x06 => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x07 => {
			cpu.rlca(
				memory, true, false);
			return 4;
		},
		0x08 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_address(w0),
				Location::from_immediate_register(RegisterName::Sp),
				memory, true, true);
			return 20;
		},
		0x09 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::Hl),
				Location::from_immediate_register(RegisterName::Bc),
				memory, true, true);
			return 8;
		},
		0x0A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Bc)),
				memory, true, false);
			return 8;
		},
		0x0B => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::Bc),
				memory, true, true);
			return 8;
		},
		0x0C => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x0D => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x0E => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x0F => {
			cpu.rrca(
				memory, true, false);
			return 4;
		},
		0x10 => {
			let b0 = cpu.pc_read(memory);
			cpu.stop(
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 4;
		},
		0x11 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::De),
				Location::from_immediate(w0),
				memory, true, true);
			return 12;
		},
		0x12 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::De)),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 8;
		},
		0x13 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::De),
				memory, true, true);
			return 8;
		},
		0x14 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x15 => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x16 => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x17 => {
			cpu.rla(
				memory, true, false);
			return 4;
		},
		0x18 => {
			let b0 = cpu.pc_read(memory);
			cpu.jr(
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 12;
		},
		0x19 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::Hl),
				Location::from_immediate_register(RegisterName::De),
				memory, true, true);
			return 8;
		},
		0x1A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::De)),
				memory, true, false);
			return 8;
		},
		0x1B => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::De),
				memory, true, true);
			return 8;
		},
		0x1C => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x1D => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x1E => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x1F => {
			cpu.rra(
				memory, true, false);
			return 4;
		},
		0x20 => {
			let b0 = cpu.pc_read(memory);
			cpu.jr(
				Location::from_immediate_byte(b0),
				memory, !cpu.registers.z_set(), false);
			if !cpu.registers.z_set() { return 12; } else { return 8; }
		},
		0x21 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::Hl),
				Location::from_immediate(w0),
				memory, true, true);
			return 12;
		},
		0x22 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			cpu.registers.hl = cpu.registers.hl.wrapping_add(1);
			return 8;
		},
		0x23 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 8;
		},
		0x24 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x25 => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x26 => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x27 => {
			cpu.daa(
				memory, true, false);
			return 4;
		},
		0x28 => {
			let b0 = cpu.pc_read(memory);
			cpu.jr(
				Location::from_immediate_byte(b0),
				memory, cpu.registers.z_set(), false);
			if cpu.registers.z_set() { return 12; } else { return 8; }
		},
		0x29 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::Hl),
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 8;
		},
		0x2A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			cpu.registers.hl = cpu.registers.hl.wrapping_add(1);
			return 8;
		},
		0x2B => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 8;
		},
		0x2C => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x2D => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x2E => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x2F => {
			cpu.cpl(
				memory, true, false);
			return 4;
		},
		0x30 => {
			let b0 = cpu.pc_read(memory);
			cpu.jr(
				Location::from_immediate_byte(b0),
				memory, !cpu.registers.c_set(), false);
			if !cpu.registers.c_set() { return 12; } else { return 8; }
		},
		0x31 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::Sp),
				Location::from_immediate(w0),
				memory, true, true);
			return 12;
		},
		0x32 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			cpu.registers.hl = cpu.registers.hl.wrapping_sub(1);
			return 8;
		},
		0x33 => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::Sp),
				memory, true, true);
			return 8;
		},
		0x34 => {
			cpu.inc(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 12;
		},
		0x35 => {
			cpu.dec(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 12;
		},
		0x36 => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 12;
		},
		0x37 => {
			cpu.scf(
				memory, true, false);
			return 4;
		},
		0x38 => {
			let b0 = cpu.pc_read(memory);
			cpu.jr(
				Location::from_immediate_byte(b0),
				memory, cpu.registers.c_set(), false);
			if cpu.registers.c_set() { return 12; } else { return 8; }
		},
		0x39 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::Hl),
				Location::from_immediate_register(RegisterName::Sp),
				memory, true, true);
			return 8;
		},
		0x3A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			cpu.registers.hl = cpu.registers.hl.wrapping_sub(1);
			return 8;
		},
		0x3B => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::Sp),
				memory, true, true);
			return 8;
		},
		0x3C => {
			cpu.inc(
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x3D => {
			cpu.dec(
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x3E => {
			let b0 = cpu.pc_read(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0x3F => {
			cpu.ccf(
				memory, true, false);
			return 4;
		},
		0x40 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x41 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x42 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x43 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x44 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x45 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x46 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x47 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::B),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x48 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x49 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x4A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x4B => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x4C => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x4D => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x4E => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x4F => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::C),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x50 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x51 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x52 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x53 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x54 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x55 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x56 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x57 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::D),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x58 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x59 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x5A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x5B => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x5C => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x5D => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x5E => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x5F => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::E),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x60 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x61 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x62 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x63 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x64 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x65 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x66 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x67 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::H),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x68 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x69 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x6A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x6B => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x6C => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x6D => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x6E => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x6F => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::L),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x70 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 8;
		},
		0x71 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 8;
		},
		0x72 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 8;
		},
		0x73 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 8;
		},
		0x74 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 8;
		},
		0x75 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 8;
		},
		0x76 => {
			cpu.halt(
				memory, true, false);
			return 4;
		},
		0x77 => {
			cpu.ld(
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 8;
		},
		0x78 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x79 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x7A => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x7B => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x7C => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x7D => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x7E => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x7F => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x80 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x81 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x82 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x83 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x84 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x85 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x86 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x87 => {
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x88 => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x89 => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x8A => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x8B => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x8C => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x8D => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x8E => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x8F => {
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x90 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x91 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x92 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x93 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x94 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x95 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x96 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x97 => {
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0x98 => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0x99 => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0x9A => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0x9B => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0x9C => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0x9D => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0x9E => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0x9F => {
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0xA0 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0xA1 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0xA2 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0xA3 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0xA4 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0xA5 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0xA6 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0xA7 => {
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0xA8 => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0xA9 => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0xAA => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0xAB => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0xAC => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0xAD => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0xAE => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0xAF => {
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0xB0 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0xB1 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0xB2 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0xB3 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0xB4 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0xB5 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0xB6 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0xB7 => {
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0xB8 => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::B),
				memory, true, false);
			return 4;
		},
		0xB9 => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::C),
				memory, true, false);
			return 4;
		},
		0xBA => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::D),
				memory, true, false);
			return 4;
		},
		0xBB => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::E),
				memory, true, false);
			return 4;
		},
		0xBC => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::H),
				memory, true, false);
			return 4;
		},
		0xBD => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::L),
				memory, true, false);
			return 4;
		},
		0xBE => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(cpu.registers.read_word(RegisterName::Hl)),
				memory, true, false);
			return 8;
		},
		0xBF => {
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 4;
		},
		0xC0 => {
			cpu.ret(
				memory, !cpu.registers.z_set(), false);
			if !cpu.registers.z_set() { return 20; } else { return 8; }
		},
		0xC1 => {
			cpu.pop(
				Location::from_immediate_register(RegisterName::Bc),
				memory, true, true);
			return 12;
		},
		0xC2 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.jp(
				Location::from_immediate(w0),
				memory, !cpu.registers.z_set(), false);
			if !cpu.registers.z_set() { return 16; } else { return 12; }
		},
		0xC3 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.jp(
				Location::from_immediate(w0),
				memory, true, false);
			return 16;
		},
		0xC4 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.call(
				Location::from_immediate(w0),
				memory, !cpu.registers.z_set(), false);
			if !cpu.registers.z_set() { return 24; } else { return 12; }
		},
		0xC5 => {
			cpu.push(
				Location::from_immediate_register(RegisterName::Bc),
				memory, true, true);
			return 16;
		},
		0xC6 => {
			let b0 = cpu.pc_read(memory);
			cpu.add(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xC7 => {
			cpu.rst(
				Location::from_immediate(0x00),
				memory, true, false);
			return 16;
		},
		0xC8 => {
			cpu.ret(
				memory, cpu.registers.z_set(), false);
			if cpu.registers.z_set() { return 20; } else { return 8; }
		},
		0xC9 => {
			cpu.ret(
				memory, true, false);
			return 16;
		},
		0xCA => {
			let w0 = cpu.pc_read_word(memory);
			cpu.jp(
				Location::from_immediate(w0),
				memory, cpu.registers.z_set(), false);
			if cpu.registers.z_set() { return 16; } else { return 12; }
		},
		0xCC => {
			let w0 = cpu.pc_read_word(memory);
			cpu.call(
				Location::from_immediate(w0),
				memory, cpu.registers.z_set(), false);
			if cpu.registers.z_set() { return 24; } else { return 12; }
		},
		0xCD => {
			let w0 = cpu.pc_read_word(memory);
			cpu.call(
				Location::from_immediate(w0),
				memory, true, false);
			return 24;
		},
		0xCE => {
			let b0 = cpu.pc_read(memory);
			cpu.adc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xCF => {
			cpu.rst(
				Location::from_immediate(0x08),
				memory, true, false);
			return 16;
		},
		0xD0 => {
			cpu.ret(
				memory, !cpu.registers.c_set(), false);
			if !cpu.registers.c_set() { return 20; } else { return 8; }
		},
		0xD1 => {
			cpu.pop(
				Location::from_immediate_register(RegisterName::De),
				memory, true, true);
			return 12;
		},
		0xD2 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.jp(
				Location::from_immediate(w0),
				memory, !cpu.registers.c_set(), false);
			if !cpu.registers.c_set() { return 16; } else { return 12; }
		},
		0xD4 => {
			let w0 = cpu.pc_read_word(memory);
			cpu.call(
				Location::from_immediate(w0),
				memory, !cpu.registers.c_set(), false);
			if !cpu.registers.c_set() { return 24; } else { return 12; }
		},
		0xD5 => {
			cpu.push(
				Location::from_immediate_register(RegisterName::De),
				memory, true, true);
			return 16;
		},
		0xD6 => {
			let b0 = cpu.pc_read(memory);
			cpu.sub(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xD7 => {
			cpu.rst(
				Location::from_immediate(0x10),
				memory, true, false);
			return 16;
		},
		0xD8 => {
			cpu.ret(
				memory, cpu.registers.c_set(), false);
			if cpu.registers.c_set() { return 20; } else { return 8; }
		},
		0xD9 => {
			cpu.reti(
				memory, true, false);
			return 16;
		},
		0xDA => {
			let w0 = cpu.pc_read_word(memory);
			cpu.jp(
				Location::from_immediate(w0),
				memory, cpu.registers.c_set(), false);
			if cpu.registers.c_set() { return 16; } else { return 12; }
		},
		0xDC => {
			let w0 = cpu.pc_read_word(memory);
			cpu.call(
				Location::from_immediate(w0),
				memory, cpu.registers.c_set(), false);
			if cpu.registers.c_set() { return 24; } else { return 12; }
		},
		0xDE => {
			let b0 = cpu.pc_read(memory);
			cpu.sbc(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xDF => {
			cpu.rst(
				Location::from_immediate(0x18),
				memory, true, false);
			return 16;
		},
		0xE0 => {
			let b0 = cpu.pc_read(memory);
			cpu.ldh(
				Location::from_high_address(b0),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 12;
		},
		0xE1 => {
			cpu.pop(
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 12;
		},
		0xE2 => {
			cpu.ld(
				Location::from_high_address(cpu.registers.read_byte(RegisterName::C)),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 8;
		},
		0xE5 => {
			cpu.push(
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 16;
		},
		0xE6 => {
			let b0 = cpu.pc_read(memory);
			cpu.and(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xE7 => {
			cpu.rst(
				Location::from_immediate(0x20),
				memory, true, false);
			return 16;
		},
		0xE8 => {
			let b0 = cpu.pc_read(memory);
			cpu.adda(
				Location::from_immediate_register(RegisterName::Sp),
				Location::from_immediate_byte(b0),
				memory, true, true);
			return 16;
		},
		0xE9 => {
			cpu.jp(
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 4;
		},
		0xEA => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_address(w0),
				Location::from_immediate_register(RegisterName::A),
				memory, true, false);
			return 16;
		},
		0xEE => {
			let b0 = cpu.pc_read(memory);
			cpu.xor(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xEF => {
			cpu.rst(
				Location::from_immediate(0x28),
				memory, true, false);
			return 16;
		},
		0xF0 => {
			let b0 = cpu.pc_read(memory);
			cpu.ldh(
				Location::from_immediate_register(RegisterName::A),
				Location::from_high_address(b0),
				memory, true, false);
			return 12;
		},
		0xF1 => {
			cpu.pop(
				Location::from_immediate_register(RegisterName::Af),
				memory, true, true);
			return 12;
		},
		0xF2 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_high_address(cpu.registers.read_byte(RegisterName::C)),
				memory, true, false);
			return 8;
		},
		0xF3 => {
			cpu.di(
				memory, true, false);
			return 4;
		},
		0xF5 => {
			cpu.push(
				Location::from_immediate_register(RegisterName::Af),
				memory, true, true);
			return 16;
		},
		0xF6 => {
			let b0 = cpu.pc_read(memory);
			cpu.or(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xF7 => {
			cpu.rst(
				Location::from_immediate(0x30),
				memory, true, false);
			return 16;
		},
		0xF8 => {
			let b0 = cpu.pc_read(memory);
			cpu.lda(
				Location::from_immediate_register(RegisterName::Hl),
				Location::from_immediate_register(RegisterName::Sp),
				Location::from_immediate_byte(b0),
				memory, true, true);
			cpu.registers.hl = cpu.registers.hl.wrapping_add(1);
			return 12;
		},
		0xF9 => {
			cpu.ld(
				Location::from_immediate_register(RegisterName::Sp),
				Location::from_immediate_register(RegisterName::Hl),
				memory, true, true);
			return 8;
		},
		0xFA => {
			let w0 = cpu.pc_read_word(memory);
			cpu.ld(
				Location::from_immediate_register(RegisterName::A),
				Location::from_address(w0),
				memory, true, false);
			return 16;
		},
		0xFB => {
			cpu.ei(
				memory, true, false);
			return 4;
		},
		0xFE => {
			let b0 = cpu.pc_read(memory);
			cpu.cp(
				Location::from_immediate_register(RegisterName::A),
				Location::from_immediate_byte(b0),
				memory, true, false);
			return 8;
		},
		0xFF => {
			cpu.rst(
				Location::from_immediate(0x38),
				memory, true, false);
			return 16;
		},
		_ => {
			println!("Unknown Instruction: 0x{:02X}, SP: {:04X}, PC {:04X}", instr, cpu.registers.sp, cpu.registers.pc);
			unimplemented!();
		}
	}
}

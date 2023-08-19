import json

def generate():
	with open('opcodes.json', 'r') as f:
		parsed = json.load(f)

	unprefixed = parsed["unprefixed"]

	with open('opcodes.rs', 'w') as f:
		lines = [
			"use crate::registers::RegisterName;",
			"use crate::memory_utils::Location;",
			"use crate::memory::Memory;",
			"use crate::cpu::Cpu;",
			"",
			"pub fn exec_unpref(instr: u8, memory: &mut Memory, cpu: &mut Cpu) -> u8 {",
			"	match instr {",
		]

		for k, v in unprefixed.items():
			mnemonic = v["mnemonic"]

			if mnemonic[0:7].lower() == "illegal" or mnemonic[0:6].lower() == "prefix":
				continue

			lines.append("		" + k + " => {")

			operands = v["operands"]

			if mnemonic == "LD" and len(operands) == 3:
				mnemonic = "LDA"

			is_16bits = False
			cond = "true, "
			cycle_cond = ""

			idx = len(lines)
			pc_byte_count = 0
			pc_word_count = 0

			increment = False
			decrement = False

			lines.append("			cpu." + mnemonic.lower() + "(")
			for o in operands:
				if "increment" in o and o["increment"]:
					increment = True
				if "decrement" in o and o["decrement"]:
					decrement = True

				immediate = o['immediate']
				name = o["name"]
				if name == "n8" or name == "e8":
					lines.append("				Location::from_immediate_byte(b" + str(pc_byte_count) + "),")
					pc_byte_count = pc_byte_count + 1
				elif name == "n16":
					is_16bits = True
					lines.append("				Location::from_immediate(w" + str(pc_word_count) + "),")
					pc_word_count = pc_word_count + 1
				elif name == "a16":
					if o["immediate"]:
						lines.append("				Location::from_immediate(w" + str(pc_word_count) + "),")
					else:
						lines.append("				Location::from_address(w" + str(pc_word_count) + "),")
					pc_word_count = pc_word_count + 1
				elif name == "a8":
					lines.append("				Location::from_high_address(b" + str(pc_byte_count) + "),")
					pc_byte_count = pc_byte_count + 1
				elif name == "AF" or name == "BC" or name == "DE" or name == "HL" or name == "SP":
					if o["immediate"]:
						is_16bits = True
						lines.append("				Location::from_immediate_register(RegisterName::" + name.title() + "),")
					else:
						lines.append("				Location::from_address(cpu.registers.read_word(RegisterName::" + name.title() + ")),")
				elif name == "C" and (mnemonic == "CALL" or mnemonic == "RET" or mnemonic == "JP" or mnemonic == "JR"):
					cond = "cpu.registers." + name[0].lower() + "_set(), "
					cycle_cond = "cpu.registers." + name[0].lower() + "_set()"
				elif name == "A" or name == "F" or name == "B" or name == "C" or name == "D" or name == "E" or name == "H" or name == "L" or name == "S" or name == "P":
					if o["immediate"]:
						lines.append("				Location::from_immediate_register(RegisterName::" + name.title() + "),")
					else:
						lines.append("				Location::from_high_address(cpu.registers.read_byte(RegisterName::" + name.title() + ")),")
				elif name == "$00" or name == "$08" or name == "$10" or name == "$18" or name == "$20" or name == "$28" or name == "$30" or name == "$38":
					lines.append("				Location::from_immediate(0x" + name[1:] + "),")
				elif name == "Z" or name == "NZ" or name == "NC":
					if len(name) == 1:
						cond = "cpu.registers." + name[0].lower() + "_set(), "
						cycle_cond = "cpu.registers." + name[0].lower() + "_set()"
					else:
						cond = "!cpu.registers." + name[1].lower() + "_set(), "
						cycle_cond = "!cpu.registers." + name[1].lower() + "_set()"
				else:
					print("Can't parse operand: " + name)

			for i in range(0, pc_byte_count):
				lines.insert(idx, "			let b" + str(i) + " = cpu.pc_read(memory);")

			for i in range(0, pc_word_count):
				lines.insert(idx, "			let w" + str(i) + " = cpu.pc_read_word(memory);")

			is_16 = "true" 
			if not is_16bits:
				is_16 = "false"

			lines.append("				memory, " + cond + is_16 + ");")

			if increment:
				lines.append("			cpu.registers.hl = cpu.registers.hl.wrapping_add(1);")
			if decrement:
				lines.append("			cpu.registers.hl = cpu.registers.hl.wrapping_sub(1);")

			if len(v["cycles"]) == 1:
				lines.append("			return " + str(v["cycles"][0]) + ";")
			else:
				lines.append("			if " + cycle_cond + " { return " + str(v["cycles"][0]) + "; } else { return " + str(v["cycles"][1]) + "; }")


			lines.append("		},")

		lines.append("		_ => {")
		lines.append('			println!("Unknown Instruction: 0x{:02X}, SP: {:04X}, PC {:04X}", instr, cpu.registers.sp, cpu.registers.pc);')
		lines.append("			unimplemented!();")
		lines.append("		}")

		lines.append("	}")
		lines.append("}")

		f.writelines(s + '\n' for s in lines)

if __name__ == "__main__":
	generate()
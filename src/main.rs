use std::fs::File;
use std::io::Read;

use cpu::*;
use memory::*;

mod cpu;
mod memory;

fn main() {
	let mut cpu = CPU::new();
	let mut bus = MemoryBus::new();

	// Start in System mode
	cpu.cpsr.set_mode_bits(0x1f);

	let mut bios_data = Vec::<u8>::new();
	File::open("data/bios.gba").expect("Bios couldn't be opened!").read_to_end(&mut bios_data).unwrap();
	bus.load_bios(&bios_data);

	let mut cartridge_data = Vec::<u8>::new();
	if File::open("data/demos/hello.gba").expect("Cartridge couldn't be opened!").read_to_end(&mut cartridge_data).is_ok() {
		bus.load_cartridge(&cartridge_data);

		let mut thumb_mode = false;
		let mut pc = 0;
		while pc + 2 < bios_data.len() {
			if thumb_mode {
				decode_thumb(&mut thumb_mode, &mut bios_data, &mut pc);
			} else {
				decode_arm(&mut thumb_mode, &mut bios_data, &mut pc);
			}
		}
	} else {
		println!("Cartridge couldn't be read!");
	}
}

fn print_assembly_line(line: &String, pc: usize) {
	println!("{:#06X}| {}", pc, line);
}

fn decode_thumb(thumb_mode: &mut bool, data: &mut Vec<u8>, pc: &mut usize) {
	let bytes: [u8; 2] = [data[*pc], data[*pc + 1]];
	let instruction = u16::from_le_bytes(bytes);
	if (0xf800 & instruction) == 0x1800 {
		let op = if (0x0200 & instruction) > 0 { "ADD" } else { "SUB" };
		let i = (0x0400 & instruction) > 0;
		let rn = if i {
			format!("R{}", (0x01c0 & instruction) >> 6)
		} else {
			format!("#{:#X}", (0x01c0 & instruction) >> 6)
		};

		print_assembly_line(&format!("{} R{}, R{}, {}", op, instruction & 0x0003, (instruction & 0x001c) >> 3, rn), *pc);
	} else if (0xe000 & instruction) == 0x0000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "LSL",
			0x1 => op = "LSR",
			0x2 => op = "ASR",
			_ => panic!("ERROR!!!")
		}

		print_assembly_line(&format!("{} R{}, R{}, #{:#X}", op, instruction & 0x0003, (instruction & 0x001c) >> 3, (instruction & 0x07c0) >> 6), *pc);
	} else if (0xe000 & instruction) == 0x2000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "MOV",
			0x1 => op = "CMP",
			0x2 => op = "ADD",
			0x3 => op = "SUB",
			_ => panic!("ERROR!!!")
		}

		print_assembly_line(&format!("{} R{}, #{:#X}", op, (instruction & 0x0700) >> 8, instruction & 0x00ff), *pc);
	} else if (0xfc00 & instruction) == 0x4000 {
		let op;
		match (0x03c0 & instruction) >> 6 {
			0x0 => op = "AND",
			0x1 => op = "EOR",
			0x2 => op = "LSL",
			0x3 => op = "LSR",
			0x4 => op = "ASR",
			0x5 => op = "ADC",
			0x6 => op = "SBC",
			0x7 => op = "ROR",
			0x8 => op = "TST",
			0x9 => op = "NEG",
			0xa => op = "CMP",
			0xb => op = "CMN",
			0xc => op = "ORR",
			0xd => op = "MUL",
			0xe => op = "BIC",
			0xf => op = "MVN",
			_ => panic!("ERROR!!!")
		}

		print_assembly_line(&format!("{} R{}, R{}", op, instruction & 0x0007, (instruction & 0x001c) >> 3), *pc);
	} else if (0xfc00 & instruction) == 0x4400 {
		let op;
		match (0x0300 & instruction) >> 8 {
			0x0 => op = "ADD",
			0x1 => op = "CMP",
			0x2 => op = "MOV",
			0x3 => op = "BX",
			_ => panic!("ERROR!!!")
		}

		let rd = if op == "BX" {
			String::from("")
		} else {
			format!("R{}, ", (instruction & 0x001c) >> 3)
		};

		print_assembly_line(&format!("{} {}R{}", op, rd, instruction & 0x0007), *pc);

		if op == "BX" {
			*thumb_mode = !*thumb_mode;
			println!("ARM ------------------------------------------------------------------------------------------------------------------------ ARM");
		}
	} else if (0xf800 & instruction) == 0x4800 {
		print_assembly_line(&format!("LDR R{}, PC, #{:#X}", (instruction & 0x0700) >> 8, instruction & 0x00ff), *pc);
	} else if (0xf200 & instruction) == 0x5000 {
		let op;
		match (0x0c00 & instruction) >> 10 {
			0x0 => op = "STR",
			0x1 => op = "STRB",
			0x2 => op = "LDR",
			0x3 => op = "LDRB",
			_ => panic!("ERROR!!!")
		}

		print_assembly_line(&format!("{} R{}, R{}, R{}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x01c0) >> 6), *pc);
	} else if (0xf200 & instruction) == 0x5200 {
		let op;
		match (0x0c00 & instruction) >> 10 {
			0x0 => op = "STRH",
			0x1 => op = "LDSB",
			0x2 => op = "LDRH",
			0x3 => op = "LDSH",
			_ => panic!("ERROR!!!")
		}

		print_assembly_line(&format!("{} R{}, R{}, R{}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x01c0) >> 6), *pc);
	} else if (0xe000 & instruction) == 0x6000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "STR",
			0x1 => op = "LDR",
			0x2 => op = "STRB",
			0x3 => op = "LDRB",
			_ => panic!("ERROR!!!")
		}

		print_assembly_line(&format!("{} R{}, R{}, #{:#X}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6), *pc);
	} else if (0xf000 & instruction) == 0x8000 {
		let op = if (0x0800 & instruction) > 0 { "LDRH" } else { "STRH" };
		print_assembly_line(&format!("{} R{}, R{}, #{:#X}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6), *pc);
	} else if (0xf000 & instruction) == 0x9000 {
		let op = if (0x0800 & instruction) > 0 { "LDR" } else { "STR" };
		print_assembly_line(&format!("{} R{}, SP, #{:#X}", op, (instruction & 0x0700) >> 8, instruction & 0x00ff), *pc);
	} else if (0xf000 & instruction) == 0xa000 {
		let op = if (0x0800 & instruction) > 0 { "SP" } else { "PC" };
		print_assembly_line(&format!("ADD R{}, {}, #{:#X}", (instruction & 0x0700) >> 8, op, instruction & 0x00ff), *pc);
	} else if (0xff00 & instruction) == 0xb000 {
		let sign = if (0x0080 & instruction) > 0 { "" } else { "-" };
		print_assembly_line(&format!("ADD SP, #{}{:#X}", sign, instruction & 0x00ef), *pc);
	} else if (0xf600 & instruction) == 0xb400 {
		let op = if (0x0800 & instruction) > 0 { "POP" } else { "PUSH" };
		let r = if (0x0100 & instruction) > 0 {
			if op == "PUSH" { ", LR" } else { ", PC" }
		} else {
			""
		};

		let mut regs = String::from("{ ");
		for i in 0..8 {
			if ((1 << i) & instruction) > 0 {
				let comma = if regs.len() > 2 { ", " } else { "" };
				regs += &*format!("{}R{}", comma, i);
			}
		}
		regs = format!("{}{} }}", regs, r);

		print_assembly_line(&format!("{} {}", op, regs), *pc);
	} else if (0xf000 & instruction) == 0xc000 {
		let op = if (0x0800 & instruction) > 0 { "LDMIA" } else { "STMIA" };

		let mut regs = String::from("{ ");
		for i in 0..8 {
			if ((1 << i) & instruction) > 0 {
				let comma = if regs.len() > 2 { ", " } else { "" };
				regs += &*format!("{}R{}", comma, i);
			}
		}
		regs += " }";

		print_assembly_line(&format!("{} R{}!, {}", op, (instruction & 0x0700) >> 8, regs), *pc);
	} else if (0xff00 & instruction) == 0xdf00 {
		print_assembly_line(&format!("SWI"), *pc);
	} else if (0xf000 & instruction) == 0xd000 {
		let op;
		match (0x0f00 & instruction) >> 8 {
			0x0 => op = "BEQ",
			0x1 => op = "BNE",
			0x2 => op = "BCS",
			0x3 => op = "BCC",
			0x4 => op = "BMI",
			0x5 => op = "BPL",
			0x6 => op = "BVS",
			0x7 => op = "BVC",
			0x8 => op = "BHI",
			0x9 => op = "BLS",
			0xa => op = "BGE",
			0xb => op = "BLT",
			0xc => op = "BGT",
			0xd => op = "BLE",
			0xe => op = "UNDEFINED",
			_ => panic!("ERROR!!!")
		}

		// TODO: Interpret as signed
		print_assembly_line(&format!("{} Offset: {:#X}", op, instruction & 0x00ff), *pc);
	} else if (0xf800 & instruction) == 0xf000 {
		// TODO: Interpret as signed
		let hi = (instruction & 0x07ff) as u32;

		*pc += 2;
		let bytes2: [u8; 2] = [data[*pc], data[*pc + 1]];
		let instruction2 = u16::from_le_bytes(bytes2);
		if (0xf800 & instruction2) != 0xf800 {
			panic!("Instruction after BL is not BL!!!");
		}
		let lo = (instruction & 0x07ff) as u32;
		let offset = (hi << 12) & (lo << 1);

		print_assembly_line(&format!("BL Target: {:#X}", *pc as u32 + 4 + offset), *pc);
	} else {
		print_assembly_line(&format!("Missing instruction!"), *pc);
	}

	*pc += 2;
}

fn decode_arm(thumb_mode: &mut bool, data: &mut Vec<u8>, pc: &mut usize) {
	let bytes: [u8; 4] = [data[*pc], data[*pc + 1], data[*pc + 2], data[*pc + 3]];
	let instruction = u32::from_le_bytes(bytes);
	let cond = instruction >> (32 - 4);
	if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
		*thumb_mode = !*thumb_mode;
		print_assembly_line(&format!("BX {} R{}", cond, instruction & 0x0000_000f), *pc);
		println!("THUMB ------------------------------------------------------------------------------------------------------------------------ THUMB");
	} else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
		if 1 << 24 & instruction > 0 {
			print_assembly_line(&format!("BL {} R{}", cond, instruction & 0x0000_000f), *pc);
		} else {
			print_assembly_line(&format!("B {} R{}", cond, instruction & 0x0000_000f), *pc);
		}
	} else if (0xe000_0010 & instruction) == 0x0600_0010 {
		print_assembly_line(&format!("Undefined instruction!"), *pc);
	} else if (0x0fb0_0ff0 & instruction) == 0x0100_0090 {
		if 1 << 22 & instruction > 0 {
			print_assembly_line(&format!("SWPB R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, instruction & 0x0000_000f, (instruction & 0x000f_0000) >> 16), *pc);
		} else {
			print_assembly_line(&format!("SWP R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, instruction & 0x0000_000f, (instruction & 0x000f_0000) >> 16), *pc);
		}
	} else if (0x0f00_00f0 & instruction) == 0x0000_0090 {
		let s = if (0x0010_0000 & instruction) > 0 { "S" } else { "" };

		let op;
		match (0x01e0_0000 & instruction) >> 21 {
			0x0 => op = "MUL",
			0x1 => op = "MLA",
			0x4 => op = "UMULL",
			0x5 => op = "UMLAL",
			0x6 => op = "SMULL",
			0x7 => op = "SMALL",
			_ => panic!("ERROR!!!")
		}

		// TODO: Revisit params!!!
		print_assembly_line(&format!("{}{} R{}, R{}, R{}", op, s, (instruction & 0x000f_0000) >> 16, instruction & 0x0000_000f, (instruction & 0x0000_0f00) >> 8), *pc);
	} else if (0x0fbf_0fff & instruction) == 0x010f_0000 {
		if (instruction & 0x0010_0000) > 0 {
			print_assembly_line(&format!("MRS R{}, CPSR", (instruction & 0x0000_f000) >> 12, ), *pc);
		} else {
			print_assembly_line(&format!("MRS R{}, SPSR", (instruction & 0x0000_f000) >> 12, ), *pc);
		}
	} else if (0x0db0_f000 & instruction) == 0x0129_f000 {
		let mut fields = String::from("");
		if (0x0008_000 & instruction) > 0 {
			fields += "f";
		}
		if (0x0004_0000 & instruction) > 0 {
			fields += "s";
		}
		if (0x0002_0000 & instruction) > 0 {
			fields += "x";
		}
		if (0x0001_0000 & instruction) > 0 {
			fields += "c";
		}
		if fields.len() > 0 {
			fields = String::from("_") + &*fields;
		}
		let psr = if (instruction & 0x0010_0000) > 0 { "CPSR" } else { "SPSR" };
		if (instruction & 0x0200_0000) > 0 {
			print_assembly_line(&format!("MSR {}{}, #{:#X}", psr, fields, instruction & 0x0000_00ff), *pc);
		} else {
			print_assembly_line(&format!("MSR {}{}, R{}", psr, fields, instruction & 0x0000_00ff), *pc);
		}
	} else if (0x0c00_0000 & instruction) == 0x0400_0000 {
		let b = if (0x0040_0000 & instruction) > 0 { "B" } else { "" };
		let t = if (0x0020_0000 & instruction) > 0 { "T" } else { "" };
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };

		print_assembly_line(&format!("{}{}{} R{}", l, b, t, (instruction & 0x0000_f000) >> 12), *pc);
	} else if (0x0e40_0F90 & instruction) == 0x0000_0090 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
		let op;
		if (0x0000_0020 & instruction) > 0 {
			op = "H"
		} else if (0x0000_0030 & instruction) > 0 {
			op = "SB"
		} else if (0x0000_0040 & instruction) > 0 {
			op = "SH"
		} else {
			panic!("ERROR!!!");
		}

		print_assembly_line(&format!("{}{} R{}", l, op, instruction & 0x0000_000f), *pc);
	} else if (0x0e40_0090 & instruction) == 0x0040_0090 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
		let op;
		if (0x0000_0020 & instruction) > 0 {
			op = "H"
		} else if (0x0000_0030 & instruction) > 0 {
			op = "SB"
		} else if (0x0000_0040 & instruction) > 0 {
			op = "SH"
		} else {
			panic!("ERROR!!!");
		}

		print_assembly_line(&format!("{}{} #{:#X}", l, op, (instruction & 0x0000_0f00) >> 4 | instruction & 0x0000_000f), *pc);
	} else if (0x0e00_0000 & instruction) == 0x0800_0000 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDM" } else { "STM" };
		let w = if (0x0020_0000 & instruction) > 0 { "!" } else { "" };
		let s = if (0x0040_0000 & instruction) > 0 { "^" } else { "" };
		let u = if (0x0080_0000 & instruction) > 0 { "I" } else { "D" };
		let p = if (0x0100_0000 & instruction) > 0 { "B" } else { "A" };

		let mut regs = String::from("{ ");
		for i in 0..16 {
			if ((1 << i) & instruction) > 0 {
				let comma = if regs.len() > 2 { ", " } else { "" };
				regs += &*format!("{}R{}", comma, i);
			}
		}
		regs += " }";

		print_assembly_line(&format!("{}{}{} R{}{}, {}{}", l, u, p, (instruction & 0x000f_0000) >> 16, w, regs, s), *pc);
	} else if (0x0f00_0000 & instruction) == 0x0f00_0000 {
		print_assembly_line(&format!("SWI"), *pc);
	} else if (0x0c00_0000 & instruction) == 0x0000_0000 {
		let i = (0x0200_0000 & instruction) > 0;
		let s = if (0x0010_0000 & instruction) > 0 { "S" } else { "" };
		let mut rn = &*format!("R{},", (instruction & 0x000f_0000) >> 16);
		let mut rd = &*format!("R{},", (instruction & 0x0000_f000) >> 12);

		let op;
		match (0x01e0_0000 & instruction) >> 21 {
			0x0 => op = "AND",
			0x1 => op = "EOR",
			0x2 => op = "SUB",
			0x3 => op = "RSB",
			0x4 => op = "ADD",
			0x5 => op = "ADC",
			0x6 => op = "SBC",
			0x7 => op = "RSC",
			0x8 => {
				op = "TST";
				rd = "";
			},
			0x9 => {
				op = "TEQ";
				rd = "";
			},
			0xa => {
				op = "CMP";
				rd = "";
			},
			0xb => {
				op = "CMN";
				rd = "";
			},
			0xc => op = "ORR",
			0xd => {
				op = "MOV";
				rn = "";
			},
			0xe => op = "BIC",
			0xf => {
				op = "MVN";
				rn = "";
			},
			_ => panic!("ERROR!!!")
		}

		let op2 = if i {
			format!("#{:#X}", 0x0000_00ff & instruction)
		} else {
			format!("R{}", 0x0000_000f & instruction)
		};

		print_assembly_line(&format!("{}{} {}{} {}", op, s, rd, rn, op2), *pc);
	} else {
		print_assembly_line(&format!("Missing instruction!"), *pc);
	}

	*pc += 4;
}

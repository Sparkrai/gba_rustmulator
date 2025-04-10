use num_traits::FromPrimitive;

use crate::arm7tdmi::{sign_extend, EShiftType};
use bitvec::prelude::*;

pub fn print_assembly_line(line: String, pc: u32) {
	println!("{:#06X}| {}", pc, line)
}

pub fn disassemble_cond(cond: u8) -> &'static str {
	match cond {
		0x0 => "EQ",
		0x1 => "NE",
		0x2 => "CS",
		0x3 => "CC",
		0x4 => "MI",
		0x5 => "PL",
		0x6 => "VS",
		0x7 => "VC",
		0x8 => "HI",
		0x9 => "LS",
		0xa => "GE",
		0xb => "LT",
		0xc => "GT",
		0xd => "LE",
		_ => "",
	}
}

pub fn get_register_list(instruction: u32, thumb: bool) -> String {
	let mut regs = String::from("{ ");
	let bits = if thumb { 8 } else { 16 };

	let reg_list = (((1 << bits) - 1) & instruction).view_bits::<Lsb0>().to_bitvec().into_boxed_bitslice();
	for i in 0..bits {
		if reg_list[i] {
			if i > 0 && reg_list[i - 1] {
				if i < bits - 1 && reg_list[i + 1] {
					continue;
				} else {
					regs += &*format!("-R{}", i);
					continue;
				}
			}

			let comma = if regs.len() > 2 { ", " } else { "" };
			regs += &*format!("{}R{}", comma, i);
		}
	}
	regs += " }";

	return regs;
}

pub fn disassemble_thumb(instruction: u16) -> String {
	return if (0xf800 & instruction) == 0x1800 {
		let op = if (0x0200 & instruction) != 0 { "SUB" } else { "ADD" };
		let i = (0x0400 & instruction) != 0;
		let rn = if i {
			format!("#{}", (0x01c0 & instruction) >> 6)
		} else {
			format!("R{}", (0x01c0 & instruction) >> 6)
		};

		format!("{} R{}, R{}, {}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, rn)
	} else if (0xe000 & instruction) == 0x0000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "LSL",
			0x1 => op = "LSR",
			0x2 => op = "ASR",
			_ => panic!("ERROR!!!"),
		}

		format!("{} R{}, R{}, #{}", op, instruction & 0x0003, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6)
	} else if (0xe000 & instruction) == 0x2000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "MOV",
			0x1 => op = "CMP",
			0x2 => op = "ADD",
			0x3 => op = "SUB",
			_ => panic!("ERROR!!!"),
		}

		format!("{} R{}, #{}", op, (instruction & 0x0700) >> 8, instruction & 0x00ff)
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
			_ => panic!("ERROR!!!"),
		}

		format!("{} R{}, R{}", op, instruction & 0x0007, (instruction & 0x0038) >> 3)
	} else if (0xfc00 & instruction) == 0x4400 {
		let op;
		match (0x0300 & instruction) >> 8 {
			0x0 => op = "ADD",
			0x1 => op = "CMP",
			0x2 => op = "MOV",
			0x3 => op = "BX",
			_ => panic!("ERROR!!!"),
		}

		let rm = (instruction & 0x0078) >> 3;
		let rd = if op == "BX" {
			String::from("")
		} else {
			format!("R{}, ", (instruction & 0x0007) | ((instruction & 0x0080) >> 4))
		};

		format!("{} {}R{}", op, rd, rm)
	} else if (0xf800 & instruction) == 0x4800 {
		format!("LDR R{}, [PC, #{}]", (instruction & 0x0700) >> 8, instruction & 0x00ff)
	} else if (0xf200 & instruction) == 0x5000 {
		let op;
		match (0x0c00 & instruction) >> 10 {
			0x0 => op = "STR",
			0x1 => op = "STRB",
			0x2 => op = "LDR",
			0x3 => op = "LDRB",
			_ => panic!("ERROR!!!"),
		}

		format!("{} R{}, [R{}, R{}]", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x01c0) >> 6)
	} else if (0xf200 & instruction) == 0x5200 {
		let op;
		match (0x0c00 & instruction) >> 10 {
			0x0 => op = "STRH",
			0x1 => op = "LDSB",
			0x2 => op = "LDRH",
			0x3 => op = "LDSH",
			_ => panic!("ERROR!!!"),
		}

		format!("{} R{}, [R{}, R{}]", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x01c0) >> 6)
	} else if (0xe000 & instruction) == 0x6000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "STR",
			0x1 => op = "LDR",
			0x2 => op = "STRB",
			0x3 => op = "LDRB",
			_ => panic!("ERROR!!!"),
		}

		format!("{} R{}, [R{}, #{}]", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6)
	} else if (0xf000 & instruction) == 0x8000 {
		let op = if (0x0800 & instruction) > 0 { "LDRH" } else { "STRH" };
		format!("{} R{}, [R{}, #{}]", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6)
	} else if (0xf000 & instruction) == 0x9000 {
		let op = if (0x0800 & instruction) > 0 { "LDR" } else { "STR" };
		format!("{} R{}, SP, #{}", op, (instruction & 0x0700) >> 8, (instruction & 0x00ff) << 2)
	} else if (0xf000 & instruction) == 0xa000 {
		let op = if (0x0800 & instruction) > 0 { "SP" } else { "PC" };
		format!("ADD R{}, {}, #{}", (instruction & 0x0700) >> 8, op, instruction & 0x00ff)
	} else if (0xff00 & instruction) == 0xb000 {
		let op = if (0x0080 & instruction) != 0 { "SUB" } else { "ADD" };
		format!("{} SP, #{}", op, (instruction & 0x007f) << 2)
	} else if (0xf600 & instruction) == 0xb400 {
		let op = if (0x0800 & instruction) > 0 { "POP" } else { "PUSH" };
		let r = if (0x0100 & instruction) > 0 {
			if op == "PUSH" {
				", LR"
			} else {
				", PC"
			}
		} else {
			""
		};

		let regs = get_register_list(instruction as u32, true);

		format!("{} {}{}", op, regs, r)
	} else if (0xf000 & instruction) == 0xc000 {
		let op = if (0x0800 & instruction) > 0 { "LDMIA" } else { "STMIA" };

		let regs = get_register_list(instruction as u32, true);
		format!("{} R{}!, {}", op, (instruction & 0x0700) >> 8, regs)
	} else if (0xff00 & instruction) == 0xdf00 {
		format!("SWI")
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
			_ => panic!("ERROR!!!"),
		}

		let offset = sign_extend(instruction & 0x00ff, 8) << 1;
		format!("{} Offset: {}", op, offset)
	} else if (0xf800 & instruction) == 0xe000 {
		let offset = sign_extend(instruction & 0x07ff, 11) << 1;
		format!("B Offset: #{}", offset)
	} else if (0xf800 & instruction) == 0xf000 {
		let hi = sign_extend(instruction & 0x07ff, 11);
		format!("BL Target: #{} + ", hi << 12)
	} else if (0xf800 & instruction) == 0xf800 {
		let lo = sign_extend(instruction & 0x07ff, 11);
		format!("#{}", lo << 1)
	} else {
		format!("Missing instruction!")
	};
}

pub fn disassemble_arm(instruction: u32) -> String {
	let cond = (instruction >> (32 - 4)) as u8;
	if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
		return format!("BX {} R{}", disassemble_cond(cond), instruction & 0x0000_000f);
	} else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
		if 1 << 24 & instruction > 0 {
			return format!("BL {} #{}", disassemble_cond(cond), instruction & 0x00ff_ffff);
		} else {
			return format!("B {} #{}", disassemble_cond(cond), instruction & 0x00ff_ffff);
		}
	} else if (0xe000_0010 & instruction) == 0x0600_0010 {
		return format!("Undefined instruction!");
	} else if (0x0fb0_0ff0 & instruction) == 0x0100_0090 {
		if 1 << 22 & instruction > 0 {
			return format!(
				"SWPB R{}, R{}, R{}",
				(instruction & 0x0000_f000) >> 12,
				instruction & 0x0000_000f,
				(instruction & 0x000f_0000) >> 16
			);
		} else {
			return format!(
				"SWP R{}, R{}, R{}",
				(instruction & 0x0000_f000) >> 12,
				instruction & 0x0000_000f,
				(instruction & 0x000f_0000) >> 16
			);
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
			0x7 => op = "SMLAL",
			_ => panic!("ERROR!!!"),
		}

		// TODO: Revisit params!!!
		return format!(
			"{}{} {} R{}, R{}, R{}",
			op,
			s,
			disassemble_cond(cond),
			(instruction & 0x000f_0000) >> 16,
			instruction & 0x0000_000f,
			(instruction & 0x0000_0f00) >> 8
		);
	} else if (0x0fbf_0fff & instruction) == 0x010f_0000 {
		if (instruction & 0x0040_0000) > 0 {
			return format!("MRS {} R{}, CPSR", disassemble_cond(cond), (instruction & 0x0000_f000) >> 12);
		} else {
			return format!("MRS {} R{}, SPSR", disassemble_cond(cond), (instruction & 0x0000_f000) >> 12);
		}
	} else if (0x0db0_f000 & instruction) == 0x0120_f000 {
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
		let psr = if (instruction & 0x0040_0000) > 0 { "SPSR" } else { "CPSR" };
		if (instruction & 0x0200_0000) > 0 {
			return format!("MSR {} {}{}, #{}", disassemble_cond(cond), psr, fields, instruction & 0x0000_00ff);
		} else {
			return format!("MSR {} {}{}, R{}", disassemble_cond(cond), psr, fields, instruction & 0x0000_00ff);
		}
	} else if (0x0c00_0000 & instruction) == 0x0400_0000 {
		let p = (0x0100_0000 & instruction) > 0;
		let w = (0x0020_0000 & instruction) > 0;
		let i = (0x0200_0000 & instruction) > 0;
		let u = if (0x0080_0000 & instruction) > 0 { "+" } else { "-" };
		let b = if (0x0040_0000 & instruction) > 0 { "B" } else { "" };
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
		let t = if !p && w { "T" } else { "" };

		let rn = (instruction & 0x000f_0000) >> 16;
		let address;
		if i {
			let rm = instruction & 0x0000_000f;
			let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 5).unwrap();
			let shift = (0x0000_0f80 & instruction) >> 7;

			address = format!("[R{}, R{}, {:?} #{}]", rn, rm, shift_type, shift);
		} else {
			if p {
				let pre = if w { "!" } else { "" };
				address = format!("[R{}, #{}{}]{}", rn, u, instruction & 0x0000_0fff, pre);
			} else {
				address = format!("[R{}], #{}{}", rn, u, instruction & 0x0000_0fff);
			}
		}

		return format!("{}{}{} {} R{}, {}", l, b, t, disassemble_cond(cond), (instruction & 0x0000_f000) >> 12, address);
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

		return format!("{}{} {} R{}", l, op, disassemble_cond(cond), instruction & 0x0000_000f);
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

		return format!("{}{} {} #{}", l, op, disassemble_cond(cond), (instruction & 0x0000_0f00) >> 4 | instruction & 0x0000_000f);
	} else if (0x0e00_0000 & instruction) == 0x0800_0000 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDM" } else { "STM" };
		let w = if (0x0020_0000 & instruction) > 0 { "!" } else { "" };
		let s = if (0x0040_0000 & instruction) > 0 { "^" } else { "" };
		let u = if (0x0080_0000 & instruction) > 0 { "I" } else { "D" };
		let p = if (0x0100_0000 & instruction) > 0 { "B" } else { "A" };

		let regs = get_register_list(instruction, false);

		return format!("{}{}{} {} R{}{}, {}{}", l, u, p, disassemble_cond(cond), (instruction & 0x000f_0000) >> 16, w, regs, s);
	} else if (0x0f00_0000 & instruction) == 0x0f00_0000 {
		return format!("SWI");
	} else if (0x0c00_0000 & instruction) == 0x0000_0000 {
		let i = (0x0200_0000 & instruction) > 0;
		let mut s = if (0x0010_0000 & instruction) > 0 { "S" } else { "" };
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
				s = "";
			}
			0x9 => {
				op = "TEQ";
				rd = "";
				s = "";
			}
			0xa => {
				op = "CMP";
				rd = "";
				s = "";
			}
			0xb => {
				op = "CMN";
				rd = "";
				s = "";
			}
			0xc => op = "ORR",
			0xd => {
				op = "MOV";
				rn = "";
			}
			0xe => op = "BIC",
			0xf => {
				op = "MVN";
				rn = "";
			}
			_ => panic!("ERROR!!!"),
		}

		let shifter_operand;
		if i {
			let rot = (0x0000_0f00 & instruction) >> 8;
			shifter_operand = format!("#{}", (0x0000_00ff & instruction).rotate_right(rot * 2));
		} else {
			let rm = instruction & 0x0000_000f;
			let r = (instruction & 0x0000_0010) > 0;
			let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 5).unwrap();
			if r {
				let rs = (0x0000_0f00 & instruction) >> 8;
				shifter_operand = format!("R{}, {:?}, R{}", rm, shift_type, rs);
			} else {
				let shift = (0x0000_0f80 & instruction) >> 7;
				shifter_operand = format!("R{}, {:?}, #{}", rm, shift_type, shift);
			}
		}

		return format!("{}{} {} {}{} {}", op, s, disassemble_cond(cond), rd, rn, shifter_operand);
	} else {
		return format!("Missing instruction!");
	}
}

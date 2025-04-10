use bitfield::*;
use num_traits::{FromPrimitive, PrimInt};

use crate::arm7tdmi::cpu::{CpuResult, CPU, LINK_REGISTER_REGISTER, PROGRAM_COUNTER_REGISTER, STACK_POINTER_REGISTER};
use crate::arm7tdmi::{cond_passed, load_32_from_memory, sign_extend, EExceptionType, EShiftType};
use crate::system::{MemoryInterface, SystemBus};

bitfield! {
	/// Exposes common information about an encoded THUMB instruction
	pub struct ThumbInstruction(u16);
	impl Debug;
	u8;
	pub get_cond, _: 11, 8;
	// Registers
	pub get_rd_index, _: 2, 0;
	pub get_rn_index, _: 5, 3;
	pub get_rm_index, _: 8, 6;
	/// Index of high Register Rm
	pub get_hi_rm_index, _: 6, 3;
	pub get_rs_index, _: 10, 8;
	// Flags
	pub get_b, _: 12;
	pub get_l, _: 11;
	pub get_i, _: 10;
	pub get_is_sub, _: 9;
	pub get_r, _: 8;
	pub get_is_neg, _: 7;
	// Immediates
	pub u32, get_offset_11, _: 10, 0;
	pub u32, get_imm_8, _: 7, 0;
	pub i8, into i32, get_signed_imm_8, _: 7, 0;
	pub u32, get_imm_7, _: 6, 0;
	pub u32, get_imm_5, _: 10, 6;
	raw_shift_type, _: 12, 11;
	pub u8, get_register_list, _: 7, 0;
}

impl ThumbInstruction {
	/// Index of high Register Rd
	pub fn get_hi_rd_index(&self) -> u8 {
		BitRange::<u8>::bit_range(self, 2, 0) | ((self.bit(7) as u8) << 3)
	}

	pub fn get_shift_type(&self) -> EShiftType {
		FromPrimitive::from_u8(self.raw_shift_type()).unwrap()
	}
}

pub fn execute_thumb(raw_instruction: u16, cpu: &mut CPU, bus: &mut SystemBus) -> CpuResult {
	let instruction = ThumbInstruction(raw_instruction);
	// ADD / SUB register
	if (0xf800 & raw_instruction) == 0x1800 {
		let is_sub = instruction.get_is_sub();
		let i = instruction.get_i();

		let rn = cpu.get_register_value(instruction.get_rn_index());
		let rd_index = instruction.get_rd_index();
		let operand = if i {
			instruction.get_rm_index() as u32
		} else {
			cpu.get_register_value(instruction.get_rm_index())
		};

		if is_sub {
			// Borrowed if carries bits over
			let (alu_out, borrowed) = rn.overflowing_sub(operand as u32);
			// Overflow is sign changes
			let (_, overflow) = (rn as i32).overflowing_sub(operand as i32);

			cpu.set_register_value(rd_index, alu_out);

			cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
			cpu.get_mut_cpsr().set_z(alu_out == 0);
			cpu.get_mut_cpsr().set_c(!borrowed);
			cpu.get_mut_cpsr().set_v(overflow);
		} else {
			// Borrowed if carries bits over
			let (alu_out, borrowed) = rn.overflowing_add(operand as u32);
			// Overflow is sign changes
			let (_, overflow) = (rn as i32).overflowing_add(operand as i32);

			cpu.set_register_value(rd_index, alu_out);

			cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
			cpu.get_mut_cpsr().set_z(alu_out == 0);
			cpu.get_mut_cpsr().set_c(borrowed);
			cpu.get_mut_cpsr().set_v(overflow);
		}
	} else if (0xe000 & raw_instruction) == 0x0000 {
		// Move shifted register (LSL/LSR/ASR)
		let shift_type = instruction.get_shift_type();

		let offset = instruction.get_imm_5();
		let rd_index = instruction.get_rd_index();
		let rm = cpu.get_register_value(instruction.get_rn_index());
		let alu_out;
		let shifter_carry_out;
		match shift_type {
			EShiftType::LSL => {
				if offset == 0 {
					alu_out = rm;
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else {
					alu_out = rm << offset;
					shifter_carry_out = rm.bit(32 - offset as usize);
				}
			}
			EShiftType::LSR => {
				if offset == 0 {
					shifter_carry_out = (rm & 0x8000_0000) != 0;
					alu_out = 0;
				} else {
					shifter_carry_out = rm.bit((offset - 1) as usize);
					alu_out = rm >> offset;
				}
			}
			EShiftType::ASR => {
				if offset == 0 {
					if (rm & 0x8000_0000) == 0 {
						alu_out = 0;
					} else {
						alu_out = 0xffff_ffff;
					}
					shifter_carry_out = (rm & 0x8000_0000) > 0;
				} else {
					alu_out = rm.signed_shr(offset as u32);
					shifter_carry_out = rm.bit((offset - 1) as usize);
				}
			}
			EShiftType::ROR => {
				panic!("ERROR!");
			}
		}

		cpu.set_register_value(rd_index, alu_out);

		cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
		cpu.get_mut_cpsr().set_z(alu_out == 0);
		cpu.get_mut_cpsr().set_c(shifter_carry_out);
	} else if (0xe000 & raw_instruction) == 0x2000 {
		// ALU immediate
		let rd_index = instruction.get_rs_index();
		let rd = cpu.get_register_value(rd_index);
		let operand = instruction.get_imm_8();
		let op: u32 = instruction.bit_range(12, 11);
		match op {
			// MOV
			0x0 => {
				cpu.set_register_value(rd_index, operand);

				cpu.get_mut_cpsr().set_n((operand & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(operand == 0);
			}
			// CMP
			0x1 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(operand);
				// Overflow is sign changes
				let (_, overflow) = (rd as i32).overflowing_sub(operand as i32);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// ADD
			0x2 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_add(operand);
				// Overflow is sign changes
				let (_, overflow) = (rd as i32).overflowing_add(operand as i32);

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// SUB
			0x3 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(operand);
				// Overflow is sign changes
				let (_, overflow) = (rd as i32).overflowing_sub(operand as i32);

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			_ => panic!("ERROR!!!"),
		}
	} else if (0xfc00 & raw_instruction) == 0x4000 {
		// ALU register
		let rm = cpu.get_register_value(instruction.get_rn_index());
		let rd_index = instruction.get_rd_index();
		let rd = cpu.get_register_value(rd_index);
		let op: u32 = instruction.bit_range(9, 6);
		match op {
			// AND
			0x0 => {
				let alu_out = rd & rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// EOR
			0x1 => {
				let alu_out = rd ^ rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// LSL
			0x2 => {
				let rs = rm & 0x000_00ff;
				let shifter_carry_out;
				let alu_out;
				if rs == 0 {
					alu_out = rd;
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else if rs < 32 {
					alu_out = rd << rs;
					shifter_carry_out = rd.bit(32 - rs as usize);
				} else if rs == 32 {
					alu_out = 0;
					shifter_carry_out = (rd & 0x0000_0001) != 0;
				} else {
					alu_out = 0;
					shifter_carry_out = false;
				}
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(shifter_carry_out);
			}
			// LSR
			0x3 => {
				let rs = rm & 0x000_00ff;
				let shifter_carry_out;
				let alu_out;
				if rs == 0 {
					alu_out = rd;
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else if rs < 32 {
					alu_out = rd.unsigned_shr(rs);
					shifter_carry_out = rd.bit((rs - 1) as usize);
				} else if rs == 32 {
					alu_out = 0;
					shifter_carry_out = (rd & 0x8000_0000) != 0;
				} else {
					alu_out = 0;
					shifter_carry_out = false;
				}
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(shifter_carry_out);
			}
			// ASR
			0x4 => {
				let rs = rm & 0x000_00ff;
				let shifter_carry_out;
				let alu_out;
				if rs == 0 {
					alu_out = rd;
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else if rs < 32 {
					alu_out = rd.signed_shr(rs);
					shifter_carry_out = rd.bit((rs - 1) as usize);
				} else {
					shifter_carry_out = (rd & 0x0000_0001) != 0;
					if !shifter_carry_out {
						alu_out = 0;
					} else {
						alu_out = 0xffff_ffff;
					}
				}
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(shifter_carry_out);
			}
			// ADC
			0x5 => {
				// Borrowed if carries bits over
				let (alu_out_first, borrowed_first) = rd.overflowing_add(rm);
				let c = cpu.get_cpsr().get_c() as u32;
				let (alu_out, borrowed_second) = alu_out_first.overflowing_add(c);
				let borrowed = borrowed_first || borrowed_second;

				// Overflow if sign changes
				let (_, overflow_first) = (rd as i32).overflowing_add(rm as i32);
				let (_, overflow_second) = (alu_out_first as i32).overflowing_add(c as i32);
				let overflow = overflow_first || overflow_second;

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// SBC
			0x6 => {
				// Borrowed if carries bits over
				let (alu_out_first, borrowed_first) = rd.overflowing_sub(rm);
				let c = !cpu.get_cpsr().get_c() as u32;
				let (alu_out, borrowed_second) = alu_out_first.overflowing_sub(c);
				let borrowed = borrowed_first || borrowed_second;

				// Overflow if sign changes
				let (_, overflow_first) = (rd as i32).overflowing_sub(rm as i32);
				let (_, overflow_second) = (alu_out_first as i32).overflowing_sub(c as i32);
				let overflow = overflow_first || overflow_second;

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// ROR
			0x7 => {
				let rs = rm & 0x000_00ff;
				let rs_shift = rs & 0x1f;
				let shifter_carry_out;
				let alu_out;
				if rs == 0 {
					alu_out = rd;
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else if rs_shift == 0 {
					alu_out = rd;
					shifter_carry_out = (rd & 0x8000_0000) != 0;
				} else {
					alu_out = rd.rotate_right(rs_shift);
					shifter_carry_out = rd.bit((rs_shift - 1) as usize);
				}

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(shifter_carry_out);
			}
			// TST
			0x8 => {
				let alu_out = rd & rm;
				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// NEG
			0x9 => {
				// Overflow is sign changes
				let (alu_out, overflow) = 0i32.overflowing_sub(rm as i32);

				cpu.set_register_value(rd_index, alu_out as u32);

				cpu.get_mut_cpsr().set_n((alu_out as u32 & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(true); // No carry can occur from 0
				cpu.get_mut_cpsr().set_v(overflow); // No overflow can occur from 0
			}
			// CMP
			0xa => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(rm);
				// Overflow is sign changes
				let (_, overflow) = (rd as i32).overflowing_sub(rm as i32);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// CMN
			0xb => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_add(rm);
				// Overflow is sign changes
				let (_, overflow) = (rd as i32).overflowing_add(rm as i32);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// ORR
			0xc => {
				let alu_out = rd | rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// MUL
			0xd => {
				let alu_out = rm.wrapping_mul(rd);
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(false);
			}
			// BIC
			0xe => {
				let alu_out = rd & !rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// MVN
			0xf => {
				let alu_out = !rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			_ => panic!("ERROR!!!"),
		}
	} else if (0xff80 & raw_instruction) == 0x4700 {
		// Branch exchange (BX)
		let rm = cpu.get_register_value(instruction.get_hi_rm_index());

		let t = (0x1 & rm) != 0;
		cpu.get_mut_cpsr().set_t(t);

		// NOTE: Enforce alignment
		let address = if t { rm & !0x1 } else { rm & !0x3 };
		cpu.set_register_value(PROGRAM_COUNTER_REGISTER, address);
		return CpuResult::FlushPipeline;
	} else if (0xfc00 & raw_instruction) == 0x4400 {
		// Hi register ALUs
		let rm = cpu.get_register_value(instruction.get_hi_rm_index());
		let rd_index = instruction.get_hi_rd_index();
		let rd = cpu.get_register_value(rd_index);
		match BitRange::<u8>::bit_range(&instruction, 9, 8) {
			// ADD
			0x0 => cpu.set_register_value(rd_index, rd.wrapping_add(rm)),
			// CMP
			0x1 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(rm);
				// Overflow is sign changes
				let (_, overflow) = (rd as i32).overflowing_sub(rm as i32);

				cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);

				return CpuResult::Continue;
			}
			// MOV
			0x2 => cpu.set_register_value(rd_index, rm),
			_ => panic!("ERROR!!!"),
		}

		// NOTE: PC Changed!!!
		if rd_index == PROGRAM_COUNTER_REGISTER {
			return CpuResult::FlushPipeline;
		}
	} else if (0xf800 & raw_instruction) == 0x4800 {
		// LDR PC relative
		let rd_index = instruction.get_rs_index();
		let operand = instruction.get_imm_8();

		let address = (cpu.get_register_value(PROGRAM_COUNTER_REGISTER) & 0xffff_fffc) + (operand * 4) as u32;
		cpu.set_register_value(rd_index, bus.read_32(address));
	} else if (0xf200 & raw_instruction) == 0x5000 {
		// LDR/STR with register offset
		let l = instruction.get_l();

		// NOTE: Flag is in bits 10
		let b = instruction.get_i();

		let rm = cpu.get_register_value(instruction.get_rm_index());
		let rn = cpu.get_register_value(instruction.get_rn_index());
		let rd_index = instruction.get_rd_index();

		let address = rn.wrapping_add(rm);
		if l {
			let data;
			if b {
				data = bus.read_8(address) as u32;
			} else {
				data = load_32_from_memory(bus, address);
			}
			cpu.set_register_value(rd_index, data);
		} else {
			let rd = cpu.get_register_value(rd_index);
			if b {
				bus.write_8(address, rd as u8);
			} else {
				// NOTE: Forced alignment! (UNPREDICTABLE)
				bus.write_32(address & !0x0000_0003, rd);
			}
		}
	} else if (0xf200 & raw_instruction) == 0x5200 {
		// LDR/STR sign-extended byte/halfword
		let rm = cpu.get_register_value(instruction.get_rm_index());
		let rn = cpu.get_register_value(instruction.get_rn_index());
		let rd_index = instruction.get_rd_index();

		let address = rn.wrapping_add(rm);

		let l = instruction.get_l();

		// NOTE: Flag is in bits 10
		let s = instruction.get_i();

		// STRH
		if !l && !s {
			let rd = cpu.get_register_value(rd_index);
			// NOTE: Forced alignment! (UNPREDICTABLE)
			bus.write_16(address & !0x1, rd as u16);
		} else {
			let data;
			// LDSH
			if s && l {
				if (address & 0x0000_0001) == 0 {
					data = bus.read_16(address) as i16 as u32;
				} else {
					// NOTE: Read byte! (UNPREDICTABLE)
					data = bus.read_8(address) as i8 as u32;
				}
			}
			// LDSB
			else if s {
				data = bus.read_8(address) as i8 as u32;
			}
			// LDRH
			else if l {
				if (address & 0x0000_0001) == 0 {
					data = bus.read_16(address) as u32;
				} else {
					// NOTE: Forced alignment and rotation of data! (UNPREDICTABLE)
					data = (bus.read_16(address & !0x1) as u32).rotate_right(8);
				}
			} else {
				std::unreachable!();
			}

			cpu.set_register_value(rd_index, data);
		}
	} else if (0xe000 & raw_instruction) == 0x6000 {
		// LDR/STR with immediate offset
		let l = instruction.get_l();
		let b = instruction.get_b();

		let offset = instruction.get_imm_5();
		let rn = cpu.get_register_value(instruction.get_rn_index());
		let rd_index = instruction.get_rd_index();

		let address = if b { rn.wrapping_add(offset) } else { rn.wrapping_add(offset * 4) };

		if l {
			let data;
			if b {
				data = bus.read_8(address) as u32;
			} else {
				data = load_32_from_memory(bus, address);
			}

			cpu.set_register_value(rd_index, data);
		} else {
			let rd = cpu.get_register_value(rd_index);
			if b {
				bus.write_8(address, rd as u8);
			} else {
				// NOTE: Forced alignment! (UNPREDICTABLE)
				bus.write_32(address & !0x0000_0003, rd);
			}
		}
	} else if (0xf000 & raw_instruction) == 0x8000 {
		// LDR/STR halfword with immediate offset
		let l = instruction.get_l();

		let offset = instruction.get_imm_5();
		let rn = cpu.get_register_value(instruction.get_rn_index());
		let rd_index = instruction.get_rd_index();

		let address = rn.wrapping_add(offset * 2);
		if l {
			let data;
			if (address & 0x0000_0001) == 0 {
				data = bus.read_16(address) as u32;
			} else {
				// NOTE: Forced alignment and rotation of data! (UNPREDICTABLE)
				data = (bus.read_16(address & !0x0000_0001) as u32).rotate_right(8);
			}

			cpu.set_register_value(rd_index, data);
		} else {
			let rd = cpu.get_register_value(rd_index);
			// NOTE: Forced alignment! (UNPREDICTABLE)
			bus.write_16(address & !0x0000_0001, rd as u16);
		}
	} else if (0xf000 & raw_instruction) == 0x9000 {
		// LDR/STR SP relative
		let l = instruction.get_l();

		let offset = instruction.get_imm_8();
		let rd_index = instruction.get_rs_index();

		let address = cpu.get_register_value(STACK_POINTER_REGISTER).wrapping_add(offset * 4);
		if l {
			let data = load_32_from_memory(bus, address);

			cpu.set_register_value(rd_index, data);
		} else {
			let rd = cpu.get_register_value(rd_index);
			// NOTE: Forced alignment! (UNPREDICTABLE)
			bus.write_32(address & !0x0000_0003, rd);
		}
	} else if (0xf000 & raw_instruction) == 0xa000 {
		// ADD Get relative offset
		let sp = instruction.get_l();
		let rd_index = instruction.get_rs_index();
		let operand = instruction.get_imm_8();

		let value;
		if sp {
			value = cpu.get_register_value(STACK_POINTER_REGISTER) + (operand * 4);
		} else {
			value = (cpu.get_register_value(PROGRAM_COUNTER_REGISTER) & !0x3) + (operand * 4);
		}

		cpu.set_register_value(rd_index, value);
	} else if (0xff00 & raw_instruction) == 0xb000 {
		// ADD offset to Stack Pointer
		let is_sub = instruction.get_is_neg();
		let operand = instruction.get_imm_7();
		let sp = cpu.get_register_value(STACK_POINTER_REGISTER);

		if is_sub {
			cpu.set_register_value(STACK_POINTER_REGISTER, sp.wrapping_sub(operand << 2));
		} else {
			cpu.set_register_value(STACK_POINTER_REGISTER, sp.wrapping_add(operand << 2));
		}
	} else if (0xf600 & raw_instruction) == 0xb400 {
		// PUSH/POP
		let pop = instruction.get_l();
		let r = instruction.get_r();
		let sp = cpu.get_register_value(STACK_POINTER_REGISTER);
		let reg_list = instruction.get_register_list();

		if pop {
			// NOTE: Forced alignment!
			let start_address = sp;
			let end_address = sp.wrapping_add(4 * (r as u32 + reg_list.count_ones() as u32));
			let mut address = start_address;

			for i in 0..8 {
				if reg_list.bit(i) {
					cpu.set_register_value(i as u8, bus.read_32(address & !0x3));
					address = address.wrapping_add(4);
				}
			}

			if r {
				let value = bus.read_32(address & !0x3) & !0x1;
				cpu.set_register_value(PROGRAM_COUNTER_REGISTER, value);
				address = address.wrapping_add(4);
			}
			debug_assert_eq!(end_address, address);

			cpu.set_register_value(STACK_POINTER_REGISTER, end_address);
		} else {
			// NOTE: Forced alignment!
			let start_address = sp.wrapping_sub(4 * (r as u32 + reg_list.count_ones() as u32));
			let end_address = sp.wrapping_sub(4);
			let mut address = start_address;
			for i in 0..8 {
				if reg_list.bit(i) {
					bus.write_32(address & !0x3, cpu.get_register_value(i as u8));
					address = address.wrapping_add(4);
				}
			}

			if r {
				bus.write_32(address & !0x3, cpu.get_register_value(LINK_REGISTER_REGISTER));
				address = address.wrapping_add(4);
			}
			debug_assert_eq!(end_address, address.wrapping_sub(4));

			cpu.set_register_value(STACK_POINTER_REGISTER, start_address);
		}

		// NOTE: PC Changed!!!
		if pop && r {
			return CpuResult::FlushPipeline;
		}
	} else if (0xf000 & raw_instruction) == 0xc000 {
		// LDMIA/STMIA
		let l = instruction.get_l();
		let rn_index = instruction.get_rs_index();
		let rn = cpu.get_register_value(rn_index);
		let reg_list = instruction.get_register_list();

		// NOTE: UNPREDICTABLE!!!
		if reg_list == 0 {
			// Addressing Mode
			let address = rn & !0x3;
			cpu.set_register_value(rn_index, rn.wrapping_add(0x40));

			if l {
				let value = load_32_from_memory(bus, address);
				cpu.set_register_value(PROGRAM_COUNTER_REGISTER, value);

				return CpuResult::FlushPipeline;
			} else {
				let value = cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 2;
				bus.write_32(address, value);
			}
		} else {
			// Addressing Mode
			let start_address = rn;
			let end_address = rn.wrapping_add(4 * (reg_list.count_ones() as u32)) - 4;
			let mut address = start_address;

			let store_rn = reg_list.bit(rn_index as usize);
			if !(l && store_rn) {
				cpu.set_register_value(rn_index, rn.wrapping_add(4 * (reg_list.count_ones() as u32)));
			}

			if l {
				for i in 0..8 {
					if reg_list.bit(i) {
						cpu.set_register_value(i as u8, bus.read_32(address));
						address = address.wrapping_add(4);
					}
				}
				debug_assert_eq!(end_address, address.wrapping_sub(4));
			} else {
				let mut first = true;
				for i in 0..8 {
					if reg_list.bit(i) {
						// NOTE: UNPREDICTABLE BEHAVIOR
						let value = if first && i == rn_index as usize { rn } else { cpu.get_register_value(i as u8) };

						bus.write_32(address, value);
						address = address.wrapping_add(4);

						first = false;
					}
				}

				debug_assert_eq!(end_address, address.wrapping_sub(4));
			}
		}
	} else if (0xff00 & raw_instruction) == 0xdf00 {
		// SWI Software Interrupt Exception
		cpu.exception(EExceptionType::SoftwareInterrupt);
		return CpuResult::FlushPipeline;
	} else if (0xf000 & raw_instruction) == 0xd000 {
		// Conditional Branch
		let cond = instruction.get_cond();
		if cond_passed(cpu, cond) {
			let offset = instruction.get_signed_imm_8() << 1;

			cpu.set_register_value(
				PROGRAM_COUNTER_REGISTER,
				(cpu.get_register_value(PROGRAM_COUNTER_REGISTER) as i32).wrapping_add(offset) as u32,
			);
			return CpuResult::FlushPipeline;
		}
	} else if (0xf800 & raw_instruction) == 0xe000 {
		// Unconditional Branch
		let offset = sign_extend(instruction.get_offset_11(), 11) << 1;
		cpu.set_register_value(
			PROGRAM_COUNTER_REGISTER,
			(cpu.get_register_value(PROGRAM_COUNTER_REGISTER) as i32).wrapping_add(offset) as u32,
		);
		return CpuResult::FlushPipeline;
	} else if (0xf000 & raw_instruction) == 0xf000 {
		// BL
		let h = instruction.get_l();
		let pc = cpu.get_register_value(PROGRAM_COUNTER_REGISTER) as i32;

		if !h {
			let offset = sign_extend(instruction.get_offset_11(), 11);
			cpu.set_register_value(LINK_REGISTER_REGISTER, pc.wrapping_add(offset << 12) as u32);
		} else {
			let offset = instruction.get_offset_11();
			let lr = cpu.get_register_value(LINK_REGISTER_REGISTER);
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, lr.wrapping_add(offset << 1) as u32);
			// NOTE: Address of next instruction
			cpu.set_register_value(LINK_REGISTER_REGISTER, ((pc - 2) | 0x1) as u32);
			return CpuResult::FlushPipeline;
		}
	}

	CpuResult::Continue
}

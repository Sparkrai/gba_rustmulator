use bitvec::order::Lsb0;
use bitvec::prelude::BitView;
use num_traits::PrimInt;

use crate::arm7tdmi::cpu::{CPU, PROGRAM_COUNTER_REGISTER};
use crate::arm7tdmi::EShiftType;
use crate::memory::MemoryBus;

pub fn operate_thumb(instruction: u16, cpu: &mut CPU, bus: &mut MemoryBus) {
	if (0xf800 & instruction) == 0x1800 {
		let is_sub = (0x0200 & instruction) != 0;
		let i = (0x0400 & instruction) != 0;

		let rn = cpu.get_register_value(((0x0038 & instruction) >> 3) as u8);
		let rd_index = (0x0007 & instruction) as u8;
		let operand = if i {
			((0x01c0 & instruction) >> 6) as u32
		} else {
			cpu.get_register_value(((0x01c0 & instruction) >> 6) as u8)
		};

		if is_sub {
			// Borrowed if carries bits over
			let (alu_out, borrowed) = rn.overflowing_sub(operand as u32);
			// Overflow is sign changes
			let overflow = (rn as i32).is_positive() != (operand as i32).is_positive() && (rn as i32).is_positive() != (alu_out as i32).is_positive();

			cpu.set_register_value(rd_index, alu_out);

			cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
			cpu.get_mut_cpsr().set_z(alu_out == 0);
			cpu.get_mut_cpsr().set_c(!borrowed);
			cpu.get_mut_cpsr().set_v(overflow);
		} else {
			// Borrowed if carries bits over
			let (alu_out, borrowed) = rn.overflowing_add(operand as u32);
			// Overflow is sign changes
			let overflow = (rn as i32).is_positive() == (operand as i32).is_positive() && (rn as i32).is_positive() != (alu_out as i32).is_positive();

			cpu.set_register_value(rd_index, alu_out);

			cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
			cpu.get_mut_cpsr().set_z(alu_out == 0);
			cpu.get_mut_cpsr().set_c(borrowed);
			cpu.get_mut_cpsr().set_v(overflow);
		}
	} else if (0xe000 & instruction) == 0x0000 {
		let shift_type = match (0x1800 & instruction) >> 11 {
			0x0 => EShiftType::LSL,
			0x1 => EShiftType::LSR,
			0x2 => EShiftType::ASR,
			_ => panic!("ERROR!!!")
		};

		let offset = (0x07c0 & instruction) >> 6;
		let rd_index = (0x0007 & instruction) as u8;
		let rm = cpu.get_register_value(((0x01c0 & instruction) >> 6) as u8);
		let alu_out;
		let shifter_carry_out;
		match shift_type {
			EShiftType::LSL => {
				if offset == 0 {
					alu_out = rm;
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else {
					alu_out = rm << offset;
					shifter_carry_out = rm.view_bits::<Lsb0>()[32 - offset as usize];
				}
			}
			EShiftType::LSR => {
				if offset == 0 {
					shifter_carry_out = (rm & 0x8000_0000) != 0;
					alu_out = 0;
				} else {
					shifter_carry_out = rm.view_bits::<Lsb0>()[(offset - 1) as usize];
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
					shifter_carry_out = rm.view_bits::<Lsb0>()[(offset - 1) as usize];
				}
			}
			EShiftType::ROR => {
				panic!("ERROR!");
			}
		}

		cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
		cpu.get_mut_cpsr().set_z(alu_out == 0);
		cpu.get_mut_cpsr().set_c(shifter_carry_out);
	} else if (0xe000 & instruction) == 0x2000 {
		let rd_index = ((0x0700 & instruction) >> 8) as u8;
		let rd = cpu.get_register_value(rd_index);
		let operand = (0x00ff & instruction) as u32;
		let op = (0x1800 & instruction) >> 11;
		match op {
			// MOV
			0x0 => {
				cpu.set_register_value(rd_index, operand);

				cpu.get_mut_cpsr().set_n((operand & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(operand == 0);
			}
			// CMP
			0x1 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(operand);
				// Overflow is sign changes
				let overflow = (rd as i32).is_positive() != (operand as i32).is_positive() && (rd as i32).is_positive() != (alu_out as i32).is_positive();

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// ADD
			0x2 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_add(operand);
				// Overflow is sign changes
				let overflow = (rd as i32).is_positive() == (operand as i32).is_positive() && (rd as i32).is_positive() != (alu_out as i32).is_positive();

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// SUB
			0x3 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(operand);
				// Overflow is sign changes
				let overflow = (rd as i32).is_positive() != (operand as i32).is_positive() && (rd as i32).is_positive() != (alu_out as i32).is_positive();

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			_ => panic!("ERROR!!!")
		}
	} else if (0xfc00 & instruction) == 0x4000 {
		let rm = cpu.get_register_value(((0x0038 & instruction) >> 3) as u8);
		let rd_index = (0x0007 & instruction) as u8;
		let rd = cpu.get_register_value(rd_index);
		let op = (0x03c0 & instruction) >> 6;
		match op {
			// AND
			0x0 => {
				let alu_out = rd & rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// EOR
			0x1 => {
				let alu_out = rd ^ rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
					shifter_carry_out = rd.view_bits::<Lsb0>()[32 - rs as usize];
				} else if rs == 32 {
					alu_out = 0;
					shifter_carry_out = (rd & 0x0000_0001) != 0;
				} else {
					alu_out = 0;
					shifter_carry_out = false;
				}
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
					alu_out = rd >> rs;
					shifter_carry_out = rd.view_bits::<Lsb0>()[(rs - 1) as usize];
				} else if rs == 32 {
					alu_out = 0;
					shifter_carry_out = (rd & 0x0000_0001) != 0;
				} else {
					alu_out = 0;
					shifter_carry_out = false;
				}
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
					shifter_carry_out = rd.view_bits::<Lsb0>()[(rs - 1) as usize];
				} else {
					shifter_carry_out = (rd & 0x0000_0001) != 0;
					if !shifter_carry_out {
						alu_out = 0;
					} else {
						alu_out = 0xffff_ffff;
					}
				}
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
				let overflow = ((rd as i32).is_positive() == (rm as i32).is_positive() && (rd as i32).is_positive() != (alu_out_first as i32).is_positive())
					|| ((alu_out_first as i32).is_positive() == (c as i32).is_positive() && (alu_out_first as i32).is_positive() != (alu_out as i32).is_positive());

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
				let overflow = ((rd as i32).is_positive() != (rm as i32).is_positive() && (rd as i32).is_positive() != (alu_out_first as i32).is_positive())
					|| ((alu_out_first as i32).is_positive() != (c as i32).is_positive() && (alu_out_first as i32).is_positive() != (alu_out as i32).is_positive());

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
					shifter_carry_out = rd.view_bits::<Lsb0>()[(rs_shift - 1) as usize];
				}

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(shifter_carry_out);
			}
			// TST
			0x8 => {
				let alu_out = rd & rm;
				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// NEG
			0x9 => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = 0u32.overflowing_sub(rm);
				// Overflow is sign changes
				let overflow = (0 as i32).is_positive() != (rm as i32).is_positive() && (0 as i32).is_positive() != (alu_out as i32).is_positive();

				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// CMP
			0xa => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_sub(rm);
				// Overflow is sign changes
				let overflow = (rd as i32).is_positive() != (rm as i32).is_positive() && (rd as i32).is_positive() != (alu_out as i32).is_positive();

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(!borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// CMN
			0xb => {
				// Borrowed if carries bits over
				let (alu_out, borrowed) = rd.overflowing_add(rm);
				// Overflow is sign changes
				let overflow = (rd as i32).is_positive() == (rm as i32).is_positive() && (rd as i32).is_positive() != (alu_out as i32).is_positive();

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(borrowed);
				cpu.get_mut_cpsr().set_v(overflow);
			}
			// ORR
			0xc => {
				let alu_out = rd | rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// MUL
			0xd => {
				let alu_out = rm * rd;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
				cpu.get_mut_cpsr().set_c(false);
			}
			// BIC
			0xe => {
				let alu_out = rd & !rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			// MVN
			0xf => {
				let alu_out = !rm;
				cpu.set_register_value(rd_index, alu_out);

				cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
				cpu.get_mut_cpsr().set_z(alu_out == 0);
			}
			_ => panic!("ERROR!!!")
		}
	}

	cpu.set_register_value(PROGRAM_COUNTER_REGISTER, cpu.get_register_value(PROGRAM_COUNTER_REGISTER) - 2);
}

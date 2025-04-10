use std::u32;

use bitvec::prelude::*;
use num_traits::{FromPrimitive, PrimInt};

use crate::arm7tdmi::cpu::{CPU, LINK_REGISTER_REGISTER, PROGRAM_COUNTER_REGISTER};
use crate::arm7tdmi::{cond_passed, load_32_from_memory, sign_extend, EExceptionType, EOperatingMode, EShiftType};
use crate::system::{MemoryInterface, SystemBus};

pub fn operate_arm(cpu: &mut CPU, bus: &mut SystemBus, instruction: u32) {
	let cond = (instruction >> (32 - 4)) as u8;
	if cond_passed(cpu, cond) {
		if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
			// BX
			let rm = cpu.get_register_value((instruction & 0x0000_000f) as u8);
			cpu.get_mut_cpsr().set_t((rm & 0x0000_0001) != 0);
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, rm & 0xffff_fffe);
			return;
		} else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
			// Branch
			if 0x0100_0000 & instruction > 0 {
				// Branch with Link
				cpu.set_register_value(LINK_REGISTER_REGISTER, cpu.get_current_pc() + 4);
			}

			let offset = sign_extend(0x00ff_ffff & instruction, 24);
			cpu.set_register_value(
				PROGRAM_COUNTER_REGISTER,
				(cpu.get_register_value(PROGRAM_COUNTER_REGISTER) as i32).wrapping_add(offset << 2) as u32,
			);
			return;
		} else if (0xe000_0010 & instruction) == 0x0600_0010 {
			// Undefined instruction
			cpu.exception(EExceptionType::Undefined);
			return;
		} else if (0x0fb0_0ff0 & instruction) == 0x0100_0090 {
			// SWP/SWPB
			let b = (1 << 22 & instruction) != 0;

			let rn = cpu.get_register_value(((instruction & 0x000f_0000) >> 16) as u8);
			let rm = cpu.get_register_value((instruction & 0x0000_000f) as u8);
			let rd_index = ((instruction & 0x0000_f000) >> 12) as u8;

			if b {
				let temp = bus.read_8(rn);
				bus.write_8(rn, rm as u8);
				cpu.set_register_value(rd_index, temp as u32);
			} else {
				let temp;
				if (rn & 0x0000_0003) == 0 {
					temp = bus.read_32(rn);
				} else {
					// NOTE: Forced alignment and rotation of data! (UNPREDICTABLE)
					temp = bus.read_32(rn & !0x0000_0003).rotate_right((rn & 0x0000_0003) * 8);
				}

				// NOTE: Forced alignment! (UNPREDICTABLE)
				bus.write_32(rn & !0x0000_0003, rm);
				cpu.set_register_value(rd_index, temp);
			}
		} else if (0x0f00_00f0 & instruction) == 0x0000_0090 {
			// MUL/MLA Multiply
			let s = (0x0010_0000 & instruction) != 0;
			let rn_index = ((instruction & 0x0000_f000) >> 12) as u8;
			let rn = cpu.get_register_value(rn_index);
			let rs = cpu.get_register_value(((instruction & 0x0000_0f00) >> 8) as u8);
			let rm = cpu.get_register_value((instruction & 0x0000_000f) as u8);
			let rd_index = ((instruction & 0x000f_0000) >> 16) as u8;

			match (0x01e0_0000 & instruction) >> 21 {
				// MUL
				0x0 => {
					let alu_out = rm.wrapping_mul(rs);
					cpu.set_register_value(rd_index, alu_out);

					if s {
						cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
						cpu.get_mut_cpsr().set_z(alu_out == 0);
						cpu.get_mut_cpsr().set_c(false);
					}
				}
				// MLA
				0x1 => {
					let alu_out = rm.wrapping_mul(rs).wrapping_add(rn);
					cpu.set_register_value(rd_index, alu_out);

					if s {
						cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
						cpu.get_mut_cpsr().set_z(alu_out == 0);
						cpu.get_mut_cpsr().set_c(false);
					}
				}
				// UMULL
				0x4 => {
					let alu_out = (rm as u64).wrapping_mul(rs as u64);
					cpu.set_register_value(rd_index, (alu_out >> 32) as u32);
					cpu.set_register_value(rn_index, alu_out as u32);

					if s {
						cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000_0000_0000) != 0);
						cpu.get_mut_cpsr().set_z(alu_out == 0);
						cpu.get_mut_cpsr().set_c(false);
						cpu.get_mut_cpsr().set_v(false);
					}
				}
				// UMLAL
				0x5 => {
					let alu_out = (rm as u64).wrapping_mul(rs as u64);
					let (lo, carry) = (alu_out as u32).overflowing_add(rn);
					cpu.set_register_value(rn_index, lo);

					let rd = cpu.get_register_value(rd_index);
					let hi = (alu_out >> 32) as u32 + rd + carry as u32;
					cpu.set_register_value(rd_index, hi);

					if s {
						cpu.get_mut_cpsr().set_n((hi & 0x8000_0000) != 0);
						cpu.get_mut_cpsr().set_z(hi == 0 && lo == 0);
						cpu.get_mut_cpsr().set_c(false);
						cpu.get_mut_cpsr().set_v(false);
					}
				}
				// SMULL
				0x6 => {
					let alu_out = (rm as i32 as i64).wrapping_mul(rs as i32 as i64);
					cpu.set_register_value(rd_index, (alu_out >> 32) as u32);
					cpu.set_register_value(rn_index, alu_out as u32);

					if s {
						cpu.get_mut_cpsr().set_n(alu_out.is_negative());
						cpu.get_mut_cpsr().set_z(alu_out == 0);
						cpu.get_mut_cpsr().set_c(false);
						cpu.get_mut_cpsr().set_v(false);
					}
				}
				// SMLAL
				0x7 => {
					let alu_out = (rm as i32 as i64).wrapping_mul(rs as i32 as i64);
					let (lo, carry) = (alu_out as u32).overflowing_add(rn);
					cpu.set_register_value(rn_index, lo);

					let rd = cpu.get_register_value(rd_index);
					let hi = ((alu_out >> 32) as u32).wrapping_add(rd).wrapping_add(carry as u32);
					cpu.set_register_value(rd_index, hi);

					if s {
						cpu.get_mut_cpsr().set_n((hi & 0x8000_0000) != 0);
						cpu.get_mut_cpsr().set_z(hi == 0 && lo == 0);
						cpu.get_mut_cpsr().set_c(false);
						cpu.get_mut_cpsr().set_v(false);
					}
				}
				_ => panic!("ERROR!!!"),
			}
		} else if (0x0fbf_0fff & instruction) == 0x010f_0000 {
			// MRS (PSR Transfer)
			let r = (0x0040_0000 & instruction) > 0;
			let rd_index = ((instruction & 0x0000_f000) >> 12) as u8;

			// SPSR vs CPSR
			if r {
				cpu.set_register_value(rd_index, cpu.get_spsr(cpu.get_operating_mode()).get_value());
			} else {
				cpu.set_register_value(rd_index, cpu.get_cpsr().get_value());
			}
		} else if (0x0db0_f000 & instruction) == 0x0120_f000 {
			// MSR (PSR Transfer)
			let i = (0x0200_0000 & instruction) > 0;
			let f_mask = if (0x0008_0000 & instruction) > 0 { 0xff00_0000u32 } else { 0x0000_0000 };
			let s_mask = if (0x0004_0000 & instruction) > 0 { 0x00ff_0000u32 } else { 0x0000_0000 };
			let x_mask = if (0x0002_0000 & instruction) > 0 { 0x0000_ff00u32 } else { 0x0000_0000 };
			let c_mask = if (0x0001_0000 & instruction) > 0 { 0x0000_00ffu32 } else { 0x0000_0000 };

			let r = (0x0040_0000 & instruction) > 0;

			let operand;
			if i {
				let rot = (0x0000_0f00 & instruction) >> 8;
				operand = (0x0000_00ff & instruction).rotate_right(rot * 2);
			} else {
				operand = cpu.get_register_value((instruction & 0x0000_000f) as u8);
			}

			let byte_mask = f_mask | s_mask | x_mask | c_mask;

			const STATE_MASK: u32 = 0x0000_0020;
			const USER_MASK: u32 = 0xf000_0000;
			const PRIV_MASK: u32 = 0x0000_00df;

			let mask;
			let psr;
			if !r {
				if cpu.get_operating_mode() != EOperatingMode::UserMode {
					if (operand & STATE_MASK) != 0 {
						// NOTE: UNPREDICTABLE!
					}
					mask = byte_mask & (USER_MASK | PRIV_MASK);
				} else {
					mask = byte_mask & USER_MASK;
				}

				psr = cpu.get_mut_cpsr();
			} else {
				mask = byte_mask & (USER_MASK | PRIV_MASK | STATE_MASK);
				if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
					psr = cpu.get_mut_spsr(cpu.get_operating_mode());
				} else {
					// NOTE: UNPREDICTABLE!
					psr = cpu.get_mut_cpsr();
				}
			}

			psr.set_value((psr.get_value() & !mask) | (operand & mask));
		} else if (0x0c00_0000 & instruction) == 0x0400_0000 {
			// LDR/STR Single Data Transfer
			let i = (0x0200_0000 & instruction) > 0;
			let p = (0x0100_0000 & instruction) > 0;
			let u = (0x0080_0000 & instruction) > 0;
			let b = (0x0040_0000 & instruction) > 0;
			let w = (0x0020_0000 & instruction) > 0;
			let l = (0x0010_0000 & instruction) > 0;

			let rn_index = ((instruction & 0x000f_0000) >> 16) as u8;
			let rn = cpu.get_register_value(rn_index);
			let rd_index = ((instruction & 0x0000_f000) >> 12) as u8;
			let offset;
			if i {
				let rm = cpu.get_register_value((instruction & 0x0000_000f) as u8);
				let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 5).unwrap();
				let shift = (0x0000_0f80 & instruction) >> 7;

				if shift > 0 || shift_type != EShiftType::LSL {
					match shift_type {
						EShiftType::LSL => {
							offset = rm << shift;
						}
						EShiftType::LSR => {
							if shift == 0 {
								offset = 0;
							} else {
								offset = rm >> shift;
							}
						}
						EShiftType::ASR => {
							if shift == 0 {
								if (rm & 0x8000_0000) > 0 {
									offset = 0xffff_ffff;
								} else {
									offset = 0;
								}
							} else {
								offset = rm.signed_shr(shift);
							}
						}
						EShiftType::ROR => {
							if shift == 0 {
								offset = ((cpu.get_cpsr().get_c() as u32) << 31) | (rm >> 1);
							} else {
								offset = rm.rotate_right(shift);
							}
						}
					}
				} else {
					offset = rm;
				}
			} else {
				// Immediate
				offset = instruction & 0x0000_0fff;
			}

			let address = if p {
				if u {
					rn.wrapping_add(offset)
				} else {
					rn.wrapping_sub(offset)
				}
			} else {
				rn
			};

			// Forced User Mode
			let old_mode = cpu.get_operating_mode();
			if !p && w {
				cpu.set_operating_mode(EOperatingMode::UserMode);
			}

			if l {
				// Pre Indexed
				if p && w {
					cpu.set_register_value(rn_index, address);
				} else if !p {
					// Post Indexed
					let new_address = if u { rn.wrapping_add(offset) } else { rn.wrapping_sub(offset) };
					cpu.set_register_value(rn_index, new_address);
				}
			}

			if b {
				if l {
					cpu.set_register_value(rd_index, bus.read_8(address) as u32);
				} else {
					let rd = if rd_index == PROGRAM_COUNTER_REGISTER {
						cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 4
					} else {
						cpu.get_register_value(rd_index)
					};
					bus.write_8(address, rd as u8);
				}
			} else {
				if l {
					let data = load_32_from_memory(cpu, bus, address);

					if rd_index == PROGRAM_COUNTER_REGISTER {
						cpu.set_register_value(rd_index, data & !0x3);
					} else {
						cpu.set_register_value(rd_index, data);
					}
				} else {
					let rd = if rd_index == PROGRAM_COUNTER_REGISTER {
						cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 4
					} else {
						cpu.get_register_value(rd_index)
					};
					// NOTE: Forced alignment! (UNPREDICTABLE)
					bus.write_32(address & !0x0000_0003, rd);
				}
			}

			if !l {
				// Pre Indexed
				if p && w {
					cpu.set_register_value(rn_index, address);
				} else if !p {
					// Post Indexed
					let new_address = if u { rn.wrapping_add(offset) } else { rn.wrapping_sub(offset) };
					cpu.set_register_value(rn_index, new_address);
				}
			}

			// Restore Mode
			if !p && w {
				cpu.set_operating_mode(old_mode);
			}
		} else if (0x0e00_0090 & instruction) == 0x0000_0090 {
			//LDRSHD/STRSHD Halfword, Doubleword, Signed Data Transfer
			let p = (0x0100_0000 & instruction) != 0;
			let u = (0x0080_0000 & instruction) != 0;
			let i = (0x0040_0000 & instruction) != 0;
			let w = (0x0020_0000 & instruction) != 0;
			let l = (0x0010_0000 & instruction) != 0;

			let h = (0x0000_0020 & instruction) != 0;
			let s = (0x0000_0040 & instruction) != 0;

			let rn_index = ((instruction & 0x000f_0000) >> 16) as u8;
			let rn = cpu.get_register_value(rn_index);
			let rd_index = ((instruction & 0x0000_f000) >> 12) as u8;

			// Instructions don't exist in ARMv4
			debug_assert!((!l && !s && h) || (l && (s || h)), "NOT VALID INSTRUCTION!");

			let offset;
			if i {
				offset = ((instruction & 0x0000_0f00) >> 4) | (instruction & 0x0000_000f);
			} else {
				let rm_index = (instruction & 0x0000_000f) as u8;
				offset = cpu.get_register_value(rm_index);
			}

			let address = if p {
				if u {
					rn.wrapping_add(offset)
				} else {
					rn.wrapping_sub(offset)
				}
			} else {
				rn
			};

			if l {
				// Pre Indexed
				if p && w {
					cpu.set_register_value(rn_index, address);
				} else if !p {
					// Post Indexed
					let new_address = if u { rn.wrapping_add(offset) } else { rn.wrapping_sub(offset) };
					cpu.set_register_value(rn_index, new_address);
				}
			}

			if l {
				let data;
				if h {
					if s {
						if (address & 0x0000_0001) == 0 {
							data = bus.read_16(address) as i16 as u32;
						} else {
							// NOTE: Read byte! (UNPREDICTABLE)
							data = bus.read_8(address) as i8 as u32;
						}
					} else {
						if (address & 0x0000_0001) == 0 {
							data = bus.read_16(address) as u32;
						} else {
							// NOTE: Forced alignment and rotation of data! (UNPREDICTABLE)
							data = (bus.read_16(address & !0x1) as u32).rotate_right(8);
						}
					}
				} else {
					// S
					data = bus.read_8(address) as i8 as u32;
				}

				if rd_index == PROGRAM_COUNTER_REGISTER {
					// NOTE: Forced alignment! (UNPREDICTABLE)
					cpu.set_register_value(rd_index, data & !0x3);
				} else {
					cpu.set_register_value(rd_index, data);
				}
			} else {
				let rd = if rd_index == PROGRAM_COUNTER_REGISTER {
					cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 4
				} else {
					cpu.get_register_value(rd_index)
				};
				// NOTE: Forced alignment! (UNPREDICTABLE)
				bus.write_16(address & !0x1, rd as u16);
			}

			if !l {
				// Pre Indexed
				if p && w {
					cpu.set_register_value(rn_index, address);
				} else if !p {
					// Post Indexed
					let new_address = if u { rn.wrapping_add(offset) } else { rn.wrapping_sub(offset) };
					cpu.set_register_value(rn_index, new_address);
				}
			}
		} else if (0x0e00_0000 & instruction) == 0x0800_0000 {
			// LDM/STM Load/Store multiple registers
			let l = (0x0010_0000 & instruction) != 0;
			let w = (0x0020_0000 & instruction) != 0;
			let s = (0x0040_0000 & instruction) != 0;
			let u = (0x0080_0000 & instruction) != 0;
			let p = (0x0100_0000 & instruction) != 0;

			// NOTE: Forced alignment!!!
			let rn_index = ((instruction & 0x000f_0000) >> 16) as u8;
			let rn = cpu.get_register_value(rn_index);
			let reg_list = &instruction.view_bits::<Lsb0>()[0..16];

			// NOTE: UNPREDICTABLE!!!
			if reg_list.not_any() {
				// Addressing Mode
				let aligned_rn = rn & !0x3;
				let address;
				if u {
					if p {
						address = aligned_rn + 4;
					} else {
						address = aligned_rn;
					}
				} else {
					if p {
						address = aligned_rn.wrapping_sub(0x40);
					} else {
						address = aligned_rn.wrapping_sub(0x40) + 4;
					}
				}

				if w {
					if u {
						cpu.set_register_value(rn_index, rn.wrapping_add(0x40));
					} else {
						cpu.set_register_value(rn_index, rn.wrapping_sub(0x40));
					}
				}

				if l {
					let value = load_32_from_memory(cpu, bus, address);
					cpu.set_register_value(PROGRAM_COUNTER_REGISTER, value);
				} else {
					let value = cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 4;
					bus.write_32(address, value);
				}
			} else {
				// Addressing Mode
				let aligned_rn = rn & !0x3;
				let start_address;
				let end_address;
				if u {
					if p {
						start_address = aligned_rn + 4;
						end_address = aligned_rn.wrapping_add(4 * (reg_list.count_ones() as u32));
					} else {
						start_address = aligned_rn;
						end_address = aligned_rn.wrapping_add(4 * (reg_list.count_ones() as u32)) - 4;
					}
				} else {
					if p {
						start_address = aligned_rn.wrapping_sub(4 * (reg_list.count_ones() as u32));
						end_address = aligned_rn - 4;
					} else {
						start_address = aligned_rn.wrapping_sub(4 * (reg_list.count_ones() as u32)) + 4;
						end_address = aligned_rn;
					}
				}

				let store_rn = reg_list[rn_index as usize];
				let user_bank_transfer = if s {
					if l {
						!reg_list[PROGRAM_COUNTER_REGISTER as usize]
					} else {
						true
					}
				} else {
					false
				};

				let old_mode = cpu.get_operating_mode();
				if user_bank_transfer {
					cpu.set_operating_mode(EOperatingMode::UserMode);
				}

				// NOTE: UNPREDICTABLE BEHAVIOR
				if w && !(l && store_rn) {
					if u {
						cpu.set_register_value(rn_index, rn.wrapping_add(4 * (reg_list.count_ones() as u32)));
					} else {
						cpu.set_register_value(rn_index, rn.wrapping_sub(4 * (reg_list.count_ones() as u32)));
					}
				}

				let mut address = start_address;
				if l {
					for i in 0..15 {
						if reg_list[i] {
							let value = load_32_from_memory(cpu, bus, address);
							cpu.set_register_value(i as u8, value);
							address = address.wrapping_add(4);
						}
					}

					if reg_list[PROGRAM_COUNTER_REGISTER as usize] {
						if s {
							let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
							cpu.get_mut_cpsr().set_value(spsr);
						}

						let value = load_32_from_memory(cpu, bus, address) & !0x3;
						cpu.set_register_value(PROGRAM_COUNTER_REGISTER, value);
						address = address.wrapping_add(4);
					}
					debug_assert_eq!(end_address, address.wrapping_sub(4));
				} else {
					let mut first = true;
					for i in 0..16 {
						if reg_list[i] {
							// NOTE: UNPREDICTABLE BEHAVIOR
							let value = if first && i == rn_index as usize {
								rn
							} else {
								if i as u8 == PROGRAM_COUNTER_REGISTER {
									cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 4
								} else {
									cpu.get_register_value(i as u8)
								}
							};

							bus.write_32(address, value);
							address = address.wrapping_add(4);

							first = false;
						}
					}

					debug_assert_eq!(end_address, address.wrapping_sub(4));
				}

				if user_bank_transfer {
					cpu.set_operating_mode(old_mode);
				}
			}
		} else if (0x0f00_0000 & instruction) == 0x0f00_0000 {
			// SWI Software Interrupt Exception
			cpu.exception(EExceptionType::SoftwareInterrupt);
			return;
		} else if (0x0c00_0000 & instruction) == 0x0000_0000 {
			// ALU
			let i = (0x0200_0000 & instruction) > 0;
			let s = (0x0010_0000 & instruction) > 0;
			let rn_index = ((instruction & 0x000f_0000) >> 16) as u8;
			let mut rn = cpu.get_register_value(rn_index);
			let rd_index = ((instruction & 0x0000_f000) >> 12) as u8;

			let shifter_operand;
			let shifter_carry_out;
			if i {
				let rot = (0x0000_0f00 & instruction) >> 8;
				shifter_operand = (0x0000_00ff & instruction).rotate_right(rot * 2);

				if rot == 0 {
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else {
					shifter_carry_out = (shifter_operand & 0x8000_0000) != 0;
				}
			} else {
				let rm_index = (instruction & 0x0000_000f) as u8;
				let mut rm = cpu.get_register_value(rm_index);
				let r = (instruction & 0x0000_0010) > 0;
				let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 5).unwrap();
				if r {
					let rs = cpu.get_register_value(((0x0000_0f00 & instruction) >> 8) as u8) & 0x0000_00ff;

					// NOTE: When using R15 as operand (Rm or Rn), the returned value depends on the instruction: PC+12 if I=0,R=1 (shift by register), otherwise PC+8 (shift by immediate)
					if rn_index == PROGRAM_COUNTER_REGISTER {
						rn += 4;
					} else if rm_index == PROGRAM_COUNTER_REGISTER {
						rm += 4;
					}

					match shift_type {
						EShiftType::LSL => {
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.get_cpsr().get_c();
							} else if rs < 32 {
								shifter_operand = rm << rs;
								shifter_carry_out = rm.view_bits::<Lsb0>()[32 - rs as usize];
							} else if rs == 32 {
								shifter_operand = 0;
								shifter_carry_out = (rm & 0x0000_0001) > 0;
							} else {
								shifter_operand = 0;
								shifter_carry_out = false;
							}
						}
						EShiftType::LSR => {
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.get_cpsr().get_c();
							} else if rs < 32 {
								shifter_operand = rm >> rs;
								shifter_carry_out = rm.view_bits::<Lsb0>()[(rs - 1) as usize];
							} else if rs == 32 {
								shifter_operand = 0;
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = 0;
								shifter_carry_out = false;
							}
						}
						EShiftType::ASR => {
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.get_cpsr().get_c();
							} else if rs < 32 {
								shifter_operand = rm.signed_shr(rs);
								shifter_carry_out = rm.view_bits::<Lsb0>()[(rs - 1) as usize];
							} else {
								if (rm & 0x8000_0000) == 0 {
									shifter_operand = 0;
								} else {
									shifter_operand = 0xffff_ffff;
								}
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							}
						}
						EShiftType::ROR => {
							let rs_shift = rs & 0x1f;
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.get_cpsr().get_c();
							} else if rs_shift == 0 {
								shifter_operand = rm;
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = rm.rotate_right(rs_shift);
								shifter_carry_out = rm.view_bits::<Lsb0>()[(rs_shift - 1) as usize];
							}
						}
					}
				} else {
					let shift = (0x0000_0f80 & instruction) >> 7;
					match shift_type {
						EShiftType::LSL => {
							if shift == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.get_cpsr().get_c();
							} else {
								shifter_operand = rm << shift;
								shifter_carry_out = rm.view_bits::<Lsb0>()[32 - shift as usize];
							}
						}
						EShiftType::LSR => {
							if shift == 0 {
								shifter_operand = 0;
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = rm >> shift;
								shifter_carry_out = rm.view_bits::<Lsb0>()[(shift - 1) as usize];
							}
						}
						EShiftType::ASR => {
							if shift == 0 {
								if (rm & 0x8000_0000) == 0 {
									shifter_operand = 0;
								} else {
									shifter_operand = 0xffff_ffff;
								}
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = rm.signed_shr(shift);
								shifter_carry_out = rm.view_bits::<Lsb0>()[(shift - 1) as usize];
							}
						}
						EShiftType::ROR => {
							if shift == 0 {
								shifter_operand = ((cpu.get_cpsr().get_c() as u32) << 31) | (rm >> 1);
								shifter_carry_out = (rm & 0x0000_0001) != 0;
							} else {
								shifter_operand = rm.rotate_right(shift);
								shifter_carry_out = rm.view_bits::<Lsb0>()[(shift - 1) as usize];
							}
						}
					}
				}
			};

			match (0x01e0_0000 & instruction) >> 21 {
				// AND
				0x0 => {
					let alu_out = rn & shifter_operand;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				// EOR
				0x1 => {
					let alu_out = rn ^ shifter_operand;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				// SUB
				0x2 => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = rn.overflowing_sub(shifter_operand);
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							// Overflow is sign changes
							let overflow = rn.view_bits::<Lsb0>()[31] != shifter_operand.view_bits::<Lsb0>()[31] && rn.view_bits::<Lsb0>()[31] != alu_out.view_bits::<Lsb0>()[31];

							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(!borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// RSB
				0x3 => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = shifter_operand.overflowing_sub(rn);
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							// Overflow if sign changes
							let overflow =
								shifter_operand.view_bits::<Lsb0>()[31] != rn.view_bits::<Lsb0>()[31] && shifter_operand.view_bits::<Lsb0>()[31] != alu_out.view_bits::<Lsb0>()[31];

							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(!borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				//ADD
				0x4 => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = rn.overflowing_add(shifter_operand);
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							// Overflow if sign changes
							let overflow = rn.view_bits::<Lsb0>()[31] == shifter_operand.view_bits::<Lsb0>()[31] && rn.view_bits::<Lsb0>()[31] != alu_out.view_bits::<Lsb0>()[31];

							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// ADC
				0x5 => {
					// Borrowed if carries bits over
					let (alu_out_first, borrowed_first) = rn.overflowing_add(shifter_operand);
					let c = cpu.get_cpsr().get_c() as u32;
					let (alu_out, borrowed_second) = alu_out_first.overflowing_add(c);
					let borrowed = borrowed_first || borrowed_second;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							// Overflow if sign changes
							let overflow = (rn.view_bits::<Lsb0>()[31] == shifter_operand.view_bits::<Lsb0>()[31]
								&& rn.view_bits::<Lsb0>()[31] != alu_out_first.view_bits::<Lsb0>()[31])
								|| (!alu_out_first.view_bits::<Lsb0>()[31] && alu_out.view_bits::<Lsb0>()[31]);

							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// SBC
				0x6 => {
					// Borrowed if carries bits over
					let (alu_out_first, borrowed_first) = rn.overflowing_sub(shifter_operand);
					let c = !cpu.get_cpsr().get_c() as u32;
					let (alu_out, borrowed_second) = alu_out_first.overflowing_sub(c);
					let borrowed = borrowed_first || borrowed_second;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							// Overflow if sign changes
							let overflow = (rn.view_bits::<Lsb0>()[31] != shifter_operand.view_bits::<Lsb0>()[31]
								&& rn.view_bits::<Lsb0>()[31] != alu_out_first.view_bits::<Lsb0>()[31])
								|| (alu_out_first.view_bits::<Lsb0>()[31] && !alu_out.view_bits::<Lsb0>()[31]);

							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(!borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// RSC
				0x7 => {
					// Borrowed if carries bits over
					let (alu_out_first, borrowed_first) = shifter_operand.overflowing_sub(rn);
					let c = !cpu.get_cpsr().get_c() as u32;
					let (alu_out, borrowed_second) = alu_out_first.overflowing_sub(c);
					let borrowed = borrowed_first || borrowed_second;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							// Overflow if sign changes
							let overflow = (shifter_operand.view_bits::<Lsb0>()[31] != rn.view_bits::<Lsb0>()[31]
								&& shifter_operand.view_bits::<Lsb0>()[31] != alu_out_first.view_bits::<Lsb0>()[31])
								|| (alu_out_first.view_bits::<Lsb0>()[31] && !alu_out.view_bits::<Lsb0>()[31]);

							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(!borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// TST
				0x8 => {
					let alu_out = rn & shifter_operand;

					if rd_index == PROGRAM_COUNTER_REGISTER {
						if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
							let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
							cpu.get_mut_cpsr().set_value(spsr);
						} else {
							// NOTE: UNPREDICTABLE!
						}
					}

					cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(shifter_carry_out);
				}
				// TEQ
				0x9 => {
					let alu_out = rn ^ shifter_operand;

					if rd_index == PROGRAM_COUNTER_REGISTER {
						if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
							let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
							cpu.get_mut_cpsr().set_value(spsr);
						} else {
							// NOTE: UNPREDICTABLE!
						}
					}

					cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(shifter_carry_out);
				}
				// CMPs
				0xa => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = rn.overflowing_sub(shifter_operand);
					// Overflow is sign changes
					let overflow = rn.view_bits::<Lsb0>()[31] != shifter_operand.view_bits::<Lsb0>()[31] && rn.view_bits::<Lsb0>()[31] != alu_out.view_bits::<Lsb0>()[31];

					if rd_index == PROGRAM_COUNTER_REGISTER {
						if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
							let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
							cpu.get_mut_cpsr().set_value(spsr);
						} else {
							// NOTE: UNPREDICTABLE!
						}
					}

					cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(!borrowed);
					cpu.get_mut_cpsr().set_v(overflow);
				}
				// CMN
				0xb => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = rn.overflowing_add(shifter_operand);
					// Overflow is sign changes
					let overflow = rn.view_bits::<Lsb0>()[31] == shifter_operand.view_bits::<Lsb0>()[31] && rn.view_bits::<Lsb0>()[31] != alu_out.view_bits::<Lsb0>()[31];

					if rd_index == PROGRAM_COUNTER_REGISTER {
						if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
							let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
							cpu.get_mut_cpsr().set_value(spsr);
						} else {
							// NOTE: UNPREDICTABLE!
						}
					}

					cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(borrowed);
					cpu.get_mut_cpsr().set_v(overflow);
				}
				// ORR
				0xc => {
					let alu_out = rn | shifter_operand;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				// MOV
				0xd => {
					cpu.set_register_value(rd_index, shifter_operand);

					if s && rd_index == PROGRAM_COUNTER_REGISTER {
						if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
							let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
							cpu.get_mut_cpsr().set_value(spsr);
						}
					} else if s {
						cpu.get_mut_cpsr().set_n((shifter_operand & 0x8000_0000) != 0);
						cpu.get_mut_cpsr().set_z(shifter_operand == 0);
						cpu.get_mut_cpsr().set_c(shifter_carry_out);
					}
				}
				// BIC
				0xe => {
					let alu_out = rn & !shifter_operand;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				// MVN
				0xf => {
					let alu_out = !shifter_operand;
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								// NOTE: UNPREDICTABLE!
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x8000_0000) != 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				_ => {}
			}
		}
	}
}

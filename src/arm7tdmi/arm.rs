use bitvec::order::Lsb0;
use bitvec::prelude::BitView;
use num_traits::{FromPrimitive, PrimInt};

use crate::arm7tdmi::cpu::{CPU, LINK_REGISTER_REGISTER, PROGRAM_COUNTER_REGISTER};
use crate::arm7tdmi::EShiftType;
use crate::arm7tdmi::{sign_extend, EOperatingMode};
use crate::memory::MemoryBus;

fn cond_passed(cpu: &CPU, cond: u8) -> bool {
	match cond {
		0x0 => cpu.get_cpsr().get_z(),                                                      // Equal (Zero)
		0x1 => !cpu.get_cpsr().get_z(),                                                     // Not Equal (Nonzero)
		0x2 => cpu.get_cpsr().get_c(),                                                      // Carry set
		0x3 => !cpu.get_cpsr().get_c(),                                                     // Carry cleared
		0x4 => cpu.get_cpsr().get_n(),                                                      // Signed negative
		0x5 => !cpu.get_cpsr().get_n(),                                                     // Signed positive or zero
		0x6 => cpu.get_cpsr().get_v(),                                                      // Signed overflow
		0x7 => !cpu.get_cpsr().get_v(),                                                     // Signed no overflow
		0x8 => cpu.get_cpsr().get_c() && !cpu.get_cpsr().get_z(),                           // Unsigned higher
		0x9 => !cpu.get_cpsr().get_c() && cpu.get_cpsr().get_z(),                           // Unsigned lower or same
		0xa => cpu.get_cpsr().get_n() == cpu.get_cpsr().get_v(),                            // Signed greater or equal
		0xb => cpu.get_cpsr().get_n() != cpu.get_cpsr().get_v(),                            // Signed less than
		0xc => !cpu.get_cpsr().get_z() && cpu.get_cpsr().get_n() == cpu.get_cpsr().get_v(), // Signed greater than
		0xd => cpu.get_cpsr().get_z() && cpu.get_cpsr().get_n() != cpu.get_cpsr().get_v(),  // Signed less or equal
		_ => true,
	}
}

pub fn operate_arm(cpu: &mut CPU, bus: &mut MemoryBus, instruction: u32) {
	let cond = (instruction >> (32 - 4)) as u8;
	if cond_passed(cpu, cond) {
		if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
			let rm = cpu.get_register_value((instruction & 0x0000_000f) as u8);
			cpu.get_mut_cpsr().set_t((rm & 0x0000_0001) != 0);
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, rm & 0xffff_fffe);
			return;
		} else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
			// Branch
			if 0x0100_0000 & instruction > 0 {
				// Branch with Link
				cpu.set_register_value(LINK_REGISTER_REGISTER, cpu.get_register_value(PROGRAM_COUNTER_REGISTER) + 4);
			}

			let offset = sign_extend(0x00ff_ffff & instruction);
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, (cpu.get_register_value(PROGRAM_COUNTER_REGISTER) as i32 + (offset << 2)) as u32);
			return;
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
						panic!("UNPREDICTABLE!");
					}
					mask = byte_mask & (USER_MASK | PRIV_MASK);
				} else {
					mask = byte_mask & USER_MASK;
				}

				psr = cpu.get_mut_cpsr();
			} else {
				if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
					mask = byte_mask & (USER_MASK | PRIV_MASK | STATE_MASK);
					psr = cpu.get_mut_spsr(cpu.get_operating_mode());
				} else {
					panic!("UNPREDICTABLE!");
				}
			}

			psr.set_value((psr.get_value() & !mask) | (operand & mask));
		} else if (0x0c00_0000 & instruction) == 0x0400_0000 {
			// Single Data Transfer
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
				let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 0).unwrap();
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
					rn + offset
				} else {
					rn - offset
				}
			} else {
				rn
			};

			if b {
				if l {
					cpu.set_register_value(rd_index, bus.read_8(address) as u32);
				} else {
					bus.write_8(address, cpu.get_register_value(rd_index) as u8);
				}
			} else {
				if l {
					cpu.set_register_value(rd_index, bus.read_32(address) as u32);
				} else {
					bus.write_32(address, cpu.get_register_value(rd_index));
				}
			}

			// Pre Indexed
			if p && w {
				cpu.set_register_value(rn_index, address);
			} else if !p {
				// Post Indexed
				if w {
					// TODO: User mode!!!
				}

				let new_address = if u { rn + offset } else { rn - offset };
				cpu.set_register_value(rn_index, new_address);
			}
		} else if (0x0c00_0000 & instruction) == 0x0000_0000 {
			// ALU
			let i = (0x0200_0000 & instruction) > 0;
			let s = (0x0010_0000 & instruction) > 0;
			let rn = cpu.get_register_value(((instruction & 0x000f_0000) >> 16) as u8);
			let rd_index = ((instruction & 0x0000_f000) >> 12) as u8;

			let shifter_operand;
			let shifter_carry_out;
			if i {
				let rot = (0x0000_0f00 & instruction) >> 8;
				shifter_operand = (0x0000_00ff & instruction).rotate_right(rot * 2);

				if rot == 0 {
					shifter_carry_out = cpu.get_cpsr().get_c();
				} else {
					shifter_carry_out = (shifter_operand & 0x800_0000) > 0;
				}
			} else {
				let rm = cpu.get_register_value((instruction & 0x0000_000f) as u8);
				let r = (instruction & 0x0000_0010) > 0;
				let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 5).unwrap();
				if r {
					let rs = cpu.get_register_value(((0x0000_0f00 & instruction) >> 8) as u8) & 0x0000_00ff;

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
								shifter_carry_out = (rm & 0x0000_0001) > 0;
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							// Overflow is sign changes
							let overflow = (rn as i32).is_positive() != (shifter_operand as i32).is_positive() && (rn as i32).is_positive() != (alu_out as i32).is_positive();

							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(!borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// RSB
				0x3 => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = shifter_operand.overflowing_add(rn);
					cpu.set_register_value(rd_index, alu_out);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								panic!("UNPREDICTABLE!");
							}
						} else {
							// Overflow if sign changes
							let overflow =
								(shifter_operand as i32).is_positive() != (rn as i32).is_positive() && (shifter_operand as i32).is_positive() != (alu_out as i32).is_positive();

							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							// Overflow if sign changes
							let overflow = (rn as i32).is_positive() == (shifter_operand as i32).is_positive() && (rn as i32).is_positive() != (alu_out as i32).is_positive();

							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							// Overflow if sign changes
							let overflow = ((rn as i32).is_positive() == (shifter_operand as i32).is_positive()
								&& (rn as i32).is_positive() != (alu_out_first as i32).is_positive())
								|| ((alu_out_first as i32).is_positive() == (c as i32).is_positive() && (alu_out_first as i32).is_positive() != (alu_out as i32).is_positive());

							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							// Overflow if sign changes
							let overflow = ((rn as i32).is_positive() != (shifter_operand as i32).is_positive()
								&& (rn as i32).is_positive() != (alu_out_first as i32).is_positive())
								|| ((alu_out_first as i32).is_positive() != (c as i32).is_positive() && (alu_out_first as i32).is_positive() != (alu_out as i32).is_positive());

							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							// Overflow if sign changes
							let overflow = ((shifter_operand as i32).is_positive() != (rn as i32).is_positive()
								&& (shifter_operand as i32).is_positive() != (alu_out_first as i32).is_positive())
								|| ((alu_out_first as i32).is_positive() != (c as i32).is_positive() && (alu_out_first as i32).is_positive() != (alu_out as i32).is_positive());

							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(!borrowed);
							cpu.get_mut_cpsr().set_v(overflow);
						}
					}
				}
				// TST
				0x8 => {
					let alu_out = rn & shifter_operand;

					cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(shifter_carry_out);
				}
				// TEQ
				0x9 => {
					let alu_out = rn ^ shifter_operand;

					cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(shifter_carry_out);
				}
				// CMP
				0xa => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = rn.overflowing_sub(shifter_operand);
					// Overflow is sign changes
					let overflow = (rn as i32).is_positive() != (shifter_operand as i32).is_positive() && (rn as i32).is_positive() != (alu_out as i32).is_positive();

					cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(!borrowed);
					cpu.get_mut_cpsr().set_v(overflow);
				}
				// CMN
				0xb => {
					// Borrowed if carries bits over
					let (alu_out, borrowed) = rn.overflowing_add(shifter_operand);
					// Overflow is sign changes
					let overflow = (rn as i32).is_positive() == (shifter_operand as i32).is_positive() && (rn as i32).is_positive() != (alu_out as i32).is_positive();

					cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
					cpu.get_mut_cpsr().set_z(alu_out == 0);
					cpu.get_mut_cpsr().set_c(borrowed);
					cpu.get_mut_cpsr().set_v(overflow);
				}
				// ORR
				0xc => {
					cpu.set_register_value(rd_index, rn | shifter_operand);

					if s {
						if rd_index == PROGRAM_COUNTER_REGISTER {
							if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
								let spsr = cpu.get_spsr(cpu.get_operating_mode()).get_value();
								cpu.get_mut_cpsr().set_value(spsr);
							} else {
								panic!("UNPREDICTABLE!");
							}
						} else {
							let rd = cpu.get_register_value(rd_index);
							cpu.get_mut_cpsr().set_n((rd & 0x800_0000) > 0);
							cpu.get_mut_cpsr().set_z(rd == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				// MOV
				0xd => {
					cpu.set_register_value(rd_index, shifter_operand);

					let rd = cpu.get_register_value(rd_index);
					if s && rd_index == PROGRAM_COUNTER_REGISTER {
						if cpu.get_operating_mode() != EOperatingMode::UserMode && cpu.get_operating_mode() != EOperatingMode::SystemMode {
							*cpu.get_mut_cpsr() = cpu.get_spsr(cpu.get_operating_mode()).clone();
						}
					} else if s {
						cpu.get_mut_cpsr().set_n((rd & 0x800_0000) > 0);
						cpu.get_mut_cpsr().set_z(rd == 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
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
								panic!("UNPREDICTABLE!");
							}
						} else {
							cpu.get_mut_cpsr().set_n((alu_out & 0x800_0000) > 0);
							cpu.get_mut_cpsr().set_z(alu_out == 0);
							cpu.get_mut_cpsr().set_c(shifter_carry_out);
						}
					}
				}
				_ => {}
			}
		}
	}

	cpu.set_register_value(PROGRAM_COUNTER_REGISTER, cpu.get_current_pc() + 4);
}

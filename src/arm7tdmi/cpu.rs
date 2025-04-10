use bitvec::prelude::*;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::arm7tdmi::psr::CPSR;
use crate::arm7tdmi::EOperatingMode;

// Special registers
pub const STACK_POINTER_REGISTER: u8 = 13;
pub const LINK_REGISTER_REGISTER: u8 = 14;
pub const PROGRAM_COUNTER_REGISTER: u8 = 15;

pub struct CPU {
	// General Purpose Registers
	registers: [u32; 16],

	// FIQ Registers
	r8_fiq: u32,
	r9_fiq: u32,
	r10_fiq: u32,
	r11_fiq: u32,
	r12_fiq: u32,

	// Stack Pointer Registers
	r13_fiq: u32,
	r13_svc: u32,
	r13_abt: u32,
	r13_irq: u32,
	r13_und: u32,

	// Link Registers
	r14_fiq: u32,
	r14_svc: u32,
	r14_abt: u32,
	r14_irq: u32,
	r14_und: u32,

	// Current Program Status Register
	cpsr: CPSR,

	// Saved Program Status Registers
	spsr_fiq: CPSR,
	spsr_svc: CPSR,
	spsr_abt: CPSR,
	spsr_irq: CPSR,
	spsr_und: CPSR,
}

impl CPU {
	pub fn new() -> Self {
		Self {
			registers: [0; 16],
			r8_fiq: 0,
			r9_fiq: 0,
			r10_fiq: 0,
			r11_fiq: 0,
			r12_fiq: 0,
			r13_fiq: 0,
			r13_svc: 0,
			r13_abt: 0,
			r13_irq: 0,
			r13_und: 0,
			r14_fiq: 0,
			r14_svc: 0,
			r14_abt: 0,
			r14_irq: 0,
			r14_und: 0,
			cpsr: CPSR::new(),
			spsr_fiq: CPSR::new(),
			spsr_svc: CPSR::new(),
			spsr_abt: CPSR::new(),
			spsr_irq: CPSR::new(),
			spsr_und: CPSR::new(),
		}
	}

	pub fn get_registers(&self) -> Box<[u32]> {
		let mode = self.get_operating_mode();
		match mode {
			EOperatingMode::FiqMode => [
				&self.registers[0..8],
				&[self.r8_fiq, self.r9_fiq, self.r10_fiq, self.r11_fiq, self.r12_fiq, self.r13_fiq, self.r14_fiq],
				&[self.registers[15]],
			]
			.concat()
			.into_boxed_slice(),
			EOperatingMode::IrqMode => [&self.registers[0..13], &[self.r13_irq, self.r14_irq], &[self.registers[15]]].concat().into_boxed_slice(),
			EOperatingMode::SupervisorMode => [&self.registers[0..13], &[self.r13_svc, self.r14_svc], &[self.registers[15]]].concat().into_boxed_slice(),
			EOperatingMode::AbortMode => [&self.registers[0..13], &[self.r13_abt, self.r14_abt], &[self.registers[15]]].concat().into_boxed_slice(),
			EOperatingMode::UndefinedMode => [&self.registers[0..13], &[self.r13_und, self.r14_und], &[self.registers[15]]].concat().into_boxed_slice(),
			_ => self.registers.to_vec().into_boxed_slice(),
		}
	}

	pub fn get_current_pc(&self) -> u32 {
		return self.registers[PROGRAM_COUNTER_REGISTER as usize];
	}

	pub fn get_register_value(&self, index: u8) -> u32 {
		// TODO: Check if true for all instructions!!!
		if index == PROGRAM_COUNTER_REGISTER {
			let pc_offset = if self.get_cpsr().get_t() { 4 } else { 8 };
			return self.registers[index as usize] + pc_offset;
		}

		let mode = self.get_operating_mode();
		match mode {
			EOperatingMode::FiqMode => match index {
				8 => self.r8_fiq,
				9 => self.r9_fiq,
				10 => self.r10_fiq,
				11 => self.r11_fiq,
				12 => self.r12_fiq,
				13 => self.r13_fiq,
				14 => self.r14_fiq,
				_ => {
					return self.registers[index as usize];
				}
			},
			EOperatingMode::IrqMode => match index {
				13 => self.r13_irq,
				14 => self.r14_irq,
				_ => {
					return self.registers[index as usize];
				}
			},
			EOperatingMode::SupervisorMode => match index {
				13 => self.r13_svc,
				14 => self.r14_svc,
				_ => {
					return self.registers[index as usize];
				}
			},
			EOperatingMode::AbortMode => match index {
				13 => self.r13_abt,
				14 => self.r14_abt,
				_ => {
					return self.registers[index as usize];
				}
			},
			EOperatingMode::UndefinedMode => match index {
				13 => self.r13_und,
				14 => self.r14_und,
				_ => {
					return self.registers[index as usize];
				}
			},
			_ => {
				return self.registers[index as usize];
			}
		}
	}

	pub fn set_register_value(&mut self, index: u8, value: u32) {
		let mode = self.get_operating_mode();
		match mode {
			EOperatingMode::FiqMode => match index {
				8 => self.r8_fiq = value,
				9 => self.r9_fiq = value,
				10 => self.r10_fiq = value,
				11 => self.r11_fiq = value,
				12 => self.r12_fiq = value,
				13 => self.r13_fiq = value,
				14 => self.r14_fiq = value,
				_ => {
					self.registers[index as usize] = value;
				}
			},
			EOperatingMode::IrqMode => match index {
				13 => self.r13_irq = value,
				14 => self.r14_irq = value,
				_ => {
					self.registers[index as usize] = value;
				}
			},
			EOperatingMode::SupervisorMode => match index {
				13 => self.r13_svc = value,
				14 => self.r14_svc = value,
				_ => {
					self.registers[index as usize] = value;
				}
			},
			EOperatingMode::AbortMode => match index {
				13 => self.r13_abt = value,
				14 => self.r14_abt = value,
				_ => {
					self.registers[index as usize] = value;
				}
			},
			EOperatingMode::UndefinedMode => match index {
				13 => self.r13_und = value,
				14 => self.r14_und = value,
				_ => {
					self.registers[index as usize] = value;
				}
			},
			_ => {
				self.registers[index as usize] = value;
			}
		}
	}

	pub fn get_cpsr(&self) -> &CPSR {
		&self.cpsr
	}

	pub fn get_mut_cpsr(&mut self) -> &mut CPSR {
		&mut self.cpsr
	}

	pub fn get_spsr(&self, mode: EOperatingMode) -> &CPSR {
		match mode {
			EOperatingMode::FiqMode => &self.spsr_fiq,
			EOperatingMode::IrqMode => &self.spsr_irq,
			EOperatingMode::SupervisorMode => &self.spsr_svc,
			EOperatingMode::AbortMode => &self.spsr_abt,
			EOperatingMode::UndefinedMode => &self.spsr_und,
			_ => &self.cpsr,
		}
	}

	pub fn get_mut_spsr(&mut self, mode: EOperatingMode) -> &mut CPSR {
		match mode {
			EOperatingMode::FiqMode => &mut self.spsr_fiq,
			EOperatingMode::IrqMode => &mut self.spsr_irq,
			EOperatingMode::SupervisorMode => &mut self.spsr_svc,
			EOperatingMode::AbortMode => &mut self.spsr_abt,
			EOperatingMode::UndefinedMode => &mut self.spsr_und,
			_ => &mut self.cpsr,
		}
	}

	pub fn get_operating_mode(&self) -> EOperatingMode {
		FromPrimitive::from_u32(self.cpsr.get_mode_bits().load_le()).unwrap()
	}

	pub fn set_operating_mode(&mut self, mode: EOperatingMode) {
		self.cpsr.set_mode_bits(mode.to_u8().unwrap());
	}
}

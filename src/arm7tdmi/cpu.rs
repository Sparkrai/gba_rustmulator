use bitvec::prelude::*;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::arm7tdmi::psr::PSR;
use crate::arm7tdmi::{arm, thumb, EExceptionType, EOperatingMode};
use crate::system::{MemoryInterface, SystemBus};

// Special registers
pub const STACK_POINTER_REGISTER: u8 = 13;
pub const LINK_REGISTER_REGISTER: u8 = 14;
pub const PROGRAM_COUNTER_REGISTER: u8 = 15;

/// Result of a CPU instruction
pub enum CpuResult {
	Continue,
	FlushPipeline,
}

/// Owns the banked register values
pub struct BankedRegisters {
	// UserMode and SystemMode share the same ones
	banked_r13s: [u32; 6],
	banked_r14s: [u32; 6],

	banked_user_registers: [u32; 5],
	banked_fiq_registers: [u32; 5],
}

impl BankedRegisters {
	pub fn new() -> Self {
		Self {
			banked_r13s: [0; 6],
			banked_r14s: [0; 6],
			banked_user_registers: [0; 5],
			banked_fiq_registers: [0; 5],
		}
	}
}

pub struct CPU {
	// General Purpose Registers
	registers: [u32; 16],

	// Current Program Status Register
	cpsr: PSR,

	// Saved Program Status Registers
	spsr_fiq: PSR,
	spsr_svc: PSR,
	spsr_abt: PSR,
	spsr_irq: PSR,
	spsr_und: PSR,

	// Banked Registers
	banks: BankedRegisters,
}

impl CPU {
	pub fn new() -> Self {
		Self {
			registers: [0; 16],
			cpsr: PSR::new(),
			spsr_fiq: PSR::new(),
			spsr_svc: PSR::new(),
			spsr_abt: PSR::new(),
			spsr_irq: PSR::new(),
			spsr_und: PSR::new(),
			banks: BankedRegisters::new(),
		}
	}

	pub fn get_registers(&self) -> &[u32] {
		&self.registers
	}

	pub fn get_current_pc(&self) -> u32 {
		return self.registers[PROGRAM_COUNTER_REGISTER as usize];
	}

	/// The length in bytes of an instruction in the current CPU state.
	/// ARM = 4 - THUMB = 2
	pub fn get_instruction_length(&self) -> u32 {
		if self.cpsr.get_t() {
			2
		} else {
			4
		}
	}

	pub fn get_register_value(&self, index: u8) -> u32 {
		if index == PROGRAM_COUNTER_REGISTER {
			let pc_offset = if self.cpsr.get_t() { 4 } else { 8 };
			return self.registers[index as usize] + pc_offset;
		}

		self.registers[index as usize]
	}

	pub fn set_register_value(&mut self, index: u8, value: u32) {
		self.registers[index as usize] = value;
	}

	pub fn get_cpsr(&self) -> &PSR {
		&self.cpsr
	}

	pub fn get_mut_cpsr(&mut self) -> &mut PSR {
		&mut self.cpsr
	}

	pub fn get_spsr(&self, mode: EOperatingMode) -> &PSR {
		match mode {
			EOperatingMode::FiqMode => &self.spsr_fiq,
			EOperatingMode::IrqMode => &self.spsr_irq,
			EOperatingMode::SupervisorMode => &self.spsr_svc,
			EOperatingMode::AbortMode => &self.spsr_abt,
			EOperatingMode::UndefinedMode => &self.spsr_und,
			_ => &self.cpsr,
		}
	}

	pub fn get_mut_spsr(&mut self, mode: EOperatingMode) -> &mut PSR {
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

	pub fn change_operating_mode(&mut self, new_mode: EOperatingMode, old_mode: EOperatingMode) {
		self.cpsr.set_mode_bits(new_mode.to_u8().unwrap());

		let new_index = match new_mode {
			EOperatingMode::UserMode => 0,
			EOperatingMode::FiqMode => 1,
			EOperatingMode::IrqMode => 2,
			EOperatingMode::SupervisorMode => 3,
			EOperatingMode::AbortMode => 4,
			EOperatingMode::UndefinedMode => 5,
			EOperatingMode::SystemMode => 0,
		};

		let old_index = match old_mode {
			EOperatingMode::UserMode => 0,
			EOperatingMode::FiqMode => 1,
			EOperatingMode::IrqMode => 2,
			EOperatingMode::SupervisorMode => 3,
			EOperatingMode::AbortMode => 4,
			EOperatingMode::UndefinedMode => 5,
			EOperatingMode::SystemMode => 0,
		};

		if new_index == old_index {
			return;
		}

		let r13 = self.registers[13];
		self.registers[13] = self.banks.banked_r13s[new_index];
		self.banks.banked_r13s[old_index] = r13;

		let r14 = self.registers[14];
		self.registers[14] = self.banks.banked_r14s[new_index];
		self.banks.banked_r14s[old_index] = r14;

		// Fiq Mode Registers
		if new_mode == EOperatingMode::FiqMode {
			for (i, fiq_reg) in self.banks.banked_fiq_registers.iter().cloned().enumerate() {
				let reg = self.registers[i + 8];
				self.registers[i + 8] = fiq_reg;
				self.banks.banked_user_registers[i] = reg;
			}
		} else if old_mode == EOperatingMode::FiqMode {
			for (i, fiq_reg) in self.banks.banked_user_registers.iter().cloned().enumerate() {
				let reg = self.registers[i + 8];
				self.registers[i + 8] = fiq_reg;
				self.banks.banked_fiq_registers[i] = reg;
			}
		}
	}

	pub fn exception(&mut self, exception_type: EExceptionType) {
		let exception_vector_address;
		let return_address_offset;
		let operating_mode;
		match exception_type {
			EExceptionType::Reset => {
				exception_vector_address = 0x0;
				return_address_offset = 0x0;
				operating_mode = EOperatingMode::SupervisorMode;
			}
			EExceptionType::Undefined => {
				exception_vector_address = 0x4;
				return_address_offset = self.get_instruction_length();
				operating_mode = EOperatingMode::UndefinedMode;
			}
			EExceptionType::SoftwareInterrupt => {
				exception_vector_address = 0x8;
				return_address_offset = self.get_instruction_length();
				operating_mode = EOperatingMode::SupervisorMode;
			}
			//			EExceptionType::PrefetchAbort => {
			//				exception_vector_address = 0xc;
			//				return_address_offset = 0x4;
			//				operating_mode = EOperatingMode::AbortMode;
			//			}
			//			EExceptionType::DataAbort => {
			//				exception_vector_address = 0x10;
			//				return_address_offset = 0x8;
			//				operating_mode = EOperatingMode::AbortMode;
			//			}
			EExceptionType::Irq => {
				exception_vector_address = 0x18;
				return_address_offset = 0x4;
				operating_mode = EOperatingMode::IrqMode;
			}
			EExceptionType::Fiq => {
				exception_vector_address = 0x1c;
				return_address_offset = 0x4;
				operating_mode = EOperatingMode::FiqMode;
			}
		}

		let old_operating_mode = self.get_operating_mode();
		let cpsr_value = self.cpsr.get_value();
		self.get_mut_spsr(operating_mode).set_value(cpsr_value);

		// Change mode
		self.change_operating_mode(operating_mode, old_operating_mode);

		self.cpsr.set_t(false);
		if exception_type == EExceptionType::Reset || exception_type == EExceptionType::Fiq {
			self.cpsr.set_f(true);
		}
		self.cpsr.set_i(true);

		// Return address
		self.set_register_value(LINK_REGISTER_REGISTER, self.get_current_pc() + return_address_offset);

		self.set_register_value(PROGRAM_COUNTER_REGISTER, exception_vector_address);
	}

	/// Step the CPU by executing 1 instruction
	// TODO: Calculate cycles and update system
	pub fn step(&mut self, bus: &mut SystemBus) {
		// NOTE: Read CPU state
		let pc = self.get_current_pc();
		let result = if self.get_cpsr().get_t() {
			let instruction = bus.read_16(pc);
			thumb::execute_thumb(instruction, self, bus)
		} else {
			let instruction = bus.read_32(pc);
			arm::execute_arm(self, bus, instruction)
		};

		match result {
			CpuResult::Continue => self.set_register_value(PROGRAM_COUNTER_REGISTER, self.get_current_pc() + self.get_instruction_length()),
			CpuResult::FlushPipeline => self.set_register_value(PROGRAM_COUNTER_REGISTER, self.get_current_pc() & !0x1),
		}
	}
}

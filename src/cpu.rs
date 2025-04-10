use bitvec::prelude::*;
use num_derive::*;
use num_traits::{FromPrimitive, ToPrimitive};

pub type Gba32BitSlice = BitSlice<Lsb0, u32>;
pub type GbaRegisterBits = BitArray<Lsb0, [u32; 1]>;

// Special registers
pub const STACK_POINTER_REGISTER: u8 = 13;
pub const LINK_REGISTER_REGISTER: u8 = 14;
pub const PROGRAM_COUNTER_REGISTER: u8 = 15;

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum EOperatingMode {
	UserMode = 0x10,
	FiqMode = 0x11,
	IrqMode = 0x12,
	SupervisorMode = 0x13,
	AbortMode = 0x17,
	UndefinedMode = 0x1b,
	SystemMode = 0x1f,
}

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
			spsr_und: CPSR::new()
		}
	}

	pub fn get_registers(&self) -> &[u32] {
		// TODO: Get based on mode
		&self.registers
	}

	pub fn get_register_value(&self, index: u8) -> u32 {
		// TODO: Check if true for all instructions!!!
		let pc_offset = if self.get_cpsr().get_t() { 4 } else { 8 };
		if index == PROGRAM_COUNTER_REGISTER {
			return self.registers[index as usize] + pc_offset;
		}

		let mode = self.get_operating_mode();
		match mode {
			EOperatingMode::FiqMode => {
				match index {
					7 => self.r8_fiq,
					8 => self.r9_fiq,
					9 => self.r10_fiq,
					10 => self.r11_fiq,
					11 => self.r12_fiq,
					12 => self.r13_fiq,
					13 => self.r14_fiq,
					_ => {
						return self.registers[index as usize];
					}
				}
			}
			EOperatingMode::IrqMode => {
				match index {
					12 => self.r13_irq,
					13 => self.r14_irq,
					_ => {
						return self.registers[index as usize];
					}
				}
			}
			EOperatingMode::SupervisorMode => {
				match index {
					12 => self.r13_irq,
					13 => self.r14_irq,
					_ => {
						return self.registers[index as usize];
					}
				}
			}
			EOperatingMode::AbortMode => {
				match index {
					12 => self.r13_abt,
					13 => self.r14_abt,
					_ => {
						return self.registers[index as usize];
					}
				}
			}
			EOperatingMode::UndefinedMode => {
				match index {
					12 => self.r13_und,
					13 => self.r14_und,
					_ => {
						return self.registers[index as usize];
					}
				}
			}
			_ => {
				return self.registers[index as usize];
			}
		}
	}

	pub fn set_register_value(&mut self, index: u8, value: u32) {
		let mode = self.get_operating_mode();
		match mode {
			EOperatingMode::FiqMode => {
				match index {
					7 => self.r8_fiq = value,
					8 => self.r9_fiq = value,
					9 => self.r10_fiq = value,
					10 => self.r11_fiq = value,
					11 => self.r12_fiq = value,
					12 => self.r13_fiq = value,
					13 => self.r14_fiq = value,
					_ => {
						self.registers[index as usize] = value;
					}
				}
			}
			EOperatingMode::IrqMode => {
				match index {
					12 => self.r13_irq = value,
					13 => self.r14_irq = value,
					_ => {
						self.registers[index as usize] = value;
					}
				}
			}
			EOperatingMode::SupervisorMode => {
				match index {
					12 => self.r13_irq = value,
					13 => self.r14_irq = value,
					_ => {
						self.registers[index as usize] = value;
					}
				}
			}
			EOperatingMode::AbortMode => {
				match index {
					12 => self.r13_abt = value,
					13 => self.r14_abt = value,
					_ => {
						self.registers[index as usize] = value;
					}
				}
			}
			EOperatingMode::UndefinedMode => {
				match index {
					12 => self.r13_und = value,
					13 => self.r14_und = value,
					_ => {
						self.registers[index as usize] = value;
					}
				}
			}
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
			_ => &self.cpsr
		}
	}

	pub fn get_mut_spsr(&mut self, mode: EOperatingMode) -> &mut CPSR {
		match mode {
			EOperatingMode::FiqMode => &mut self.spsr_fiq,
			EOperatingMode::IrqMode => &mut self.spsr_irq,
			EOperatingMode::SupervisorMode => &mut self.spsr_svc,
			EOperatingMode::AbortMode => &mut self.spsr_abt,
			EOperatingMode::UndefinedMode => &mut self.spsr_und,
			_ => &mut self.cpsr
		}
	}

	pub fn get_operating_mode(&self) -> EOperatingMode {
		FromPrimitive::from_u32(self.cpsr.get_mode_bits().load_le()).unwrap()
	}
}

#[derive(Clone)]
pub struct CPSR {
	bits: GbaRegisterBits,
}

impl CPSR {
	pub fn new() -> Self {
		let mut result = Self {
			bits: bitarr![Lsb0, u32; 0; 32],
		};
		result.set_mode_bits(EOperatingMode::SystemMode.to_u8().unwrap());

		return result;
	}

	pub fn get_value(&self) -> u32 {
		self.bits.load_le()
	}

	pub fn set_value(&mut self, value: u32)  {
		self.bits.store_le(value);
	}

	// N - Sign Flag       (0=Not Signed, 1=Signed)
	pub fn get_n(&self) -> bool {
		self.bits[31]
	}

	pub fn set_n(&mut self, value: bool) {
		*self.bits.get_mut(31).unwrap() = value;
	}

	// Z - Zero Flag       (0=Not Zero, 1=Zero)
	pub fn get_z(&self) -> bool {
		self.bits[30]
	}

	pub fn set_z(&mut self, value: bool) {
		*self.bits.get_mut(30).unwrap() = value;
	}

	// C - Carry Flag      (0=Borrow/No Carry, 1=Carry/No Borrow)
	pub fn get_c(&self) -> bool {
		self.bits[29]
	}

	pub fn set_c(&mut self, value: bool) {
		*self.bits.get_mut(29).unwrap() = value;
	}

	// V - Overflow Flag   (0=No Overflow, 1=Overflow)
	pub fn get_v(&self) -> bool {
		self.bits[28]
	}

	pub fn set_v(&mut self, value: bool) {
		*self.bits.get_mut(28).unwrap() = value;
	}

	/// I - IRQ disable     (0=Enable, 1=Disable)
	pub fn get_i(&self) -> bool {
		self.bits[7]
	}

	pub fn set_i(&mut self, value: bool) {
		*self.bits.get_mut(7).unwrap() = value;
	}

	/// F - FIQ disable     (0=Enable, 1=Disable)
	pub fn get_f(&self) -> bool {
		self.bits[6]
	}

	pub fn set_fi(&mut self, value: bool) {
		*self.bits.get_mut(6).unwrap() = value;
	}

	/// T - State Bit       (0=ARM, 1=THUMB)
	pub fn get_t(&self) -> bool {
		self.bits[5]
	}

	pub fn set_t(&mut self, value: bool) {
		*self.bits.get_mut(5).unwrap() = value;
	}

	///  M4-M0 - Mode Bits
	pub fn get_mode_bits(&self) -> &Gba32BitSlice {
		&self.bits[0..=4]
	}

	pub fn set_mode_bits(&mut self, value: u8) {
		self.bits[0..=4].store_le(value);
	}
}
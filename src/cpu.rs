use bitvec::prelude::*;

pub type Gba32BitSlice = BitSlice<Lsb0, u32>;
pub type GbaRegisterBits = BitArray<Lsb0, [u32; 1]>;

pub const STACK_POINTER_REGISTER: u32 = 13;
pub const LINK_REGISTER_REGISTER: u32 = 14;
pub const PROGRAM_COUNTER_REGISTER: u32 = 15;

pub struct CPU {
	// General Purpose Registers
	pub registers: [u32; 16],

	// FIQ Registers
	pub r8_fiq: u32,
	pub r9_fiq: u32,
	pub r10_fiq: u32,
	pub r11_fiq: u32,
	pub r12_fiq: u32,

	// Stack Pointer Registers
	pub r13_fiq: u32,
	pub r13_svc: u32,
	pub r13_abt: u32,
	pub r13_irq: u32,
	pub r13_und: u32,

	// Link Registers
	pub r14_fiq: u32,
	pub r14_svc: u32,
	pub r14_abt: u32,
	pub r14_irq: u32,
	pub r14_und: u32,

	// Current Program Status Register
	pub cpsr: CPSR,

	// Saved Program Status Registers
	pub spsr_fiq: CPSR,
	pub spsr_svc: CPSR,
	pub spsr_abt: CPSR,
	pub spsr_irq: CPSR,
	pub spsr_und: CPSR,
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
}

pub struct CPSR {
	bits: GbaRegisterBits,
}

impl CPSR {
	pub fn new() -> Self {
		Self {
			bits: bitarr![Lsb0, u32; 0; 32],
		}
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
use bitvec::prelude::*;
use num_traits::{ToPrimitive, PrimInt};

use crate::arm7tdmi::{EOperatingMode, GbaRegisterBits, Gba32BitSlice};

#[derive(Clone)]
pub struct CPSR(GbaRegisterBits);

impl CPSR {
	pub fn new() -> Self {
		let mut result = Self {
			0: bitarr![Lsb0, u32; 0; 32],
		};
		result.set_mode_bits(EOperatingMode::SystemMode.to_u8().unwrap());

		return result;
	}

	pub fn get_value(&self) -> u32 {
		self.0.load_le()
	}

	pub fn set_value(&mut self, value: u32) {
		self.0.store_le(value);
	}

	// N - Sign Flag       (0=Not Signed, 1=Signed)
	pub fn get_n(&self) -> bool {
		self.0[31]
	}

	pub fn set_n(&mut self, value: bool) {
		*self.0.get_mut(31).unwrap() = value;
	}

	// Z - Zero Flag       (0=Not Zero, 1=Zero)
	pub fn get_z(&self) -> bool {
		self.0[30]
	}

	pub fn set_z(&mut self, value: bool) {
		*self.0.get_mut(30).unwrap() = value;
	}

	// C - Carry Flag      (0=Borrow/No Carry, 1=Carry/No Borrow)
	pub fn get_c(&self) -> bool {
		self.0[29]
	}

	pub fn set_c(&mut self, value: bool) {
		*self.0.get_mut(29).unwrap() = value;
	}

	// V - Overflow Flag   (0=No Overflow, 1=Overflow)
	pub fn get_v(&self) -> bool {
		self.0[28]
	}

	pub fn set_v(&mut self, value: bool) {
		*self.0.get_mut(28).unwrap() = value;
	}

	/// I - IRQ disable     (0=Enable, 1=Disable)
	pub fn get_i(&self) -> bool {
		self.0[7]
	}

	pub fn set_i(&mut self, value: bool) {
		*self.0.get_mut(7).unwrap() = value;
	}

	/// F - FIQ disable     (0=Enable, 1=Disable)
	pub fn get_f(&self) -> bool {
		self.0[6]
	}

	pub fn set_f(&mut self, value: bool) {
		*self.0.get_mut(6).unwrap() = value;
	}

	/// T - State Bit       (0=ARM, 1=THUMB)
	pub fn get_t(&self) -> bool {
		self.0[5]
	}

	pub fn set_t(&mut self, value: bool) {
		*self.0.get_mut(5).unwrap() = value;
	}

	///  M4-M0 - Mode Bits
	pub fn get_mode_bits(&self) -> &Gba32BitSlice {
		&self.0[0..=4]
	}

	pub fn set_mode_bits(&mut self, value: u8) {
		self.0[0..=4].store_le(value);
	}
}
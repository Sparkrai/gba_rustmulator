use bitvec::prelude::*;
use num_traits::ToPrimitive;

use crate::arm7tdmi::EOperatingMode;
use crate::system::{Gba32BitRegister, Gba32BitSlice};

#[derive(Clone)]
pub struct PSR {
	data: Gba32BitRegister,
}

impl PSR {
	pub fn new() -> Self {
		let mut result = Self { data: bitarr![Lsb0, u32; 0; 32] };
		result.set_mode_bits(EOperatingMode::SystemMode.to_u8().unwrap());

		result
	}

	pub fn get_value(&self) -> u32 {
		self.data.load_le()
	}

	pub fn set_value(&mut self, value: u32) {
		self.data.store_le(value);
	}

	/// N - Sign Flag       (0=Not Signed, 1=Signed)
	pub fn get_n(&self) -> bool {
		self.data[31]
	}

	pub fn set_n(&mut self, value: bool) {
		self.data.set(31, value);
	}

	/// Z - Zero Flag       (0=Not Zero, 1=Zero)
	pub fn get_z(&self) -> bool {
		self.data[30]
	}

	pub fn set_z(&mut self, value: bool) {
		self.data.set(30, value);
	}

	/// C - Carry Flag      (0=Borrow/No Carry, 1=Carry/No Borrow)
	pub fn get_c(&self) -> bool {
		self.data[29]
	}

	pub fn set_c(&mut self, value: bool) {
		self.data.set(29, value);
	}

	/// V - Overflow Flag   (0=No Overflow, 1=Overflow)
	pub fn get_v(&self) -> bool {
		self.data[28]
	}

	pub fn set_v(&mut self, value: bool) {
		self.data.set(28, value);
	}

	/// I - IRQ disable     (0=Enable, 1=Disable)
	pub fn get_i(&self) -> bool {
		self.data[7]
	}

	pub fn set_i(&mut self, value: bool) {
		self.data.set(7, value);
	}

	/// F - FIQ disable     (0=Enable, 1=Disable)
	pub fn get_f(&self) -> bool {
		self.data[6]
	}

	pub fn set_f(&mut self, value: bool) {
		self.data.set(6, value);
	}

	/// T - State Bit       (0=ARM, 1=THUMB)
	pub fn get_t(&self) -> bool {
		self.data[5]
	}

	pub fn set_t(&mut self, value: bool) {
		self.data.set(5, value);
	}

	///  M4-M0 - Mode Bits
	pub fn get_mode_bits(&self) -> &Gba32BitSlice {
		&self.data[0..=4]
	}

	pub fn set_mode_bits(&mut self, value: u8) {
		self.data[0..=4].store_le(value);
	}
}

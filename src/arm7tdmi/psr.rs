use bitfield::*;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::arm7tdmi::EOperatingMode;

bitfield! {
	#[derive(Clone)]
	pub struct PSR(u32);
	impl Debug;
	/// N - Sign Flag       (0=Not Signed, 1=Signed)
	pub get_n, set_n: 31;
	/// Z - Zero Flag       (0=Not Zero, 1=Zero)
	pub get_z, set_z: 30;
	/// C - Carry Flag      (0=Borrow/No Carry, 1=Carry/No Borrow)
	pub get_c, set_c: 29;
	/// V - Overflow Flag   (0=No Overflow, 1=Overflow)
	pub get_v, set_v: 28;
	/// I - IRQ disable     (0=Enable, 1=Disable)
	pub get_i, set_i: 7;
	/// F - FIQ disable     (0=Enable, 1=Disable)
	pub get_f, set_f: 6;
	/// T - State Bit       (0=ARM, 1=THUMB)
	pub get_t, set_t: 5;
	///  M4-M0 - Mode Bits
	pub u8, get_mode_bits, set_mode_bits: 4, 0;
}

impl PSR {
	pub fn new() -> Self {
		Self(EOperatingMode::SystemMode.to_u32().unwrap())
	}
}

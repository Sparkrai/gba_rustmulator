use crate::arm7tdmi::Gba8BitSlice;
use crate::system::MemoryInterface;
use bitvec::prelude::*;
use std::ops::Range;

pub const IO_REGISTERS_END: u32 = 0x3fe;

pub const SOUNDBIAS_RANGE: Range<usize> = 0x88..0x8c;
pub const IE_RANGE: Range<usize> = 0x200..0x202;
pub const IF_RANGE: Range<usize> = 0x202..0x204;
pub const IME_RANGE: Range<usize> = 0x208..0x20a;
pub const HALTCNT_ADDRESS: usize = 0x301;

/// Interrupt Enable Register (R/W)
pub struct IE<'a>(&'a Gba8BitSlice);

impl<'a> IE<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_v_blank(&self) -> bool {
		self.0[0]
	}

	pub fn get_h_blank(&self) -> bool {
		self.0[1]
	}

	pub fn get_v_counter_match(&self) -> bool {
		self.0[2]
	}

	pub fn get_timer0_overflow(&self) -> bool {
		self.0[3]
	}

	pub fn get_timer1_overflow(&self) -> bool {
		self.0[4]
	}

	pub fn get_timer2_overflow(&self) -> bool {
		self.0[5]
	}

	pub fn get_timer3_overflow(&self) -> bool {
		self.0[6]
	}

	pub fn get_serial_communication(&self) -> bool {
		self.0[7]
	}

	pub fn get_dma0(&self) -> bool {
		self.0[8]
	}

	pub fn get_dma1(&self) -> bool {
		self.0[9]
	}

	pub fn get_dma2(&self) -> bool {
		self.0[10]
	}

	pub fn get_dma3(&self) -> bool {
		self.0[11]
	}

	pub fn get_keypad(&self) -> bool {
		self.0[12]
	}

	pub fn get_cartridge(&self) -> bool {
		self.0[13]
	}
}

/// Interrupt Request Flags / IRQ Acknowledge (R/W)
pub struct IF<'a>(&'a mut Gba8BitSlice);

impl<'a> IF<'a> {
	pub fn new(register: &'a mut Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_v_blank(&self) -> bool {
		self.0[0]
	}

	pub fn set_v_blank(&mut self, value: bool) {
		self.0.set(0, value);
	}

	pub fn get_h_blank(&self) -> bool {
		self.0[1]
	}

	pub fn set_h_blank(&mut self, value: bool) {
		self.0.set(1, value);
	}

	pub fn get_v_counter_match(&self) -> bool {
		self.0[2]
	}

	pub fn set_v_counter_match(&mut self, value: bool) {
		self.0.set(2, value);
	}

	pub fn get_timer0_overflow(&self) -> bool {
		self.0[3]
	}

	pub fn set_timer0_overflow(&mut self, value: bool) {
		self.0.set(3, value);
	}

	pub fn get_timer1_overflow(&self) -> bool {
		self.0[4]
	}

	pub fn set_timer1_overflow(&mut self, value: bool) {
		self.0.set(4, value);
	}

	pub fn get_timer2_overflow(&self) -> bool {
		self.0[5]
	}

	pub fn set_timer2_overflow(&mut self, value: bool) {
		self.0.set(5, value);
	}

	pub fn get_timer3_overflow(&self) -> bool {
		self.0[6]
	}

	pub fn set_timer3_overflow(&mut self, value: bool) {
		self.0.set(6, value);
	}

	pub fn get_serial_communication(&self) -> bool {
		self.0[7]
	}

	pub fn set_serial_communication(&mut self, value: bool) {
		self.0.set(7, value);
	}

	pub fn get_dma0(&self) -> bool {
		self.0[8]
	}

	pub fn set_dma0(&mut self, value: bool) {
		self.0.set(8, value);
	}

	pub fn get_dma1(&self) -> bool {
		self.0[9]
	}

	pub fn set_dma1(&mut self, value: bool) {
		self.0.set(9, value);
	}

	pub fn get_dma2(&self) -> bool {
		self.0[10]
	}

	pub fn set_dma2(&mut self, value: bool) {
		self.0.set(10, value);
	}

	pub fn get_dma3(&self) -> bool {
		self.0[11]
	}

	pub fn set_dma3(&mut self, value: bool) {
		self.0.set(11, value);
	}

	pub fn get_keypad(&self) -> bool {
		self.0[12]
	}

	pub fn set_keypad(&mut self, value: bool) {
		self.0.set(12, value);
	}

	pub fn get_cartridge(&self) -> bool {
		self.0[13]
	}

	pub fn set_cartridge(&mut self, value: bool) {
		self.0.set(13, value);
	}
}

/// Sound PWM Control (R/W)
pub struct SoundBias<'a>(&'a mut Gba8BitSlice);

impl<'a> SoundBias<'a> {
	pub fn new(register: &'a mut Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_bias_level(&self) -> u16 {
		self.0[1..=9].load_le()
	}

	pub fn set_bias_level(&mut self, value: u16) {
		self.0[1..=9].store_le(value);
	}

	pub fn get_amplitude_res(&self) -> u8 {
		self.0[14..=15].load_le()
	}
}

/// Represents the hardware registers mapped to memory
pub struct IORegisters {
	registers: Box<[u8]>,
	pub halted: bool,
}

impl IORegisters {
	pub fn new() -> Self {
		let mut result = Self {
			registers: vec![0; IO_REGISTERS_END as usize].into_boxed_slice(),
			halted: false,
		};
		result.get_sound_bias().set_bias_level(0x100);

		result
	}

	pub fn get_sound_bias(&mut self) -> SoundBias {
		SoundBias::new(self.registers[SOUNDBIAS_RANGE].view_bits_mut())
	}

	pub fn get_ie(&self) -> IE {
		IE::new(self.registers[IE_RANGE].view_bits())
	}

	pub fn get_if(&mut self) -> IF {
		IF::new(self.registers[IF_RANGE].view_bits_mut())
	}

	pub fn get_ime(&self) -> bool {
		self.registers[IME_RANGE].view_bits::<Lsb0>()[0]
	}

	pub fn get_is_stop(&self) -> bool {
		self.registers[HALTCNT_ADDRESS].view_bits::<Lsb0>()[7]
	}
}

impl MemoryInterface for IORegisters {
	fn read_8(&self, address: u32) -> u8 {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		if addr <= IO_REGISTERS_END {
			return self.registers[addr as usize];
		}

		0x0 // TODO: Return proper invalid value
	}

	fn write_8(&mut self, address: u32, value: u8) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		if addr <= IO_REGISTERS_END {
			if IF_RANGE.contains(&(addr as usize)) {
				let current_if = self.get_if().0.load_le::<u16>();
				if IF_RANGE.start == addr as usize {
					self.get_if().0.store_le::<u16>(!value as u16 & current_if);
				} else {
					self.get_if().0.store_le::<u16>(!((value as u16) << 8) & current_if);
				}
			} else {
				self.registers[addr as usize] = value;
				if addr == HALTCNT_ADDRESS as u32 {
					self.halted = true;
				}
			}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		if addr <= IO_REGISTERS_END {
			unsafe {
				return *(self.registers.as_ptr().add(addr as usize) as *mut u16) as u16;
			}
		}

		0x0 // TODO: Return proper invalid value
	}

	fn write_16(&mut self, address: u32, value: u16) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		if addr <= IO_REGISTERS_END {
			if IF_RANGE.contains(&(addr as usize)) {
				let current_if = self.get_if().0.load_le::<u16>();
				self.get_if().0.store_le::<u16>(!value & current_if);
			} else {
				unsafe {
					*(self.registers.as_ptr().add(addr as usize) as *mut u16) = value;
				}

				if addr == HALTCNT_ADDRESS as u32 || addr + 1 == HALTCNT_ADDRESS as u32 {
					self.halted = true;
				}
			}
		}
	}

	fn read_32(&self, address: u32) -> u32 {
		unsafe {
			let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
			if addr <= IO_REGISTERS_END {
				return *(self.registers.as_ptr().add(addr as usize) as *mut u32) as u32;
			}

			0x0 // TODO: Return proper invalid value
		}
	}

	fn write_32(&mut self, address: u32, value: u32) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		if addr <= IO_REGISTERS_END {
			if IF_RANGE.contains(&(addr as usize)) {
				let current_if = self.get_if().0.load_le::<u16>();
				self.get_if().0.store_le::<u16>(!value as u16 & current_if);
			} else {
				unsafe {
					*(self.registers.as_ptr().add(addr as usize) as *mut u32) = value;
				}

				if HALTCNT_ADDRESS as u32 <= addr && addr + 3 >= HALTCNT_ADDRESS as u32 {
					self.halted = true;
				}
			}
		}
	}
}

use crate::arm7tdmi::Gba8BitSlice;
use crate::system::MemoryInterface;
use bitvec::prelude::*;
use std::ops::Range;

pub const IO_REGISTERS_END: u32 = 0x3fe;

pub const IE_RANGE: Range<usize> = 0x200..0x202;
pub const IF_RANGE: Range<usize> = 0x202..0x204;
pub const IME_RANGE: Range<usize> = 0x208..0x20a;

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

/// Represents the hardware registers mapped to memory
pub struct IORegisters {
	registers: Box<[u8]>,
}

impl IORegisters {
	pub fn new() -> Self {
		let mut result = Self {
			registers: vec![0; IO_REGISTERS_END as usize].into_boxed_slice(),
		};
		result.set_ime(true);
		result.registers[IE_RANGE].view_bits_mut::<Lsb0>()[0..=13].store_le(0xffffu16);

		result
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

	pub fn set_ime(&mut self, value: bool) {
		self.registers[IME_RANGE].view_bits_mut::<Lsb0>().set(0, value);
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
			self.registers[addr as usize] = value;
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		unsafe {
			let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
			if addr <= IO_REGISTERS_END {
				return *(self.registers.as_ptr().add(addr as usize) as *mut u16) as u16;
			}

			0x0 // TODO: Return proper invalid value
		}
	}

	fn write_16(&mut self, address: u32, value: u16) {
		unsafe {
			let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
			if addr <= IO_REGISTERS_END {
				*(self.registers.as_ptr().add(addr as usize) as *mut u16) = value;
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
		unsafe {
			let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
			if addr <= IO_REGISTERS_END {
				*(self.registers.as_ptr().add(addr as usize) as *mut u32) = value;
			}
		}
	}
}

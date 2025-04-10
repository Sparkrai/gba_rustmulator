use crate::arm7tdmi::{Gba16BitRegister, Gba8BitRegister, Gba8BitSlice};
use crate::system::MemoryInterface;
use bitvec::prelude::*;
use std::ops::Range;

pub const IO_REGISTERS_END: u32 = 0x3fe;

pub const SOUNDBIAS_ADDRESS: u32 = 0x88;
pub const IE_ADDRESS: u32 = 0x200;
pub const IF_ADDRESS: u32 = 0x202;
pub const IME_ADDRESS: u32 = 0x208;
pub const POSTFLG_ADDRESS: u32 = 0x300;
pub const HALTCNT_ADDRESS: u32 = 0x301;

pub struct KeyInput {
	data: Gba16BitRegister,
}

impl KeyInput {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u16; 0; 16] }
	}

	pub fn set_button_a(&mut self, value: bool) {
		self.data.set(0, value);
	}

	pub fn set_button_b(&mut self, value: bool) {
		self.data.set(1, value);
	}

	pub fn set_select(&mut self, value: bool) {
		self.data.set(2, value);
	}

	pub fn set_start(&mut self, value: bool) {
		self.data.set(3, value);
	}

	pub fn set_right(&mut self, value: bool) {
		self.data.set(4, value);
	}

	pub fn set_left(&mut self, value: bool) {
		self.data.set(5, value);
	}

	pub fn set_up(&mut self, value: bool) {
		self.data.set(6, value);
	}

	pub fn set_down(&mut self, value: bool) {
		self.data.set(7, value);
	}

	pub fn set_button_r(&mut self, value: bool) {
		self.data.set(8, value);
	}

	pub fn set_button_l(&mut self, value: bool) {
		self.data.set(9, value);
	}
}

/// Interrupt Enable Register (R/W)
pub struct IE {
	data: Gba16BitRegister,
}

impl IE {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u16; 0; 16] }
	}

	pub fn get_v_blank(&self) -> bool {
		self.data[0]
	}

	pub fn get_h_blank(&self) -> bool {
		self.data[1]
	}

	pub fn get_v_counter_match(&self) -> bool {
		self.data[2]
	}

	pub fn get_timer0_overflow(&self) -> bool {
		self.data[3]
	}

	pub fn get_timer1_overflow(&self) -> bool {
		self.data[4]
	}

	pub fn get_timer2_overflow(&self) -> bool {
		self.data[5]
	}

	pub fn get_timer3_overflow(&self) -> bool {
		self.data[6]
	}

	pub fn get_serial_communication(&self) -> bool {
		self.data[7]
	}

	pub fn get_dma0(&self) -> bool {
		self.data[8]
	}

	pub fn get_dma1(&self) -> bool {
		self.data[9]
	}

	pub fn get_dma2(&self) -> bool {
		self.data[10]
	}

	pub fn get_dma3(&self) -> bool {
		self.data[11]
	}

	pub fn get_keypad(&self) -> bool {
		self.data[12]
	}

	pub fn get_cartridge(&self) -> bool {
		self.data[13]
	}
}

/// Interrupt Request Flags / IRQ Acknowledge (R/W)
pub struct IF {
	data: Gba16BitRegister,
}

impl IF {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u16; 0; 16] }
	}

	pub fn get_v_blank(&self) -> bool {
		self.data[0]
	}

	pub fn set_v_blank(&mut self, value: bool) {
		self.data.set(0, value);
	}

	pub fn get_h_blank(&self) -> bool {
		self.data[1]
	}

	pub fn set_h_blank(&mut self, value: bool) {
		self.data.set(1, value);
	}

	pub fn get_v_counter_match(&self) -> bool {
		self.data[2]
	}

	pub fn set_v_counter_match(&mut self, value: bool) {
		self.data.set(2, value);
	}

	pub fn get_timer0_overflow(&self) -> bool {
		self.data[3]
	}

	pub fn set_timer0_overflow(&mut self, value: bool) {
		self.data.set(3, value);
	}

	pub fn get_timer1_overflow(&self) -> bool {
		self.data[4]
	}

	pub fn set_timer1_overflow(&mut self, value: bool) {
		self.data.set(4, value);
	}

	pub fn get_timer2_overflow(&self) -> bool {
		self.data[5]
	}

	pub fn set_timer2_overflow(&mut self, value: bool) {
		self.data.set(5, value);
	}

	pub fn get_timer3_overflow(&self) -> bool {
		self.data[6]
	}

	pub fn set_timer3_overflow(&mut self, value: bool) {
		self.data.set(6, value);
	}

	pub fn get_serial_communication(&self) -> bool {
		self.data[7]
	}

	pub fn set_serial_communication(&mut self, value: bool) {
		self.data.set(7, value);
	}

	pub fn get_dma0(&self) -> bool {
		self.data[8]
	}

	pub fn set_dma0(&mut self, value: bool) {
		self.data.set(8, value);
	}

	pub fn get_dma1(&self) -> bool {
		self.data[9]
	}

	pub fn set_dma1(&mut self, value: bool) {
		self.data.set(9, value);
	}

	pub fn get_dma2(&self) -> bool {
		self.data[10]
	}

	pub fn set_dma2(&mut self, value: bool) {
		self.data.set(10, value);
	}

	pub fn get_dma3(&self) -> bool {
		self.data[11]
	}

	pub fn set_dma3(&mut self, value: bool) {
		self.data.set(11, value);
	}

	pub fn get_keypad(&self) -> bool {
		self.data[12]
	}

	pub fn set_keypad(&mut self, value: bool) {
		self.data.set(12, value);
	}

	pub fn get_cartridge(&self) -> bool {
		self.data[13]
	}

	pub fn set_cartridge(&mut self, value: bool) {
		self.data.set(13, value);
	}
}

/// Undocumented - Low Power Mode Control (W)
pub struct HaltControl {
	data: Gba8BitRegister,
}

impl HaltControl {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u8; 0; 8] }
	}

	pub fn get_is_stop(&self) -> bool {
		self.data[7]
	}
}

/// Sound PWM Control (R/W)
pub struct SoundBias {
	data: Gba16BitRegister,
}

impl SoundBias {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u16; 0; 16] }
	}

	pub fn get_bias_level(&self) -> u16 {
		self.data[1..=9].load_le()
	}

	pub fn set_bias_level(&mut self, value: u16) {
		self.data[1..=9].store_le(value);
	}

	pub fn get_amplitude_res(&self) -> u8 {
		self.data[14..=15].load_le()
	}
}

/// Represents the hardware registers mapped to memory
pub struct IORegisters {
	key_input: KeyInput,
	interrupt_enable: IE,
	interrupt_request: IF,
	ime: bool,
	sound_bias: SoundBias,
	halt_cnt: HaltControl,
	pub halted: bool,
}

impl IORegisters {
	pub fn new() -> Self {
		Self {
			key_input: KeyInput::new(),
			interrupt_enable: IE::new(),
			interrupt_request: IF::new(),
			ime: false,
			sound_bias: SoundBias::new(),
			halt_cnt: HaltControl::new(),
			halted: false,
		}
	}

	pub fn get_sound_bias(&self) -> &SoundBias {
		&self.sound_bias
	}

	pub fn get_ie(&self) -> &IE {
		&self.interrupt_enable
	}

	pub fn get_if(&self) -> &IF {
		&self.interrupt_request
	}

	pub fn get_mut_if(&mut self) -> &mut IF {
		&mut self.interrupt_request
	}

	pub fn get_ime(&self) -> bool {
		self.ime
	}

	pub fn get_is_stop(&self) -> bool {
		self.halt_cnt.get_is_stop()
	}
}

impl MemoryInterface for IORegisters {
	fn read_8(&self, address: u32) -> u8 {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		let shift = (addr as usize & 0x1) * 8;
		match addr & !0x1 {
			IE_ADDRESS => self.interrupt_enable.data[shift..shift + 8].load_le(),
			IF_ADDRESS => self.interrupt_request.data[shift..shift + 8].load_le(),
			IME_ADDRESS => return if shift == 0 { self.ime as u8 } else { 0 },
			HALTCNT_ADDRESS => self.halt_cnt.data.load_le(),
			SOUNDBIAS_ADDRESS => self.sound_bias.data[shift..shift + 8].load_le(),
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_8(&mut self, address: u32, value: u8) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		let shift = (addr as usize & 0x1) * 8;
		match addr & !0x1 {
			IE_ADDRESS => self.interrupt_enable.data[shift..shift + 8].store_le(value),
			IF_ADDRESS => {
				let current_if = self.interrupt_request.data.load_le::<u16>();
				self.interrupt_request.data.store_le(!((value as u16) << shift) & current_if);
			}
			IME_ADDRESS => {
				if shift == 0 {
					self.ime = value.view_bits::<Lsb0>()[0];
				}
			}
			POSTFLG_ADDRESS => {
				if addr == HALTCNT_ADDRESS {
					self.halt_cnt.data.store_le(value);
					self.halted = true;
				}
			}
			SOUNDBIAS_ADDRESS => self.sound_bias.data[shift..shift + 8].store_le(value),
			_ => {}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		match addr {
			IE_ADDRESS => self.interrupt_enable.data.load_le(),
			IF_ADDRESS => self.interrupt_request.data.load_le(),
			IME_ADDRESS => self.ime as u16,
			POSTFLG_ADDRESS => (self.halt_cnt.data.load_le::<u8>() as u16) << 8,
			SOUNDBIAS_ADDRESS => self.sound_bias.data.load_le(),
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_16(&mut self, address: u32, value: u16) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		match addr {
			IE_ADDRESS => self.interrupt_enable.data.store_le(value),
			IF_ADDRESS => {
				let current_if = self.interrupt_request.data.load_le::<u16>();
				self.interrupt_request.data.store_le(!value & current_if);
			}
			IME_ADDRESS => {
				self.ime = value.view_bits::<Lsb0>()[0];
			}
			POSTFLG_ADDRESS => {
				self.halt_cnt.data.store_le((value >> 8) as u8);
				self.halted = true;
			}
			SOUNDBIAS_ADDRESS => self.sound_bias.data.store_le(value),
			_ => {}
		}
	}

	fn read_32(&self, address: u32) -> u32 {
		unsafe {
			let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
			match addr {
				IE_ADDRESS => self.interrupt_enable.data.load_le::<u32>() | (self.interrupt_request.data.load_le::<u32>() << 16),
				IME_ADDRESS => self.ime as u32,
				POSTFLG_ADDRESS => self.halt_cnt.data.load_le::<u32>() << 8,
				SOUNDBIAS_ADDRESS => self.sound_bias.data.load_le::<u32>(),
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_32(&mut self, address: u32, value: u32) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		match addr {
			IE_ADDRESS => {
				self.interrupt_enable.data.store_le(value as u16);

				let current_if = self.interrupt_request.data.load_le::<u16>();
				self.interrupt_request.data.store_le(!(value as u16) & current_if);
			}
			IME_ADDRESS => {
				self.ime = value.view_bits::<Lsb0>()[0];
			}
			POSTFLG_ADDRESS => {
				self.halt_cnt.data.store_le((value >> 8) as u8);
				self.halted = true;
			}
			SOUNDBIAS_ADDRESS => self.sound_bias.data.store_le(value as u16),
			_ => {}
		}
	}
}

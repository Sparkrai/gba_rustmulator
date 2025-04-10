use bitfield::*;

use crate::system::MemoryInterface;

//pub const IO_REGISTERS_END: u32 = 0x3fe;

pub const SOUNDBIAS_ADDRESS: u32 = 0x88;
pub const KEYINPUT_ADDRESS: u32 = 0x130;
pub const IE_ADDRESS: u32 = 0x200;
pub const IF_ADDRESS: u32 = 0x202;
pub const IME_ADDRESS: u32 = 0x208;
pub const POSTFLG_ADDRESS: u32 = 0x300;
pub const HALTCNT_ADDRESS: u32 = 0x301;

bitfield! {
	/// Key Status (R)
	pub struct KeyInput(u16);
	impl Debug;
	pub set_button_a, _: 0;
	pub set_button_b, _: 1;
	pub set_select, _: 2;
	pub set_start, _: 3;
	pub set_right, _: 4;
	pub set_left, _: 5;
	pub set_up, _: 6;
	pub set_down, _: 7;
	pub set_button_r, _: 8;
	pub set_button_l, _: 9;
}

bitfield! {
	/// Interrupt Enable Register (R/W)
	pub struct IE(u16);
	impl Debug;
	pub get_v_blank, _: 0;
	pub get_h_blank, _: 1;
	pub get_v_counter_match, _: 2;
	pub get_timer0_overflow, _: 3;
	pub get_timer1_overflow, _: 4;
	pub get_timer2_overflow, _: 5;
	pub get_timer3_overflow, _: 6;
	pub get_serial_communication, _: 7;
	pub get_dma0, _: 8;
	pub get_dma1, _: 9;
	pub get_dma2, _: 10;
	pub get_dma3, _: 11;
	pub get_keypad, _: 12;
	pub get_cartridge, _: 13;
}

bitfield! {
	/// Interrupt Request Flags / IRQ Acknowledge (R/W)
	pub struct IF(u16);
	impl Debug;
	pub get_v_blank, set_v_blank: 0;
	pub get_h_blank, set_h_blank: 1;
	pub get_v_counter_match, set_v_counter_match: 2;
	pub get_timer0_overflow, set_timer0_overflow: 3;
	pub get_timer1_overflow, set_timer1_overflow: 4;
	pub get_timer2_overflow, set_timer2_overflow: 5;
	pub get_timer3_overflow, set_timer3_overflow: 6;
	pub get_serial_communication, set_serial_communication: 7;
	pub get_dma0, set_dma0: 8;
	pub get_dma1, set_dma1: 9;
	pub get_dma2, set_dma2: 10;
	pub get_dma3, set_dma3: 11;
	pub get_keypad, set_keypad: 12;
	pub get_cartridge, set_cartridge: 13;
}

bitfield! {
	/// Undocumented - Post Boot / Debug Control (R/W)
	pub struct PostBootFlag(u8);
	impl Debug;
	pub get_is_not_first, _: 0;
}

bitfield! {
	/// Undocumented - Low Power Mode Control (W)
	pub struct HaltControl(u8);
	impl Debug;
	pub get_is_stop, _: 7;
}

bitfield! {
	/// Sound PWM Control (R/W)
	pub struct SoundBias(u32);
	impl Debug;
	pub u16, get_bias_level, set_bias_level: 9, 1;
	pub u8, get_amplitude_res, _: 15, 14;
}

/// Represents the hardware registers mapped to memory
pub struct IORegisters {
	sound_bias: SoundBias,
	key_input: KeyInput,
	interrupt_enable: IE,
	interrupt_request: IF,
	ime: bool,
	post_flag: PostBootFlag,
	halt_cnt: HaltControl,
	pub halted: bool,
}

impl IORegisters {
	pub fn new() -> Self {
		Self {
			sound_bias: SoundBias(0x200),
			key_input: KeyInput(0),
			interrupt_enable: IE(0),
			interrupt_request: IF(0),
			ime: false,
			post_flag: PostBootFlag(0),
			halt_cnt: HaltControl(0),
			halted: false,
		}
	}

	pub fn get_mut_key_input(&mut self) -> &mut KeyInput {
		&mut self.key_input
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

	pub fn get_sound_bias(&self) -> &SoundBias {
		&self.sound_bias
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
			SOUNDBIAS_ADDRESS => self.sound_bias.bit_range(shift + 7, shift),
			KEYINPUT_ADDRESS => self.key_input.bit_range(shift + 7, shift),
			IE_ADDRESS => self.interrupt_enable.bit_range(shift + 7, shift),
			IF_ADDRESS => self.interrupt_request.bit_range(shift + 7, shift),
			IME_ADDRESS => {
				if shift == 0 {
					self.ime as u8
				} else {
					0
				}
			}
			POSTFLG_ADDRESS => {
				if shift == 0 {
					self.post_flag.0
				} else {
					0
				}
			}
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_8(&mut self, address: u32, value: u8) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		let shift = (addr as usize & 0x1) * 8;
		match addr & !0x1 {
			SOUNDBIAS_ADDRESS => self.sound_bias.set_bit_range(shift + 7, shift, value),
			IE_ADDRESS => self.interrupt_enable.set_bit_range(shift + 7, shift, value),
			IF_ADDRESS => {
				let current_if = self.interrupt_request.0;
				self.interrupt_request.0 = !((value as u16) << shift) & current_if;
			}
			IME_ADDRESS => {
				if shift == 0 {
					self.ime = value.bit(0);
				}
			}
			POSTFLG_ADDRESS => {
				if addr == HALTCNT_ADDRESS {
					self.halt_cnt.0 = value;
					self.halted = true;
				} else {
					self.post_flag.0 = value;
				}
			}
			_ => {}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		match addr {
			SOUNDBIAS_ADDRESS => self.sound_bias.0 as u16,
			KEYINPUT_ADDRESS => self.key_input.0,
			IE_ADDRESS => self.interrupt_enable.0,
			IF_ADDRESS => self.interrupt_request.0,
			IME_ADDRESS => self.ime as u16,
			POSTFLG_ADDRESS => self.post_flag.0 as u16,
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_16(&mut self, address: u32, value: u16) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		let shift = (addr as usize & 0x2) * 16;
		match addr {
			IE_ADDRESS => self.interrupt_enable.0 = value,
			IF_ADDRESS => {
				let current_if = self.interrupt_request.0;
				self.interrupt_request.0 = !value & current_if;
			}
			IME_ADDRESS => {
				self.ime = value.bit(0);
			}
			POSTFLG_ADDRESS => {
				self.post_flag.0 = value as u8;
				self.halt_cnt.0 = (value >> 8) as u8;
				self.halted = true;
			}
			SOUNDBIAS_ADDRESS => self.sound_bias.set_bit_range(shift + 15, shift, value),
			_ => {}
		}
	}

	fn read_32(&self, address: u32) -> u32 {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		match addr {
			SOUNDBIAS_ADDRESS => self.sound_bias.0,
			KEYINPUT_ADDRESS => self.key_input.0 as u32,
			IE_ADDRESS => self.interrupt_enable.0 as u32 | ((self.interrupt_request.0 as u32) << 16),
			IME_ADDRESS => self.ime as u32,
			POSTFLG_ADDRESS => self.post_flag.0 as u32,
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_32(&mut self, address: u32, value: u32) {
		let addr = if address & 0xffff == 0x8000 { 0x800 } else { address & 0x00ff_ffff };
		match addr {
			IE_ADDRESS => {
				self.interrupt_enable.0 = value as u16;

				let current_if = self.interrupt_request.0;
				self.interrupt_request.0 = !((value << 16) as u16) & current_if;
			}
			IME_ADDRESS => {
				self.ime = value.bit(0);
			}
			POSTFLG_ADDRESS => {
				self.post_flag.0 = value as u8;
				self.halt_cnt.0 = (value >> 8) as u8;
				self.halted = true;
			}
			SOUNDBIAS_ADDRESS => self.sound_bias.0 = value,
			_ => {}
		}
	}
}

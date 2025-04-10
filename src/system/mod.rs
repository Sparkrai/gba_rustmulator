mod io;

use crate::ppu::{PPU, PPU_REGISTERS_END};
use crate::system::io::{IORegisters, IO_REGISTERS_END};

// Sizes
pub const EWRAM_SIZE: usize = 256 * 1024;
pub const IWRAM_SIZE: usize = 32 * 1024;

pub const CARTRIDGE_ROM_SIZE: usize = 0x01FF_FFFF; // 32Mb
pub const CARTRIDGE_SRAM_SIZE: usize = 64 * 1024;

// Addresses
pub const BIOS_ADDR: u32 = 0x0000_0000;
pub const EWRAM_ADDR: u32 = 0x0200_0000;
pub const IWRAM_ADDR: u32 = 0x0300_0000;
pub const IO_ADDR: u32 = 0x0400_0000;
pub const PALETTE_RAM_ADDR: u32 = 0x0500_0000;
pub const VRAM_ADDR: u32 = 0x0600_0000;
pub const OAM_ADDR: u32 = 0x0700_0000;
pub const CARTRIDGE_WS0_LO: u32 = 0x0800_0000;
pub const CARTRIDGE_WS0_HI: u32 = 0x0900_0000;
pub const CARTRIDGE_WS1_LO: u32 = 0x0A00_0000;
pub const CARTRIDGE_WS1_HI: u32 = 0x0B00_0000;
pub const CARTRIDGE_WS2_LO: u32 = 0x0C00_0000;
pub const CARTRIDGE_WS2_HI: u32 = 0x0D00_0000;
pub const CARTRIDGE_SRAM_LO: u32 = 0x0E00_0000;

/// Provides read/write access to system
pub trait MemoryInterface {
	fn read_8(&self, address: u32) -> u8;
	fn write_8(&mut self, address: u32, value: u8);
	fn read_16(&self, address: u32) -> u16;
	fn write_16(&mut self, address: u32, value: u16);
	fn read_32(&self, address: u32) -> u32;
	fn write_32(&mut self, address: u32, value: u32);
}

/// The system bus
///
/// This unit holds a system byte array which represents the address space of the system.
pub struct SystemBus {
	bios: Box<[u8]>,
	ewram: Box<[u8]>,
	iwram: Box<[u8]>,
	pub io_regs: IORegisters,
	pub ppu: PPU,
	cartridge_rom: Box<[u8]>,
	cartridge_sram: Box<[u8]>,
}

impl SystemBus {
	pub fn new_with_cartridge(bios_data: Box<[u8]>, cartridge_data: Box<[u8]>) -> Self {
		Self {
			bios: bios_data,
			ewram: vec![0; EWRAM_SIZE].into_boxed_slice(),
			iwram: vec![0; IWRAM_SIZE].into_boxed_slice(),
			io_regs: IORegisters::new(),
			ppu: PPU::new(),
			cartridge_rom: cartridge_data,
			cartridge_sram: vec![0; CARTRIDGE_SRAM_SIZE].into_boxed_slice(),
		}
	}

	pub fn new(bios_data: Box<[u8]>) -> Self {
		Self {
			bios: bios_data,
			ewram: vec![0; EWRAM_SIZE].into_boxed_slice(),
			iwram: vec![0; IWRAM_SIZE].into_boxed_slice(),
			io_regs: IORegisters::new(),
			ppu: PPU::new(),
			cartridge_rom: Vec::<u8>::new().into_boxed_slice(),
			cartridge_sram: vec![0; CARTRIDGE_SRAM_SIZE].into_boxed_slice(),
		}
	}
}

impl MemoryInterface for SystemBus {
	fn read_8(&self, address: u32) -> u8 {
		match address & 0xff00_0000 {
			BIOS_ADDR => {
				if address <= 0x3fff {
					self.bios[address as usize]
				} else {
					// TODO: Return proper invalid value
					0x0
				}
			}
			EWRAM_ADDR => self.ewram[(address & 0x3_ffff) as usize],
			IWRAM_ADDR => self.iwram[(address & 0x7fff) as usize],
			IO_ADDR => {
				if address & 0x00ff_ffff <= PPU_REGISTERS_END {
					self.ppu.read_8(address)
				} else {
					self.io_regs.read_8(address)
				}
			}
			PALETTE_RAM_ADDR | VRAM_ADDR | OAM_ADDR => self.ppu.read_8(address),
			CARTRIDGE_WS0_LO | CARTRIDGE_WS0_HI | CARTRIDGE_WS1_LO | CARTRIDGE_WS1_HI | CARTRIDGE_WS2_LO | CARTRIDGE_WS2_HI => {
				if self.cartridge_rom.len() == 0 {
					((address / 2) & 0xffff) as u8
				} else {
					self.cartridge_rom[(address & 0xff_ffff) as usize]
				}
			}
			CARTRIDGE_SRAM_LO => self.cartridge_sram[(address & 0xffff) as usize],
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_8(&mut self, address: u32, value: u8) {
		match address & 0xff00_0000 {
			EWRAM_ADDR => self.ewram[(address & 0x3_ffff) as usize] = value,
			IWRAM_ADDR => self.iwram[(address & 0x7fff) as usize] = value,
			IO_ADDR => {
				if address & 0x00ff_ffff <= PPU_REGISTERS_END {
					self.ppu.write_8(address, value);
				} else {
					self.io_regs.write_8(address, value);
				}
			}
			PALETTE_RAM_ADDR | VRAM_ADDR | OAM_ADDR => self.ppu.write_8(address, value),
			CARTRIDGE_WS0_LO | CARTRIDGE_WS0_HI | CARTRIDGE_WS1_LO | CARTRIDGE_WS1_HI | CARTRIDGE_WS2_LO | CARTRIDGE_WS2_HI => {
				if self.cartridge_rom.len() > 0 {
					self.cartridge_rom[(address & 0xff_ffff) as usize] = value
				}
			}
			CARTRIDGE_SRAM_LO => self.cartridge_sram[(address & 0xffff) as usize] = value,
			_ => {}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		unsafe {
			match address & 0xff00_0000 {
				BIOS_ADDR => {
					if address <= 0x3fff {
						*(self.bios.as_ptr().offset(address as isize) as *mut u16) as u16
					} else {
						// TODO: Return proper invalid value
						0x0
					}
				}
				EWRAM_ADDR => *(self.ewram.as_ptr().offset((address & 0x3_ffff) as isize) as *mut u16) as u16,
				IWRAM_ADDR => *(self.iwram.as_ptr().offset((address & 0x7fff) as isize) as *mut u16) as u16,
				IO_ADDR => {
					if address & 0x00ff_ffff <= PPU_REGISTERS_END {
						self.ppu.read_16(address)
					} else {
						self.io_regs.read_16(address)
					}
				}
				PALETTE_RAM_ADDR | VRAM_ADDR | OAM_ADDR => self.ppu.read_16(address),
				CARTRIDGE_WS0_LO | CARTRIDGE_WS0_HI | CARTRIDGE_WS1_LO | CARTRIDGE_WS1_HI | CARTRIDGE_WS2_LO | CARTRIDGE_WS2_HI => {
					if self.cartridge_rom.len() == 0 {
						((address / 2) & 0xffff) as u16
					} else {
						*(self.cartridge_rom.as_ptr().offset((address & 0xff_ffff) as isize) as *mut u16) as u16
					}
				}
				CARTRIDGE_SRAM_LO => *(self.cartridge_sram.as_ptr().offset((address & 0xffff) as isize) as *mut u16) as u16,
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_16(&mut self, address: u32, value: u16) {
		unsafe {
			match address & 0xff00_0000 {
				EWRAM_ADDR => *(self.ewram.as_ptr().offset((address & 0x3_ffff) as isize) as *mut u16) = value,
				IWRAM_ADDR => *(self.iwram.as_ptr().offset((address & 0x7fff) as isize) as *mut u16) = value,
				IO_ADDR => {
					if address & 0x00ff_ffff <= PPU_REGISTERS_END {
						self.ppu.write_16(address, value);
					} else {
						self.io_regs.write_16(address, value);
					}
				}
				PALETTE_RAM_ADDR | VRAM_ADDR | OAM_ADDR => self.ppu.write_16(address, value),
				CARTRIDGE_WS0_LO | CARTRIDGE_WS0_HI | CARTRIDGE_WS1_LO | CARTRIDGE_WS1_HI | CARTRIDGE_WS2_LO | CARTRIDGE_WS2_HI => {
					if self.cartridge_rom.len() > 0 {
						*(self.cartridge_rom.as_ptr().offset((address & 0xff_ffff) as isize) as *mut u16) = value
					}
				}
				CARTRIDGE_SRAM_LO => *(self.cartridge_sram.as_ptr().offset((address & 0xffff) as isize) as *mut u16) = value,
				_ => {}
			}
		}
	}

	fn read_32(&self, address: u32) -> u32 {
		unsafe {
			match address & 0xff00_0000 {
				BIOS_ADDR => {
					if address <= 0x3fff {
						*(self.bios.as_ptr().offset(address as isize) as *mut u32) as u32
					} else {
						// TODO: Return proper invalid value
						0x0
					}
				}
				EWRAM_ADDR => *(self.ewram.as_ptr().offset((address & 0x3_ffff) as isize) as *mut u32) as u32,
				IWRAM_ADDR => *(self.iwram.as_ptr().offset((address & 0x7fff) as isize) as *mut u32) as u32,
				IO_ADDR => {
					if address & 0x00ff_ffff <= PPU_REGISTERS_END {
						self.ppu.read_32(address)
					} else {
						self.io_regs.read_32(address)
					}
				}
				PALETTE_RAM_ADDR | VRAM_ADDR | OAM_ADDR => self.ppu.read_32(address),
				CARTRIDGE_WS0_LO | CARTRIDGE_WS0_HI | CARTRIDGE_WS1_LO | CARTRIDGE_WS1_HI | CARTRIDGE_WS2_LO | CARTRIDGE_WS2_HI => {
					if self.cartridge_rom.len() == 0 {
						(address / 2) & 0xffff
					} else {
						*(self.cartridge_rom.as_ptr().offset((address & 0xff_ffff) as isize) as *mut u32) as u32
					}
				}
				CARTRIDGE_SRAM_LO => *(self.cartridge_sram.as_ptr().offset((address & 0xffff) as isize) as *mut u32) as u32,
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_32(&mut self, address: u32, value: u32) {
		unsafe {
			match address & 0xff00_0000 {
				EWRAM_ADDR => *(self.ewram.as_ptr().offset((address & 0x3_ffff) as isize) as *mut u32) = value,
				IWRAM_ADDR => *(self.iwram.as_ptr().offset((address & 0x7fff) as isize) as *mut u32) = value,
				IO_ADDR => {
					if address & 0x00ff_ffff <= PPU_REGISTERS_END {
						self.ppu.write_32(address, value);
					} else {
						self.io_regs.write_32(address, value);
					}
				}
				PALETTE_RAM_ADDR | VRAM_ADDR | OAM_ADDR => self.ppu.write_32(address, value),
				CARTRIDGE_WS0_LO | CARTRIDGE_WS0_HI | CARTRIDGE_WS1_LO | CARTRIDGE_WS1_HI | CARTRIDGE_WS2_LO | CARTRIDGE_WS2_HI => {
					if self.cartridge_rom.len() > 0 {
						*(self.cartridge_rom.as_ptr().offset((address & 0xff_ffff) as isize) as *mut u32) = value
					}
				}
				CARTRIDGE_SRAM_LO => *(self.cartridge_sram.as_ptr().offset((address & 0xffff) as isize) as *mut u32) = value,
				_ => {}
			}
		}
	}
}

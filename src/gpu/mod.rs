use crate::arm7tdmi::{Gba16BitRegister, Gba16BitSlice, Gba32BitRegister};
use crate::system::MemoryInterface;
use crate::system::{OAM_ADDR, PALETTE_RAM_ADDR, VRAM_ADDR};
use bitvec::prelude::*;

pub const PALETTE_RAM_SIZE: usize = 1 * 1024;
pub const VRAM_SIZE: usize = 96 * 1024;
pub const OAM_SIZE: usize = 1 * 1024;

pub struct BG {
	bg0_cnt: Gba16BitRegister,
	bg0_hofs: Gba16BitRegister,
	bg0_vofs: Gba16BitRegister,
}

pub struct GPU {
	registers: BitArray<Lsb0, [u16; 43]>,
	disp_cnt: Gba16BitRegister,
	//	disp_cnt: &'a Gba16BitSlice,
	//	// green_swap: Gba16BitRegister, // Undocumented - Green Swap
	//	disp_stat: &'a Gba16BitSlice,
	//	vcount: &'a Gba16BitSlice,
	//	bg0_cnt: &'a Gba16BitSlice,
	//	bg1_cnt: &'a Gba16BitSlice,
	//	bg2_cnt: &'a Gba16BitSlice,
	//	bg3_cnt: &'a Gba16BitSlice,
	//	bg0_hofs: &'a Gba16BitSlice,
	//	bg0_vofs: &'a Gba16BitSlice,
	//	bg1_hofs: &'a Gba16BitSlice,
	//	bg1_vofs: &'a Gba16BitSlice,
	//	bg2_hofs: &'a Gba16BitSlice,
	//	bg2_vofs: &'a Gba16BitSlice,
	//	bg3_hofs: &'a Gba16BitSlice,
	//	bg3_vofs: &'a Gba16BitSlice,
	//	bg2_pa: &'a Gba16BitSlice,
	//	bg2_pb: &'a Gba16BitSlice,
	//	bg2_pc: &'a Gba16BitSlice,
	//	bg2_pd: &'a Gba16BitSlice,
	//	bg2_x: Gba32BitRegister,
	//	bg2_y: Gba32BitRegister,
	//	bg3_pa: &'a Gba16BitSlice,
	//	bg3_pb: &'a Gba16BitSlice,
	//	bg3_pc: &'a Gba16BitSlice,
	//	bg3_pd: &'a Gba16BitSlice,
	//	bg3_x: Gba32BitRegister,
	//	bg3_y: Gba32BitRegister,
	//	win0_h: &'a Gba16BitSlice,
	//	win1_h: &'a Gba16BitSlice,
	//	win0_v: &'a Gba16BitSlice,
	//	win1_v: &'a Gba16BitSlice,
	//	win_in: &'a Gba16BitSlice,
	//	win_out: &'a Gba16BitSlice,
	//	mosaic: &'a Gba16BitSlice,
	//	bld_cnt: &'a Gba16BitSlice,
	//	bld_alpha: &'a Gba16BitSlice,
	//	bld_y: &'a Gba16BitSlice,
	palette_ram: Box<[u8]>,
	vram: Box<[u8]>,
	oam: Box<[u8]>,
}

impl GPU {
	pub fn new() -> Self {
		Self {
			registers: bitarr![Lsb0, u16; 0; 43 * 16],
			palette_ram: vec![0; PALETTE_RAM_SIZE].into_boxed_slice(),
			vram: vec![0; VRAM_SIZE].into_boxed_slice(),
			oam: vec![0; OAM_SIZE].into_boxed_slice(),
		}
	}
}

impl MemoryInterface for GPU {
	fn read_8(&self, address: u32) -> u8 {
		match address & 0xff00_0000 {
			PALETTE_RAM_ADDR => self.palette_ram[(address & 0x3ff) as usize],
			VRAM_ADDR => self.vram[(address & 0x17fff) as usize],
			OAM_ADDR => self.oam[(address & 0x3ff) as usize],
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_8(&mut self, address: u32, value: u8) {
		let addr = (address & 0x56) * 8; // Bytes to bits
		match address & 0xff00_0000 {
			crate::system::IO_ADDR => self.registers[addr as usize..(addr + 16) as usize].store_le(value),
			PALETTE_RAM_ADDR => self.palette_ram[(address & 0x3ff) as usize] = value,
			VRAM_ADDR => self.vram[(address & 0x17fff) as usize] = value,
			OAM_ADDR => self.oam[(address & 0x3ff) as usize] = value,
			_ => {}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		unsafe {
			match address & 0xff00_0000 {
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().offset((address & 0x3ff) as isize) as *mut u16) as u16,
				VRAM_ADDR => *(self.vram.as_ptr().offset((address & 0x17fff) as isize) as *mut u16) as u16,
				OAM_ADDR => *(self.oam.as_ptr().offset((address & 0x3ff) as isize) as *mut u16) as u16,
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_16(&mut self, address: u32, value: u16) {
		unsafe {
			match address & 0xff00_0000 {
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().offset((address & 0x3ff) as isize) as *mut u16) = value,
				VRAM_ADDR => *(self.vram.as_ptr().offset((address & 0x17fff) as isize) as *mut u16) = value,
				OAM_ADDR => *(self.oam.as_ptr().offset((address & 0x3ff) as isize) as *mut u16) = value,
				_ => {}
			}
		}
	}

	fn read_32(&self, address: u32) -> u32 {
		unsafe {
			match address & 0xff00_0000 {
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().offset((address & 0x3ff) as isize) as *mut u32) as u32,
				VRAM_ADDR => *(self.vram.as_ptr().offset((address & 0x17fff) as isize) as *mut u32) as u32,
				OAM_ADDR => *(self.oam.as_ptr().offset((address & 0x3ff) as isize) as *mut u32) as u32,
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_32(&mut self, address: u32, value: u32) {
		unsafe {
			match address & 0xff00_0000 {
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().offset((address & 0x3ff) as isize) as *mut u32) = value,
				VRAM_ADDR => *(self.vram.as_ptr().offset((address & 0x17fff) as isize) as *mut u32) = value,
				OAM_ADDR => *(self.oam.as_ptr().offset((address & 0x3ff) as isize) as *mut u32) = value,
				_ => {}
			}
		}
	}
}

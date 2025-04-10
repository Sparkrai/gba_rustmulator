use std::ops::Range;

use bitvec::prelude::*;
use num_derive::*;

use crate::arm7tdmi::{Gba16BitRegister, Gba8BitSlice};
use crate::system::MemoryInterface;
use crate::system::{OAM_ADDR, PALETTE_RAM_ADDR, VRAM_ADDR};
use num_traits::FromPrimitive;

pub const PPU_REGISTERS_END: u32 = 0x56;
pub const SCREEN_TOTAL_PIXELS: usize = 38400;

pub const PALETTE_RAM_SIZE: usize = 1 * 1024;
pub const VRAM_SIZE: usize = 96 * 1024;
pub const OAM_SIZE: usize = 1 * 1024;

pub const DISP_CNT_RANGE: Range<usize> = 0x0..0x2;
pub const DISP_STAT_RANGE: Range<usize> = 0x4..0x6;
pub const VCOUNT_RANGE: Range<usize> = 0x6..0x8;
pub const BG0_CNT_RANGE: Range<usize> = 0x8..0xa;
pub const BG1_CNT_RANGE: Range<usize> = 0xa..0xc;
pub const BG2_CNT_RANGE: Range<usize> = 0xc..0xe;
pub const BG3_CNT_RANGE: Range<usize> = 0xe..0x10;
pub const BG0_HOFS_RANGE: Range<usize> = 0x10..0x12;
pub const BG0_VOFS_RANGE: Range<usize> = 0x12..0x14;
pub const BG1_HOFS_RANGE: Range<usize> = 0x14..0x16;
pub const BG1_VOFS_RANGE: Range<usize> = 0x16..0x18;
pub const BG2_HOFS_RANGE: Range<usize> = 0x18..0x1a;
pub const BG2_VOFS_RANGE: Range<usize> = 0x1a..0x1c;
pub const BG3_HOFS_RANGE: Range<usize> = 0x1c..0x1e;
pub const BG3_VOFS_RANGE: Range<usize> = 0x1e..0x20;
pub const BG2_PA_RANGE: Range<usize> = 0x20..0x22;
pub const BG2_PB_RANGE: Range<usize> = 0x22..0x24;
pub const BG2_PC_RANGE: Range<usize> = 0x24..0x26;
pub const BG2_PD_RANGE: Range<usize> = 0x26..0x28;
pub const BG2_X_RANGE: Range<usize> = 0x28..0x2c;
pub const BG2_Y_RANGE: Range<usize> = 0x2c..0x30;
pub const BG3_PA_RANGE: Range<usize> = 0x30..0x32;
pub const BG3_PB_RANGE: Range<usize> = 0x32..0x34;
pub const BG3_PC_RANGE: Range<usize> = 0x34..0x36;
pub const BG3_PD_RANGE: Range<usize> = 0x36..0x38;
pub const BG3_X_RANGE: Range<usize> = 0x38..0x3c;
pub const BG3_Y_RANGE: Range<usize> = 0x3c..0x40;
pub const WIN0_H_RANGE: Range<usize> = 0x40..0x42;
pub const WIN1_H_RANGE: Range<usize> = 0x42..0x44;
pub const WIN0_V_RANGE: Range<usize> = 0x44..0x46;
pub const WIN1_V_RANGE: Range<usize> = 0x46..0x48;
pub const WIN_IN_RANGE: Range<usize> = 0x48..0x4a;
pub const WIN_OUT_RANGE: Range<usize> = 0x4a..0x4c;
pub const MOSAIC_RANGE: Range<usize> = 0x4c..0x50;
pub const BLD_CNT_RANGE: Range<usize> = 0x50..0x52;
pub const BLD_ALPHA_RANGE: Range<usize> = 0x52..0x54;
pub const BLD_Y_RANGE: Range<usize> = 0x54..0x56;

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum EVideoMode {
	Mode0,
	Mode1,
	Mode2,
	Mode3,
	Mode4,
	Mode5,
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum EBlendMode {
	None,
	AlphaBlending,
	Lighten,
	Darken,
}

pub struct Color {
	red: u8,
	green: u8,
	blue: u8,
}

impl Color {
	pub fn new(data: u16) -> Self {
		let bits = data.view_bits::<Lsb0>();
		let r = bits[0x0..=0x4].load_le::<u8>();
		let g = bits[0x5..=0x9].load_le::<u8>();
		let b = bits[0xa..=0xe].load_le::<u8>();
		Self {
			red: r | r << 5,
			green: g | g << 5,
			blue: b | b << 5,
		}
	}

	pub fn get_red(&self) -> u8 {
		self.red
	}

	pub fn get_green(&self) -> u8 {
		self.green
	}

	pub fn get_blue(&self) -> u8 {
		self.blue
	}
}

pub struct WindowDimensions {
	x: u8,
	x2: u8,
	y: u8,
	y2: u8,
}

impl WindowDimensions {
	pub fn new(h: &Gba8BitSlice, v: &Gba8BitSlice) -> Self {
		Self {
			x: h[8..16].load_le::<u8>(),
			x2: h[0..8].load_le::<u8>() - 1,
			y: v[8..16].load_le::<u8>(),
			y2: v[0..8].load_le::<u8>() - 1,
		}
	}
}

pub struct PPU {
	// Registers
	registers: Box<[u8]>,
	//	disp_cnt: Gba16BitRegister,
	//	// green_swap: Gba16BitRegister, // Undocumented - Green Swap
	//	disp_stat: &'a Gba8BitSlice,
	//	vcount: &'a Gba8BitSlice,
	//	bg0_cnt: &'a Gba8BitSlice,
	//	bg1_cnt: &'a Gba8BitSlice,
	//	bg2_cnt: &'a Gba8BitSlice,
	//	bg3_cnt: &'a Gba8BitSlice,
	//	bg0_hofs: &'a Gba8BitSlice,
	//	bg0_vofs: &'a Gba8BitSlice,
	//	bg1_hofs: &'a Gba8BitSlice,
	//	bg1_vofs: &'a Gba8BitSlice,
	//	bg2_hofs: &'a Gba8BitSlice,
	//	bg2_vofs: &'a Gba8BitSlice,
	//	bg3_hofs: &'a Gba8BitSlice,
	//	bg3_vofs: &'a Gba8BitSlice,
	//	bg2_pa: &'a Gba8BitSlice,
	//	bg2_pb: &'a Gba8BitSlice,
	//	bg2_pc: &'a Gba8BitSlice,
	//	bg2_pd: &'a Gba8BitSlice,
	//	bg2_x: &'a Gba8BitSlice,
	//	bg2_y: &'a Gba8BitSlice,
	//	bg3_pa: &'a Gba8BitSlice,
	//	bg3_pb: &'a Gba8BitSlice,
	//	bg3_pc: &'a Gba8BitSlice,
	//	bg3_pd: &'a Gba8BitSlice,
	//	bg3_x: &'a Gba8BitSlice,
	//	bg3_y: &'a Gba8BitSlice,
	//	win0_h: &'a Gba8BitSlice,
	//	win1_h: &'a Gba8BitSlice,
	//	win0_v: &'a Gba8BitSlice,
	//	win1_v: &'a Gba8BitSlice,
	//	win_in: &'a Gba8BitSlice,
	//	win_out: &'a Gba8BitSlice,
	//	mosaic: &'a Gba8BitSlice,
	//	bld_cnt: &'a Gba8BitSlice,
	//	bld_alpha: &'a Gba8BitSlice,
	//	bld_y: &'a Gba8BitSlice,

	// Memory
	palette_ram: Box<[u8]>,
	vram: Box<[u8]>,
	oam: Box<[u8]>,
}

impl PPU {
	pub fn new() -> Self {
		Self {
			registers: vec![0; 0x56].into_boxed_slice(),
			//			disp_cnt: &registers[DISPCNT_RANGE].view_bits(),
			//			disp_stat: &registers[0x4..0x6].view_bits(),
			//			vcount: &registers[0x6..0x8].view_bits(),
			//			bg0_cnt: &registers[0x8..0xa].view_bits(),
			//			bg1_cnt: &registers[0xa..0xc].view_bits(),
			//			bg2_cnt: &registers[0xc..0xe].view_bits(),
			//			bg3_cnt: &registers[0xe..0x10].view_bits(),
			//			bg0_hofs: &registers[0x10..0x12].view_bits(),
			//			bg0_vofs: &registers[0x12..0x14].view_bits(),
			//			bg1_hofs: &registers[0x14..0x16].view_bits(),
			//			bg1_vofs: &registers[0x16..0x18].view_bits(),
			//			bg2_hofs: &registers[0x18..0x1a].view_bits(),
			//			bg2_vofs: &registers[0x1a..0x1c].view_bits(),
			//			bg3_hofs: &registers[0x1c..0x1e].view_bits(),
			//			bg3_vofs: &registers[0x1e..0x20].view_bits(),
			//			bg2_pa: &registers[0x20..0x22].view_bits(),
			//			bg2_pb: &registers[0x22..0x24].view_bits(),
			//			bg2_pc: &registers[0x24..0x26].view_bits(),
			//			bg2_pd: &registers[0x26..0x28].view_bits(),
			//			bg2_x: &registers[0x28..0x2c].view_bits(),
			//			bg2_y: &registers[0x2c..0x30].view_bits(),
			//			bg3_pa: &registers[0x30..0x32].view_bits(),
			//			bg3_pb: &registers[0x32..0x34].view_bits(),
			//			bg3_pc: &registers[0x34..0x36].view_bits(),
			//			bg3_pd: &registers[0x36..0x38].view_bits(),
			//			bg3_x: &registers[0x38..0x3c].view_bits(),
			//			bg3_y: &registers[0x3c..0x40].view_bits(),
			//			win0_h: &registers[0x40..0x42].view_bits(),
			//			win1_h: &registers[0x42..0x44].view_bits(),
			//			win0_v: &registers[0x44..0x46].view_bits(),
			//			win1_v: &registers[0x46..0x48].view_bits(),
			//			win_in: &registers[0x48..0x4a].view_bits(),
			//			win_out: &registers[0x4a..0x4c].view_bits(),
			//			mosaic: &registers[0x4c..0x50].view_bits(),
			//			bld_cnt: &registers[0x50..0x52].view_bits(),
			//			bld_alpha: &registers[0x52..0x54].view_bits(),
			//			bld_y: &registers[0x54..0x56].view_bits(),
			palette_ram: vec![0; PALETTE_RAM_SIZE].into_boxed_slice(),
			vram: vec![0; VRAM_SIZE].into_boxed_slice(),
			oam: vec![0; OAM_SIZE].into_boxed_slice(),
		}
	}

	pub fn get_disp_cnt(&mut self) -> DispCnt {
		DispCnt::new(self.registers[DISP_CNT_RANGE].view_bits_mut())
	}

	pub fn get_disp_stat(&mut self) -> DispStat {
		DispStat::new(self.registers[DISP_STAT_RANGE].view_bits_mut())
	}

	pub fn get_vcount(&self) -> u8 {
		self.registers[VCOUNT_RANGE].view_bits::<Lsb0>()[0..8].load_le()
	}

	pub fn set_vcount(&mut self, value: u8) {
		self.registers[VCOUNT_RANGE].view_bits_mut::<Lsb0>()[0..8].store_le(value);
	}

	fn get_bg0_cnt(&self) -> BgCnt {
		BgCnt::new(self.registers[BG0_CNT_RANGE].view_bits())
	}

	fn get_bg1_cnt(&self) -> BgCnt {
		BgCnt::new(self.registers[BG1_CNT_RANGE].view_bits())
	}

	fn get_bg2_cnt(&self) -> BgCnt {
		BgCnt::new(self.registers[BG2_CNT_RANGE].view_bits())
	}

	fn get_bg3_cnt(&self) -> BgCnt {
		BgCnt::new(self.registers[BG3_CNT_RANGE].view_bits())
	}

	// FIXME: Check if 8 or 9!!!
	fn get_bg0_hofs(&self) -> u16 {
		self.registers[BG0_HOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg0_vofs(&self) -> u16 {
		self.registers[BG0_VOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg1_hofs(&self) -> u16 {
		self.registers[BG1_HOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg1_vofs(&self) -> u16 {
		self.registers[BG1_VOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg2_hofs(&self) -> u16 {
		self.registers[BG2_HOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg2_vofs(&self) -> u16 {
		self.registers[BG2_VOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg3_hofs(&self) -> u16 {
		self.registers[BG3_HOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg3_vofs(&self) -> u16 {
		self.registers[BG3_VOFS_RANGE].view_bits::<Lsb0>()[0..=9].load_le()
	}

	fn get_bg2_pa(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG2_PA_RANGE].view_bits())
	}

	fn get_bg2_pb(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG2_PB_RANGE].view_bits())
	}

	fn get_bg2_pc(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG2_PC_RANGE].view_bits())
	}

	fn get_bg2_pd(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG2_PD_RANGE].view_bits())
	}

	fn get_bg2_x(&self) -> BgTransform {
		BgTransform::new(self.registers[BG2_X_RANGE].view_bits())
	}

	fn get_bg2_y(&self) -> BgTransform {
		BgTransform::new(self.registers[BG2_Y_RANGE].view_bits())
	}

	fn get_bg3_pa(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG3_PA_RANGE].view_bits())
	}

	fn get_bg3_pb(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG3_PB_RANGE].view_bits())
	}

	fn get_bg3_pc(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG3_PC_RANGE].view_bits())
	}

	fn get_bg3_pd(&self) -> BgPixelIncrement {
		BgPixelIncrement::new(self.registers[BG3_PD_RANGE].view_bits())
	}

	fn get_bg3_x(&self) -> BgTransform {
		BgTransform::new(self.registers[BG3_X_RANGE].view_bits())
	}

	fn get_bg3_y(&self) -> BgTransform {
		BgTransform::new(self.registers[BG3_Y_RANGE].view_bits())
	}

	fn get_win0_dimensions(&self) -> WindowDimensions {
		WindowDimensions::new(self.registers[WIN0_H_RANGE].view_bits(), self.registers[WIN0_V_RANGE].view_bits())
	}

	fn get_win1_dimensions(&self) -> WindowDimensions {
		WindowDimensions::new(self.registers[WIN1_H_RANGE].view_bits(), self.registers[WIN1_V_RANGE].view_bits())
	}

	fn get_win_in(&self) -> WinIn {
		WinIn::new(self.registers[WIN_IN_RANGE].view_bits())
	}

	fn get_win_out(&self) -> WinOut {
		WinOut::new(self.registers[WIN_OUT_RANGE].view_bits())
	}

	fn get_mosaic(&self) -> Mosaic {
		Mosaic::new(self.registers[MOSAIC_RANGE].view_bits())
	}

	fn get_blend_control(&self) -> BlendControl {
		BlendControl::new(self.registers[BLD_CNT_RANGE].view_bits())
	}

	fn get_a_blend_alpha(&self) -> u8 {
		self.registers[BLD_ALPHA_RANGE].view_bits::<Lsb0>()[0..=4].load_le()
	}

	fn get_b_blend_alpha(&self) -> u8 {
		self.registers[BLD_ALPHA_RANGE].view_bits::<Lsb0>()[8..=12].load_le()
	}

	fn get_blend_brightness(&self) -> u8 {
		self.registers[BLD_Y_RANGE].view_bits::<Lsb0>()[0..=4].load_le()
	}

	pub fn render(&mut self) -> Vec<u8> {
		let mut pixels = vec![0; SCREEN_TOTAL_PIXELS * 3];
		let video_mode = self.get_disp_cnt().get_bg_mode();

		match video_mode {
			EVideoMode::Mode0 => {}
			EVideoMode::Mode1 => {}
			EVideoMode::Mode2 => {
				let bg3_cnt = self.get_bg3_cnt();
				let bg3_wraparound = bg3_cnt.get_overflow_wraparound();
				let bg3_tile_size = match bg3_cnt.get_size() {
					0x0 => 16,
					0x1 => 32,
					0x2 => 64,
					0x3 => 128,
					_ => {
						panic!("IMPOSSIBLE!")
					}
				};

				let bg3_x = self.get_bg3_x().get_value();
				let bg3_y = self.get_bg3_y().get_value();

				for x in 0..240 {
					for y in 0..160 {
						let pixel = (bg3_x as usize + x) * bg3_tile_size + bg3_y as usize + y;
						let tile = pixel / 8;
						let tile_number = self.vram[bg3_cnt.get_map_data_address() + tile] as usize;

						let palette_color_index = (self.vram[bg3_cnt.get_tile_data_address() + (tile_number * 64) + (pixel % 8)] * 2) as usize;
						let color = Color::new(((self.palette_ram[palette_color_index + 1] as u16) << 8 | self.palette_ram[palette_color_index] as u16));

						let pixel_index = x * 240 + y;
						pixels[pixel_index] = color.get_red();
						pixels[pixel_index + 1] = color.get_green();
						pixels[pixel_index + 2] = color.get_blue();
					}
				}
			}
			EVideoMode::Mode3 => {}
			EVideoMode::Mode4 => {}
			EVideoMode::Mode5 => {}
		}

		pixels
	}
}

pub struct DispCnt<'a>(&'a mut Gba8BitSlice);

impl<'a> DispCnt<'a> {
	pub fn new(register: &'a mut Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_bg_mode(&self) -> EVideoMode {
		FromPrimitive::from_u8(self.0[0..=2].load_le()).unwrap()
	}

	pub fn get_display_frame_1(&self) -> bool {
		self.0[4]
	}

	pub fn get_hblank_interval_free(&self) -> bool {
		self.0[5]
	}

	pub fn get_obj_vram_mapping_one_dimensional(&self) -> bool {
		self.0[6]
	}

	pub fn get_forced_blank(&self) -> bool {
		self.0[7]
	}

	pub fn get_screen_display_bg0(&self) -> bool {
		self.0[8]
	}

	pub fn get_screen_display_bg1(&self) -> bool {
		self.0[9]
	}

	pub fn get_screen_display_bg2(&self) -> bool {
		self.0[10]
	}

	pub fn get_screen_display_bg3(&self) -> bool {
		self.0[11]
	}

	pub fn get_screen_display_obj(&self) -> bool {
		self.0[12]
	}

	pub fn get_window0_display(&self) -> bool {
		self.0[13]
	}

	pub fn get_window1_display(&self) -> bool {
		self.0[14]
	}

	pub fn get_obj_window_display(&self) -> bool {
		self.0[15]
	}
}

pub struct DispStat<'a>(&'a mut Gba8BitSlice);

impl<'a> DispStat<'a> {
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

	pub fn get_v_counter(&self) -> bool {
		self.0[2]
	}

	pub fn get_v_blank_irq(&self) -> bool {
		self.0[3]
	}

	pub fn get_h_blank_irq(&self) -> bool {
		self.0[4]
	}

	pub fn get_v_counter_irq(&self) -> bool {
		self.0[5]
	}

	pub fn get_v_count_trigger(&self) -> u8 {
		self.0[8..16].load_le()
	}
}

struct BgCnt<'a>(&'a Gba8BitSlice);

impl<'a> BgCnt<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_bg_priority(&self) -> u8 {
		self.0[0..=1].load_le()
	}

	pub fn get_tile_data_address(&self) -> usize {
		self.0[2..=3].load_le::<usize>() * 0x4000
	}

	pub fn get_mosaic(&self) -> bool {
		self.0[6]
	}

	pub fn get_palette_type(&self) -> bool {
		self.0[7]
	}

	pub fn get_map_data_address(&self) -> usize {
		self.0[8..=12].load_le::<usize>() * 0x800
	}

	pub fn get_overflow_wraparound(&self) -> bool {
		self.0[13]
	}

	pub fn get_size(&self) -> u8 {
		self.0[14..=15].load_le()
	}
}

struct BgPixelIncrement<'a>(&'a Gba8BitSlice);

impl<'a> BgPixelIncrement<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_fractional(&self) -> u8 {
		self.0[0..=7].load_le()
	}

	pub fn get_integer(&self) -> u8 {
		self.0[8..=14].load_le()
	}

	pub fn get_is_negative(&self) -> bool {
		self.0[15]
	}
}

struct BgTransform<'a>(&'a Gba8BitSlice);

impl<'a> BgTransform<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_fractional(&self) -> u8 {
		self.0[0..=7].load_le()
	}

	pub fn get_integer(&self) -> u32 {
		self.0[8..=26].load_le()
	}

	pub fn get_is_negative(&self) -> bool {
		self.0[27]
	}

	pub fn get_value(&self) -> f32 {
		let mut result = self.get_integer() as f32;
		result += self.get_fractional() as f32 * 1.0 / 256.0;

		if self.get_is_negative() {
			result *= -1.0;
		}

		result
	}
}

struct WinIn<'a>(&'a Gba8BitSlice);

impl<'a> WinIn<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_win0_bg0_enabled(&self) -> bool {
		self.0[0]
	}

	pub fn get_win0_bg1_enabled(&self) -> bool {
		self.0[1]
	}

	pub fn get_win0_bg2_enabled(&self) -> bool {
		self.0[2]
	}

	pub fn get_win0_bg3_enabled(&self) -> bool {
		self.0[3]
	}

	pub fn get_win0_obj_enabled(&self) -> bool {
		self.0[4]
	}

	pub fn get_win0_blend_enabled(&self) -> bool {
		self.0[5]
	}

	pub fn get_win1_bg0_enabled(&self) -> bool {
		self.0[8]
	}

	pub fn get_win1_bg1_enabled(&self) -> bool {
		self.0[9]
	}

	pub fn get_win1_bg2_enabled(&self) -> bool {
		self.0[10]
	}

	pub fn get_win1_bg3_enabled(&self) -> bool {
		self.0[11]
	}

	pub fn get_win1_obj_enabled(&self) -> bool {
		self.0[12]
	}

	pub fn get_win1_blend_enabled(&self) -> bool {
		self.0[13]
	}
}

struct WinOut<'a>(&'a Gba8BitSlice);

impl<'a> WinOut<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_outside_bg0_enabled(&self) -> bool {
		self.0[0]
	}

	pub fn get_outside_bg1_enabled(&self) -> bool {
		self.0[1]
	}

	pub fn get_outside_bg2_enabled(&self) -> bool {
		self.0[2]
	}

	pub fn get_outside_bg3_enabled(&self) -> bool {
		self.0[3]
	}

	pub fn get_outside_obj_enabled(&self) -> bool {
		self.0[4]
	}

	pub fn get_outside_blend_enabled(&self) -> bool {
		self.0[5]
	}

	pub fn get_obj_win_bg0_enabled(&self) -> bool {
		self.0[8]
	}

	pub fn get_obj_win_bg1_enabled(&self) -> bool {
		self.0[9]
	}

	pub fn get_obj_win_bg2_enabled(&self) -> bool {
		self.0[10]
	}

	pub fn get_obj_win_bg3_enabled(&self) -> bool {
		self.0[11]
	}

	pub fn get_obj_win_obj_enabled(&self) -> bool {
		self.0[12]
	}

	pub fn get_obj_win_blend_enabled(&self) -> bool {
		self.0[13]
	}
}

struct Mosaic<'a>(&'a Gba8BitSlice);

impl<'a> Mosaic<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_bg_x_size(&self) -> u8 {
		self.0[0..4].load_le()
	}

	pub fn get_bg_y_size(&self) -> u8 {
		self.0[4..8].load_le()
	}

	pub fn get_obj_x_size(&self) -> u8 {
		self.0[8..12].load_le()
	}

	pub fn get_obj_y_size(&self) -> u8 {
		self.0[12..16].load_le()
	}
}

struct BlendControl<'a>(&'a Gba8BitSlice);

impl<'a> BlendControl<'a> {
	pub fn new(register: &'a Gba8BitSlice) -> Self {
		Self { 0: register }
	}

	pub fn get_blend_bg0_source(&self) -> bool {
		self.0[0]
	}

	pub fn get_blend_bg1_source(&self) -> bool {
		self.0[1]
	}

	pub fn get_blend_bg2_source(&self) -> bool {
		self.0[2]
	}

	pub fn get_blend_bg3_source(&self) -> bool {
		self.0[3]
	}

	pub fn get_blend_obj_source(&self) -> bool {
		self.0[4]
	}

	pub fn get_blend_backdrop_source(&self) -> bool {
		self.0[5]
	}

	pub fn get_blend_mode(&self) -> EBlendMode {
		FromPrimitive::from_u8(self.0[6..=7].load_le()).unwrap()
	}

	pub fn get_blend_bg0_target(&self) -> bool {
		self.0[8]
	}

	pub fn get_blend_bg1_target(&self) -> bool {
		self.0[9]
	}

	pub fn get_blend_bg2_target(&self) -> bool {
		self.0[10]
	}

	pub fn get_blend_bg3_target(&self) -> bool {
		self.0[11]
	}

	pub fn get_blend_obj_target(&self) -> bool {
		self.0[12]
	}

	pub fn get_blend_backdrop_target(&self) -> bool {
		self.0[13]
	}
}

impl MemoryInterface for PPU {
	fn read_8(&self, address: u32) -> u8 {
		match address & 0xff00_0000 {
			crate::system::IO_ADDR => {
				let addr = (address & PPU_REGISTERS_END) as usize;
				self.registers[addr]
			}
			PALETTE_RAM_ADDR => self.palette_ram[(address & 0x3ff) as usize],
			VRAM_ADDR => self.vram[(address & 0x17fff) as usize],
			OAM_ADDR => self.oam[(address & 0x3ff) as usize],
			_ => 0x0, // TODO: Return proper invalid value
		}
	}

	fn write_8(&mut self, address: u32, value: u8) {
		match address & 0xff00_0000 {
			crate::system::IO_ADDR => {
				let addr = (address & PPU_REGISTERS_END) as usize;
				self.registers[addr] = value;
			}
			PALETTE_RAM_ADDR => self.palette_ram[(address & 0x3ff) as usize] = value,
			VRAM_ADDR => self.vram[(address & 0x17fff) as usize] = value,
			OAM_ADDR => self.oam[(address & 0x3ff) as usize] = value,
			_ => {}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		unsafe {
			match address & 0xff00_0000 {
				crate::system::IO_ADDR => {
					let addr = (address & PPU_REGISTERS_END) as usize;
					*(self.registers.as_ptr().add(addr) as *mut u16) as u16
				}
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().add((address & 0x3ff) as usize) as *mut u16) as u16,
				VRAM_ADDR => *(self.vram.as_ptr().add((address & 0x17fff) as usize) as *mut u16) as u16,
				OAM_ADDR => *(self.oam.as_ptr().add((address & 0x3ff) as usize) as *mut u16) as u16,
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_16(&mut self, address: u32, value: u16) {
		unsafe {
			match address & 0xff00_0000 {
				crate::system::IO_ADDR => {
					let addr = (address & PPU_REGISTERS_END) as usize;
					*(self.registers.as_ptr().add(addr) as *mut u16) = value;
				}
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().add((address & 0x3ff) as usize) as *mut u16) = value,
				VRAM_ADDR => *(self.vram.as_ptr().add((address & 0x17fff) as usize) as *mut u16) = value,
				OAM_ADDR => *(self.oam.as_ptr().add((address & 0x3ff) as usize) as *mut u16) = value,
				_ => {}
			}
		}
	}

	fn read_32(&self, address: u32) -> u32 {
		unsafe {
			match address & 0xff00_0000 {
				crate::system::IO_ADDR => {
					let addr = (address & PPU_REGISTERS_END) as usize;
					*(self.registers.as_ptr().add(addr) as *mut u32) as u32
				}
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().add((address & 0x3ff) as usize) as *mut u32) as u32,
				VRAM_ADDR => *(self.vram.as_ptr().add((address & 0x17fff) as usize) as *mut u32) as u32,
				OAM_ADDR => *(self.oam.as_ptr().add((address & 0x3ff) as usize) as *mut u32) as u32,
				_ => 0x0, // TODO: Return proper invalid value
			}
		}
	}

	fn write_32(&mut self, address: u32, value: u32) {
		unsafe {
			match address & 0xff00_0000 {
				crate::system::IO_ADDR => {
					let addr = (address & PPU_REGISTERS_END) as usize;
					*(self.registers.as_ptr().add(addr) as *mut u32) = value;
				}
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().add((address & 0x3ff) as usize) as *mut u32) = value,
				VRAM_ADDR => *(self.vram.as_ptr().add((address & 0x17fff) as usize) as *mut u32) = value,
				OAM_ADDR => *(self.oam.as_ptr().add((address & 0x3ff) as usize) as *mut u32) = value,
				_ => {}
			}
		}
	}
}

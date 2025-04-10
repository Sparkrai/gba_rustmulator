use std::ops::Range;

use bitvec::prelude::*;
use num_derive::*;
use num_traits::FromPrimitive;

use crate::arm7tdmi::{sign_extend, Gba16BitRegister, Gba32BitRegister, Gba8BitSlice};
use crate::system::{IORegister, MemoryInterface};
use crate::system::{OAM_ADDR, PALETTE_RAM_ADDR, VRAM_ADDR};

pub const PPU_REGISTERS_END: u32 = 0x56;
pub const SCREEN_TOTAL_PIXELS: usize = 38400;
pub const SPRITE_TILES_START_ADDRESS: usize = 0x10000;
pub const SPRITE_PALETTE_START_ADDRESS: u32 = 0x200;

pub const PALETTE_RAM_SIZE: usize = 1 * 1024;
pub const VRAM_SIZE: usize = 96 * 1024;
pub const OAM_SIZE: usize = 1 * 1024;

pub const DISP_CNT_ADDRESS: u32 = 0x0;
pub const DISP_STAT_ADDRESS: u32 = 0x4;
pub const VCOUNT_ADDRESS: u32 = 0x6;
pub const BG0_CNT_ADDRESS: u32 = 0x8;
pub const BG1_CNT_ADDRESS: u32 = 0xa;
pub const BG2_CNT_ADDRESS: u32 = 0xc;
pub const BG3_CNT_ADDRESS: u32 = 0xe;
pub const BG0_HOFS_ADDRESS: u32 = 0x10;
pub const BG0_VOFS_ADDRESS: u32 = 0x12;
pub const BG1_HOFS_ADDRESS: u32 = 0x14;
pub const BG1_VOFS_ADDRESS: u32 = 0x16;
pub const BG2_HOFS_ADDRESS: u32 = 0x18;
pub const BG2_VOFS_ADDRESS: u32 = 0x1a;
pub const BG3_HOFS_ADDRESS: u32 = 0x1c;
pub const BG3_VOFS_ADDRESS: u32 = 0x1e;
pub const BG2_PA_ADDRESS: u32 = 0x20;
pub const BG2_PB_ADDRESS: u32 = 0x22;
pub const BG2_PC_ADDRESS: u32 = 0x24;
pub const BG2_PD_ADDRESS: u32 = 0x26;
pub const BG2_X_ADDRESS: u32 = 0x28;
pub const BG2_Y_ADDRESS: u32 = 0x2c;
pub const BG3_PA_ADDRESS: u32 = 0x30;
pub const BG3_PB_ADDRESS: u32 = 0x32;
pub const BG3_PC_ADDRESS: u32 = 0x34;
pub const BG3_PD_ADDRESS: u32 = 0x36;
pub const BG3_X_ADDRESS: u32 = 0x38;
pub const BG3_Y_ADDRESS: u32 = 0x3c;
pub const WIN0_H_ADDRESS: u32 = 0x40;
pub const WIN1_H_ADDRESS: u32 = 0x42;
pub const WIN0_V_ADDRESS: u32 = 0x44;
pub const WIN1_V_ADDRESS: u32 = 0x46;
pub const WIN_IN_ADDRESS: u32 = 0x48;
pub const WIN_OUT_ADDRESS: u32 = 0x4a;
pub const MOSAIC_ADDRESS: u32 = 0x4c;
pub const BLD_CNT_ADDRESS: u32 = 0x50;
pub const BLD_ALPHA_ADDRESS: u32 = 0x52;
pub const BLD_Y_ADDRESS: u32 = 0x54;

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive, PartialEq)]
pub enum EVideoMode {
	Mode0,
	Mode1,
	Mode2,
	Mode3,
	Mode4,
	Mode5,
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive, PartialEq)]
pub enum EBlendMode {
	None,
	AlphaBlending,
	Lighten,
	Darken,
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive, PartialEq)]
pub enum ESpriteMode {
	Normal,
	SemiTransparent,
	ObjWindow,
}

pub struct Color {
	red: f32,
	green: f32,
	blue: f32,
}

impl Color {
	pub fn new(data: u16) -> Self {
		let bits = data.view_bits::<Lsb0>();
		let r = bits[0x0..=0x4].load_le::<u8>();
		let g = bits[0x5..=0x9].load_le::<u8>();
		let b = bits[0xa..=0xe].load_le::<u8>();

		// TODO: Gamma correction!!!
		const LCD_GAMMA: f32 = 4.0;
		const OUT_GAMMA: f32 = 2.2;
		let lb = f32::powf(b as f32 / 31.0, LCD_GAMMA);
		let lg = f32::powf(g as f32 / 31.0, LCD_GAMMA);
		let lr = f32::powf(r as f32 / 31.0, LCD_GAMMA);
		let red = f32::powf(0.0 * lb + (50.0 / 255.0) * lg + 1.0 * lr, 1.0 / OUT_GAMMA) * (255.0 / 280.0);
		let green = f32::powf((30.0 / 255.0) * lb + (230.0 / 255.0) * lg + (10.0 / 255.0) * lr, 1.0 / OUT_GAMMA) * (255.0 / 280.0);
		let blue = f32::powf((220.0 / 255.0) * lb + (10.0 / 255.0) * lg + (50.0 / 255.0) * lr, 1.0 / OUT_GAMMA) * (255.0 / 280.0);

		Self { red, green, blue }
	}

	pub fn get_red(&self) -> f32 {
		self.red
	}

	pub fn get_green(&self) -> f32 {
		self.green
	}

	pub fn get_blue(&self) -> f32 {
		self.blue
	}
}

pub struct WindowDimensions {
	h: Gba16BitRegister,
	v: Gba16BitRegister,
}

impl WindowDimensions {
	pub fn new() -> Self {
		Self {
			h: Gba16BitRegister::zeroed(),
			v: Gba16BitRegister::zeroed(),
		}
	}

	pub fn get_x1(&self) -> u8 {
		self.h[8..16].load_le::<u8>()
	}

	pub fn get_x2(&self) -> u8 {
		self.h[0..8].load_le::<u8>()
	}

	pub fn get_y1(&self) -> u8 {
		self.h[8..16].load_le::<u8>()
	}

	pub fn get_y2(&self) -> u8 {
		self.h[0..8].load_le::<u8>()
	}
}

pub struct SpriteEntry<'a>(&'a Gba8BitSlice);

impl<'a> SpriteEntry<'a> {
	pub fn new(registers: &'a Gba8BitSlice) -> Self {
		Self { 0: registers }
	}

	pub fn get_y_coord(&self) -> i32 {
		let y = self.0[0..8].load_le::<u8>() as i32;
		// NOTE: Check if it's wrapping!!!
		if y >= (160) {
			y - (1 << 8)
		} else {
			y
		}
	}

	pub fn get_is_affine(&self) -> bool {
		self.0[8]
	}

	pub fn get_is_virtual_double_sized(&self) -> bool {
		self.0[9]
	}

	pub fn get_sprite_mode(&self) -> ESpriteMode {
		FromPrimitive::from_u8(self.0[0xa..=0xb].load_le()).unwrap()
	}

	pub fn get_is_mosaic(&self) -> bool {
		self.0[0xc]
	}

	pub fn get_is_256_palette(&self) -> bool {
		self.0[0xd]
	}

	pub fn get_size(&self) -> (usize, usize) {
		let value = self.0[0xe..=0xf].load_le::<u8>() << 2 | self.0[0x1e..=0x1f].load_le::<u8>();

		match value {
			0b0000 => (8, 8),
			0b0001 => (16, 16),
			0b0010 => (32, 32),
			0b0011 => (64, 64),
			0b0100 => (16, 8),
			0b0101 => (32, 8),
			0b0110 => (64, 16),
			0b0111 => (64, 32),
			0b1000 => (8, 16),
			0b1001 => (8, 32),
			0b1010 => (16, 32),
			0b1011 => (32, 64),
			_ => panic!("UNRECOGNIZED!"),
		}
	}

	pub fn get_x_coord(&self) -> i32 {
		sign_extend(self.0[0x10..0x19].load_le::<u16>(), 9) as i32
	}

	pub fn get_affine_matrix_index(&self) -> usize {
		self.0[0x19..0x1d].load_le()
	}

	pub fn get_h_flip(&self) -> bool {
		self.0[0x1c]
	}

	pub fn get_v_flip(&self) -> bool {
		self.0[0x1d]
	}

	pub fn get_tile_index(&self) -> usize {
		self.0[0x20..=0x29].load_le()
	}

	pub fn get_priority(&self) -> u8 {
		self.0[0x2a..=0x2b].load_le()
	}

	pub fn get_palette_number(&self) -> u8 {
		self.0[0x2c..=0x2f].load_le()
	}
}

pub struct PPU {
	// Registers
	disp_cnt: DisplayControl,
	// green_swap: Gba16BitRegister, // Undocumented - Green Swap
	disp_stat: DisplayStatus,
	v_count: u8,
	bg_controls: [BackgroundControl; 4],
	bg_hofs: [u16; 4],
	bg_vofs: [u16; 4],
	bg_affine_matrices: [BackgroundAffineMatrix; 2],
	win_dimensions: [WindowDimensions; 2],
	win_in: WinIn,
	win_out: WinOut,
	mosaic: Mosaic,
	bld_cnt: BlendControl,
	bld_alpha: BlendAlpha,
	bld_y: Gba16BitRegister,

	// Memory
	palette_ram: Box<[u8]>,
	vram: Box<[u8]>,
	oam: Box<[u8]>,
}

impl PPU {
	pub fn new() -> Self {
		Self {
			disp_cnt: DisplayControl::new(),
			disp_stat: DisplayStatus::new(),
			v_count: 0,
			bg_controls: [BackgroundControl::new(); 4],
			bg_hofs: [0; 4],
			bg_vofs: [0; 4],
			bg_affine_matrices: [BackgroundAffineMatrix::new(); 2],
			win_dimensions: [WindowDimensions::new(); 2],
			win_in: WinIn::new(),
			win_out: WinOut::new(),
			mosaic: Mosaic::new(),
			bld_cnt: BlendControl::new(),
			bld_alpha: BlendAlpha::new(),
			bld_y: Gba16BitRegister::zeroed(),

			palette_ram: vec![0; PALETTE_RAM_SIZE].into_boxed_slice(),
			vram: vec![0; VRAM_SIZE].into_boxed_slice(),
			oam: vec![0; OAM_SIZE].into_boxed_slice(),
		}
	}

	pub fn get_disp_cnt(&mut self) -> &DisplayControl {
		&self.disp_cnt
	}

	pub fn get_disp_stat(&mut self) -> &DisplayStatus {
		&self.disp_stat
	}

	pub fn get_vcount(&self) -> u8 {
		self.v_count
	}

	pub fn set_vcount(&mut self, value: u8) {
		self.v_count = value
	}

	fn get_bg_cnt(&self, index: usize) -> &BackgroundControl {
		&self.bg_controls[index]
	}

	// FIXME: Check if 8 or 9!!!
	fn get_bg_hofs(&self, index: usize) -> u16 {
		self.bg_hofs[index] & 0x01ff
	}

	fn get_bg_vofs(&self, index: usize) -> u16 {
		self.bg_vofs[index] & 0x01ff
	}

	fn get_bg_affine_matrix(&self, index: usize) -> &BackgroundAffineMatrix {
		&self.bg_affine_matrices[index]
	}

	fn get_win_dimensions(&self, index: usize) -> &WindowDimensions {
		&self.win_dimensions[index]
	}

	fn get_win_in(&self) -> &WinIn {
		&self.win_in
	}

	fn get_win_out(&self) -> &WinOut {
		&self.win_out
	}

	fn get_mosaic(&self) -> &Mosaic {
		&self.mosaic
	}

	fn get_blend_control(&self) -> &BlendControl {
		&self.bld_cnt
	}

	fn get_blend_alpha(&self) -> &BlendAlpha {
		&self.bld_alpha
	}

	fn get_blend_brightness(&self) -> u8 {
		self.bld_y[0..=4].load_le()
	}

	pub fn render(&mut self) -> Vec<f32> {
		let mut pixels = vec![1.0; SCREEN_TOTAL_PIXELS * 3];
		if !self.get_disp_cnt().get_forced_blank() {
			if let Some(video_mode) = self.get_disp_cnt().get_bg_mode() {
				match video_mode {
					EVideoMode::Mode0 => {}
					EVideoMode::Mode1 => {}
					EVideoMode::Mode2 => {
						//						let bg3_cnt = self.get_bg3_cnt();
						//						let bg3_wraparound = bg3_cnt.get_overflow_wraparound();
						//						let bg3_tiles = match bg3_cnt.get_size() {
						//							0x0 => 16,
						//							0x1 => 32,
						//							0x2 => 64,
						//							0x3 => 128,
						//							_ => {
						//								panic!("IMPOSSIBLE!")
						//							}
						//						};
						//
						//						let bg3_x = self.get_bg3_x().get_value();
						//						let bg3_y = self.get_bg3_y().get_value();

						// Backgrounds
						//					for x in 0..240 {
						//						for y in 0..160 {
						//							// TODO: Use transform!!!
						//							let pixel_offset = (bg3_x as usize + x) + (bg3_y as usize + y * bg3_tiles);
						//							let tile = pixel_offset / 8;
						//							let tile_number = self.vram[bg3_cnt.get_map_data_address() + tile] as usize;
						//
						//							let palette_color_index = (self.vram[bg3_cnt.get_tile_data_address() + (tile_number * 64) + (pixel_offset % 8)] * 2) as usize;
						//							let color = Color::new((self.palette_ram[palette_color_index + 1] as u16) << 8 | self.palette_ram[palette_color_index] as u16);
						//
						//							let pixel_index = (y * 240 + x) * 3;
						//							pixels[pixel_index] = color.get_red();
						//							pixels[pixel_index + 1] = color.get_green();
						//							pixels[pixel_index + 2] = color.get_blue();
						//						}
						//					}

						if self.get_disp_cnt().get_screen_display_sprites() {
							let is_1d_mapping = self.get_disp_cnt().get_sprite_1d_mapping();
							// Reverse sprites for priority order (Sprite 0 = Front, Last Sprite = back)
							let sprites = self.oam.chunks_exact(8).map(|x| SpriteEntry::new(x.view_bits())).rev();
							for sprite in sprites.filter(|s| s.get_is_affine() || !s.get_is_virtual_double_sized()) {
								let (width, height) = sprite.get_size();
								let tiles_per_row = if sprite.get_is_256_palette() { 16 } else { 32 };
								let tile_length = if sprite.get_is_256_palette() { 64 } else { 32 };
								let start_tile_address = SPRITE_TILES_START_ADDRESS + sprite.get_tile_index() as usize * 32;

								let pixel_x0 = (width / 2) as i32;
								let pixel_y0 = (height / 2) as i32;

								let half_width = if sprite.get_is_virtual_double_sized() { width as i32 } else { pixel_x0 };
								let half_height = if sprite.get_is_virtual_double_sized() { height as i32 } else { pixel_y0 };

								for y in -half_height..half_height {
									for x in -half_width..half_width {
										let pixel_x;
										let pixel_y;
										if sprite.get_is_affine() {
											let affine_matrix_starting_address = sprite.get_affine_matrix_index() * 32;
											let pa = FixedPoint16Bit::with_value(
												self.oam[affine_matrix_starting_address + 0x6..=affine_matrix_starting_address + 0x7]
													.view_bits::<Lsb0>()
													.load_le(),
											)
											.get_value();
											let pb = FixedPoint16Bit::with_value(
												self.oam[affine_matrix_starting_address + 0xe..=affine_matrix_starting_address + 0xf]
													.view_bits::<Lsb0>()
													.load_le(),
											)
											.get_value();
											let pc = FixedPoint16Bit::with_value(
												self.oam[affine_matrix_starting_address + 0x16..=affine_matrix_starting_address + 0x17]
													.view_bits::<Lsb0>()
													.load_le(),
											)
											.get_value();
											let pd = FixedPoint16Bit::with_value(
												self.oam[affine_matrix_starting_address + 0x1e..=affine_matrix_starting_address + 0x1f]
													.view_bits::<Lsb0>()
													.load_le(),
											)
											.get_value();

											pixel_x = pixel_x0 + ((pa * x + pb * y) >> 8);
											pixel_y = pixel_y0 + ((pc * x + pd * y) >> 8);
										} else {
											pixel_x = pixel_x0 + x;
											pixel_y = pixel_y0 + y;
										}

										// NOTE: These values wrap around
										let screen_x = sprite.get_x_coord() + half_width + x;
										let screen_y = sprite.get_y_coord() + half_height + y;

										// Y has range -127/127 (within 160 vertical screen size)
										if screen_x >= 0
											&& screen_y >= 0 && screen_x < 240 && screen_y < 160
											&& pixel_x >= 0 && pixel_x < width as i32 && pixel_y >= 0
											&& pixel_y < height as i32
										{
											let pixel_index = (screen_x as usize + (screen_y as usize * 240)) * 3;

											let tx = pixel_x as usize / 8;
											let ty = pixel_y as usize / 8;
											let tile_address = if is_1d_mapping {
												let tile = tx + ty * width / 8;
												start_tile_address + tile * tile_length
											} else {
												let tile = tx + ty * tiles_per_row;
												start_tile_address + tile * tile_length
											};

											let tile_pixel = ((pixel_x % 8) + (pixel_y % 8) * 8) as usize;
											let palette_entry = (self.vram[tile_address + tile_pixel]) as u32;

											if palette_entry != 0 {
												let color;
												if sprite.get_is_256_palette() {
													color = Color::new(self.read_16(PALETTE_RAM_ADDR as u32 + SPRITE_PALETTE_START_ADDRESS + palette_entry * 2));
												} else {
													let palette_offset = sprite.get_palette_number() as u32 * 16;
													let color_address = PALETTE_RAM_ADDR as u32 + SPRITE_PALETTE_START_ADDRESS + (palette_offset + palette_entry) * 2;
													color = Color::new(self.read_16(color_address));
												}

												pixels[pixel_index] = color.get_red();
												pixels[pixel_index + 1] = color.get_green();
												pixels[pixel_index + 2] = color.get_blue();
											}
										}
									}
								}
							}
						}
					}
					EVideoMode::Mode3 => {}
					EVideoMode::Mode4 => {
						let starting_address = if self.get_disp_cnt().get_display_frame_1() { 0xA000 } else { 0x0 };

						for y in 0..160 {
							for x in 0..240 {
								let bitmap_index = (x as usize + (y as usize * 240));
								let pixel_index = bitmap_index * 3;
								let palette_entry = (self.vram[starting_address + bitmap_index]) as u32;

								let color = Color::new(self.read_16(PALETTE_RAM_ADDR as u32 + palette_entry * 2));

								pixels[pixel_index] = color.get_red();
								pixels[pixel_index + 1] = color.get_green();
								pixels[pixel_index + 2] = color.get_blue();
							}
						}
					}
					EVideoMode::Mode5 => {}
				}
			}
		}

		pixels
	}
}

pub struct DisplayControl {
	data: Gba16BitRegister,
}

impl DisplayControl {
	pub fn new() -> Self {
		Self { data: Gba16BitRegister::zeroed() }
	}

	pub fn get_bg_mode(&self) -> Option<EVideoMode> {
		FromPrimitive::from_u8(self.data[0..=2].load_le())
	}

	pub fn get_display_frame_1(&self) -> bool {
		self.data[4]
	}

	pub fn get_hblank_interval_free(&self) -> bool {
		self.data[5]
	}

	pub fn get_sprite_1d_mapping(&self) -> bool {
		self.data[6]
	}

	pub fn get_forced_blank(&self) -> bool {
		self.data[7]
	}

	pub fn get_screen_display_bg0(&self) -> bool {
		self.data[8]
	}

	pub fn get_screen_display_bg1(&self) -> bool {
		self.data[9]
	}

	pub fn get_screen_display_bg2(&self) -> bool {
		self.data[10]
	}

	pub fn get_screen_display_bg3(&self) -> bool {
		self.data[11]
	}

	pub fn get_screen_display_sprites(&self) -> bool {
		self.data[12]
	}

	pub fn get_window0_display(&self) -> bool {
		self.data[13]
	}

	pub fn get_window1_display(&self) -> bool {
		self.data[14]
	}

	pub fn get_sprite_window_display(&self) -> bool {
		self.data[15]
	}
}

impl IORegister<u16> for DisplayControl {
	fn read(&self) -> u16 {
		self.data.load_le()
	}

	fn write(&mut self, value: u16) {
		self.data.store_le(value)
	}
}

pub struct DisplayStatus(Gba16BitRegister);

impl DisplayStatus {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
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

	pub fn get_v_counter_flag(&self) -> bool {
		self.0[2]
	}

	pub fn set_v_counter_flag(&mut self, value: bool) {
		self.0.set(2, value);
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

struct BackgroundControl(Gba16BitRegister);

impl BackgroundControl {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
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

pub struct BackgroundAffineMatrix {
	pa: FixedPoint16Bit,
	pb: FixedPoint16Bit,
	pc: FixedPoint16Bit,
	pd: FixedPoint16Bit,
	x: FixedPoint28Bit,
	y: FixedPoint28Bit,
}

impl BackgroundAffineMatrix {
	pub fn new() -> Self {
		Self {
			pa: FixedPoint16Bit::new(),
			pb: FixedPoint16Bit::new(),
			pc: FixedPoint16Bit::new(),
			pd: FixedPoint16Bit::new(),
			x: FixedPoint28Bit::new(),
			y: FixedPoint28Bit::new(),
		}
	}

	pub fn get_pa(&self) -> &FixedPoint16Bit {
		&self.pa
	}

	pub fn get_pb(&self) -> &FixedPoint16Bit {
		&self.pb
	}

	pub fn get_pc(&self) -> &FixedPoint16Bit {
		&self.pc
	}

	pub fn get_pd(&self) -> &FixedPoint16Bit {
		&self.pd
	}

	pub fn get_x(&self) -> &FixedPoint28Bit {
		&self.x
	}

	pub fn get_y(&self) -> &FixedPoint28Bit {
		&self.y
	}
}

struct FixedPoint16Bit(Gba16BitRegister);

impl FixedPoint16Bit {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
	}

	pub fn with_value(value: u16) -> Self {
		Self {
			0: Gba16BitRegister::new([value; 1]),
		}
	}

	pub fn get_fractional(&self) -> u8 {
		self.0[0..=7].load_le()
	}

	pub fn get_integer(&self) -> i32 {
		self.0[8..=0xf].load_le::<u8>() as i8 as i32
	}

	pub fn get_value(&self) -> i32 {
		self.0.load_le::<u16>() as i16 as i32
	}
}

struct FixedPoint28Bit(Gba32BitRegister);

impl FixedPoint28Bit {
	pub fn new() -> Self {
		Self { 0: Gba32BitRegister::zeroed() }
	}

	pub fn with_value(value: u32) -> Self {
		Self {
			0: Gba32BitRegister::new([value; 1]),
		}
	}

	pub fn get_fractional(&self) -> u8 {
		self.0[0..=7].load_le()
	}

	pub fn get_integer(&self) -> i32 {
		sign_extend(self.0[8..=27].load_le::<u32>(), 20)
	}

	pub fn get_value(&self) -> i32 {
		sign_extend(self.0[0..=27].load_le::<u32>(), 28)
	}
}

struct WinIn(Gba16BitRegister);

impl WinIn {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
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

struct WinOut(Gba16BitRegister);

impl WinOut {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
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

struct Mosaic(Gba16BitRegister);

impl Mosaic {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
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

struct BlendControl(Gba16BitRegister);

impl BlendControl {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
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

struct BlendAlpha(Gba16BitRegister);

impl BlendAlpha {
	pub fn new() -> Self {
		Self { 0: Gba16BitRegister::zeroed() }
	}

	pub fn get_alpha_a(&self) -> u8 {
		self.0[0..=4].load_le()
	}

	pub fn get_alpha_b(&self) -> u8 {
		self.0[8..=12].load_le()
	}
}

impl MemoryInterface for PPU {
	fn read_8(&self, address: u32) -> u8 {
		match address & 0xff00_0000 {
			crate::system::IO_ADDR => {
				let addr = (address & PPU_REGISTERS_END);
				let shift = (addr & 0x1) * 8;
				match addr {
					DISP_CNT_ADDRESS => (self.disp_cnt.read() >> shift) as u8,
					DISP_STAT_ADDRESS => 0,
					VCOUNT_ADDRESS => 0,
					BG0_CNT_ADDRESS => 0,
					BG1_CNT_ADDRESS => 0,
					BG2_CNT_ADDRESS => 0,
					BG3_CNT_ADDRESS => 0,
					BG0_HOFS_ADDRESS => 0,
					BG0_VOFS_ADDRESS => 0,
					BG1_HOFS_ADDRESS => 0,
					BG1_VOFS_ADDRESS => 0,
					BG2_HOFS_ADDRESS => 0,
					BG2_VOFS_ADDRESS => 0,
					BG3_HOFS_ADDRESS => 0,
					BG3_VOFS_ADDRESS => 0,
					BG2_PA_ADDRESS => 0,
					BG2_PB_ADDRESS => 0,
					BG2_PC_ADDRESS => 0,
					BG2_PD_ADDRESS => 0,
					BG2_X_ADDRESS => 0,
					BG2_Y_ADDRESS => 0,
					BG3_PA_ADDRESS => 0,
					BG3_PB_ADDRESS => 0,
					BG3_PC_ADDRESS => 0,
					BG3_PD_ADDRESS => 0,
					BG3_X_ADDRESS => 0,
					BG3_Y_ADDRESS => 0,
					WIN0_H_ADDRESS => 0,
					WIN1_H_ADDRESS => 0,
					WIN0_V_ADDRESS => 0,
					WIN1_V_ADDRESS => 0,
					WIN_IN_ADDRESS => 0,
					WIN_OUT_ADDRESS => 0,
					MOSAIC_ADDRESS => 0,
					BLD_CNT_ADDRESS => 0,
					BLD_ALPHA_ADDRESS => 0,
					BLD_Y_ADDRESS => 0,
					_ => 0x0,
				}
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
				let addr = (address & PPU_REGISTERS_END);
				let shift = (addr & 0x1) * 8;
				match addr & !0x1 {
					DISP_CNT_ADDRESS => self.disp_cnt.write((value as u16) << shift),
					DISP_STAT_ADDRESS => {}
					VCOUNT_ADDRESS => {}
					BG0_CNT_ADDRESS => {}
					BG1_CNT_ADDRESS => {}
					BG2_CNT_ADDRESS => {}
					BG3_CNT_ADDRESS => {}
					BG0_HOFS_ADDRESS => {}
					BG0_VOFS_ADDRESS => {}
					BG1_HOFS_ADDRESS => {}
					BG1_VOFS_ADDRESS => {}
					BG2_HOFS_ADDRESS => {}
					BG2_VOFS_ADDRESS => {}
					BG3_HOFS_ADDRESS => {}
					BG3_VOFS_ADDRESS => {}
					BG2_PA_ADDRESS => {}
					BG2_PB_ADDRESS => {}
					BG2_PC_ADDRESS => {}
					BG2_PD_ADDRESS => {}
					BG2_X_ADDRESS => self.bg_affine_matrices[0].x.0.store_le::<u32>((value as u32) << shift),
					BG2_Y_ADDRESS => {}
					BG3_PA_ADDRESS => {}
					BG3_PB_ADDRESS => {}
					BG3_PC_ADDRESS => {}
					BG3_PD_ADDRESS => {}
					BG3_X_ADDRESS => {}
					BG3_Y_ADDRESS => {}
					WIN0_H_ADDRESS => {}
					WIN1_H_ADDRESS => {}
					WIN0_V_ADDRESS => {}
					WIN1_V_ADDRESS => {}
					WIN_IN_ADDRESS => {}
					WIN_OUT_ADDRESS => {}
					MOSAIC_ADDRESS => {}
					BLD_CNT_ADDRESS => {}
					BLD_ALPHA_ADDRESS => {}
					BLD_Y_ADDRESS => {}
					_ => {}
				}
			}
			PALETTE_RAM_ADDR => unsafe {
				*(self.palette_ram.as_ptr().add(((address & 0x3ff) as usize) & !0x1) as *mut u16) = (value as u16) * 0x101;
			},
			VRAM_ADDR => {
				// NOTE: Writes to BG (6000000h-600FFFFh) (or 6000000h-6013FFFh in Bitmap mode) and to Palette (5000000h-50003FFh) are writing the new 8bit value to BOTH upper and lower 8bits of the addressed halfword, ie. "[addr AND NOT 1]=data*101h"
				let end_bg_address;
				if let Some(video_mode) = self.get_disp_cnt().get_bg_mode() {
					end_bg_address = if video_mode == EVideoMode::Mode3 || video_mode == EVideoMode::Mode4 || video_mode == EVideoMode::Mode5 {
						0x0600_FFFF
					} else {
						0x0601_3FFF
					};
				} else {
					end_bg_address = 0x0600_FFFF;
				}

				if address >= 0x0600_0000 && address < end_bg_address {
					unsafe {
						*(self.vram.as_ptr().add(((address & 0x17fff) as usize) & !0x1) as *mut u16) = (value as u16) * 0x101;
					}
				}
			}
			OAM_ADDR => {} // NOTE: No 8bit write is allowed to OAM
			_ => {}
		}
	}

	fn read_16(&self, address: u32) -> u16 {
		unsafe {
			match address & 0xff00_0000 {
				crate::system::IO_ADDR => {
					let addr = (address & PPU_REGISTERS_END) as usize;
					0x0
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
					0x0
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
				}
				PALETTE_RAM_ADDR => *(self.palette_ram.as_ptr().add((address & 0x3ff) as usize) as *mut u32) = value,
				VRAM_ADDR => *(self.vram.as_ptr().add((address & 0x17fff) as usize) as *mut u32) = value,
				OAM_ADDR => *(self.oam.as_ptr().add((address & 0x3ff) as usize) as *mut u32) = value,
				_ => {}
			}
		}
	}
}

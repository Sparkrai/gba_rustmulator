use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};

use bitvec::prelude::*;
use glium;
use glium::glutin::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use glium::uniforms::{SamplerBehavior, SamplerWrapFunction};
use glium::Surface;
use imgui::*;

use arm7tdmi::cpu::*;
use system::*;

use crate::debugging::disassembling::disassemble_instruction;
use crate::debugging::{build_cpu_debug_window, build_io_registers_window, build_memory_debug_window, build_sprites_debug_window, build_tiles_debug_window};
use crate::ppu::{Color, EVideoMode, SpriteEntry, OAM_SIZE, SPRITE_PALETTE_START_ADDRESS, SPRITE_TILES_START_ADDRESS, VRAM_SIZE};
use crate::windowing::System;

mod arm7tdmi;
mod debugging;
mod ppu;
mod system;
mod windowing;

fn main() {
	let system = windowing::init("GBA Rustmulator");

	let mut cpu = CPU::new();
	// Start in System mode
	cpu.get_mut_cpsr().set_mode_bits(0x1f);

	let mut bios_data = Vec::<u8>::new();
	File::open("data/bios.gba").expect("Bios couldn't be opened!").read_to_end(&mut bios_data).unwrap();

	let mut cartridge_data = Vec::<u8>::new();
	if File::open("data/demos/sbb_aff.gba")
		.expect("Cartridge couldn't be opened!")
		.read_to_end(&mut cartridge_data)
		.is_ok()
	{
		if cartridge_data.len() < CARTRIDGE_ROM_SIZE {
			cartridge_data.resize(CARTRIDGE_ROM_SIZE - cartridge_data.len(), 0);
		}
		let mut bus = SystemBus::new_with_cartridge(bios_data.into_boxed_slice(), cartridge_data.into_boxed_slice());
		//		let mut bus = SystemBus::new(bios_data.into_boxed_slice());

		let mut show_cpu_debug_window = true;
		let mut show_memory_debug_window = true;
		let mut show_io_registers_window = true;
		let mut show_tiles_window = true;
		let mut show_sprites_window = true;
		let mut show_demo_window = false;

		let mut debug_mode = true;
		let mut execute_step = false;
		let mut breakpoint_set = false;
		let mut write_flow_to_file = false;
		let mut tiles_is_palette = false;
		let mut breakpoint_address = 0x0u32;
		let mut current_inspected_address = 0;
		let mut selected_io_register = 0;

		let System {
			event_loop,
			display,
			mut imgui,
			mut platform,
			mut renderer,
			..
		} = system;
		let mut last_frame = Instant::now();
		let target_frame_duration: Duration = Duration::from_secs_f32(1.0 / 60.0);

		let mut flow = Vec::<u8>::with_capacity(10000);
		let mut current_cycle = 0u32;

		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Poll;
			match event {
				Event::NewEvents(_) => {
					// Lock FPS
					let elapsed_time = last_frame.elapsed();
					if elapsed_time < target_frame_duration {
						spin_sleep::sleep(target_frame_duration - elapsed_time);
					}
					let duration_elapsed_for_frame = last_frame.elapsed();

					let ms_per_frame = duration_elapsed_for_frame.as_micros() as f32 / 1000.0;
					let fps = 1000.0 / ms_per_frame;
					println!("Time: {:.2} ms | {:.0} FPS", ms_per_frame, fps);

					imgui.io_mut().update_delta_time(duration_elapsed_for_frame);
					last_frame = Instant::now();
				}
				Event::MainEventsCleared => {
					// NOTE: Advance GBA by one frame
					const CYCLES_PER_FRAME: u32 = 280_896;
					if !debug_mode || execute_step {
						if execute_step {
							execute_step = false;
							current_cycle = (current_cycle + 1) % CYCLES_PER_FRAME;
							bus.ppu.step(current_cycle);

							arm7tdmi::decode(&mut cpu, &mut bus);
						} else {
							for _ in 0..=CYCLES_PER_FRAME {
								current_cycle = (current_cycle + 1) % CYCLES_PER_FRAME;
								let (h_blank_irq, v_blank_irq) = bus.ppu.step(current_cycle);

								// TODO: Check interrupts!!!
								if bus.ppu.get_disp_stat().get_v_counter_flag()
									&& bus.io_regs.get_ime() && bus.io_regs.get_ie().get_v_counter_match()
									&& bus.ppu.get_disp_stat().get_v_counter_irq()
								{
									bus.io_regs.get_mut_if().set_v_counter_match(true);
									cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									bus.io_regs.halted = false;
								}

								// H-Blank
								if h_blank_irq && bus.io_regs.get_ime() && bus.io_regs.get_ie().get_h_blank() && bus.ppu.get_disp_stat().get_h_blank_irq() {
									bus.io_regs.get_mut_if().set_h_blank(true);
									cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									bus.io_regs.halted = false;
								} else if v_blank_irq && bus.io_regs.get_ime() && bus.io_regs.get_ie().get_v_blank() && bus.ppu.get_disp_stat().get_v_blank_irq() {
									// V-Blank
									bus.io_regs.get_mut_if().set_v_blank(true);
									cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									bus.io_regs.halted = false;
								}

								if !bus.io_regs.halted {
									if write_flow_to_file {
										writeln!(&mut flow, "{:#X}: {}", cpu.get_current_pc(), disassemble_instruction(&cpu, &bus)).unwrap();
									}

									arm7tdmi::decode(&mut cpu, &mut bus);

									// NOTE: Breakpoint
									if breakpoint_set && cpu.get_current_pc() == breakpoint_address {
										debug_mode = true;

										// Write flow to file
										if write_flow_to_file {
											let mut flow_file = OpenOptions::new()
												.append(true)
												.create(true)
												.open("C:\\Users\\gbAgostPa\\Downloads\\Tests\\BIOS_Flow.txt")
												.unwrap();
											flow_file.write_all(&flow).unwrap();
											flow.clear();
										}

										break;
									}
								}
							}
						}
					}

					let gl_window = display.gl_window();
					platform.prepare_frame(imgui.io_mut(), gl_window.window()).expect("Failed to prepare frame");
					gl_window.window().request_redraw();
				}
				Event::RedrawRequested(_) => {
					let mut ui = imgui.frame();

					// NOTE: UI BEGIN!!!
					let run = true;
					ui.main_menu_bar(|| {
						ui.menu(im_str!("Debug"), true, || {
							if MenuItem::new(im_str!("CPU")).build(&ui) {
								show_cpu_debug_window = true;
							}
							if MenuItem::new(im_str!("Memory")).build(&ui) {
								show_memory_debug_window = true;
							}
							if MenuItem::new(im_str!("I/O Registers")).build(&ui) {
								show_io_registers_window = true;
							}
							if MenuItem::new(im_str!("Tiles")).build(&ui) {
								show_tiles_window = true;
							}
							if MenuItem::new(im_str!("Sprites")).build(&ui) {
								show_tiles_window = true;
							}
						});
						ui.menu(im_str!("Help"), true, || {
							if MenuItem::new(im_str!("Demo")).build(&ui) {
								show_demo_window = true;
							}
						});
					});

					// NOTE: Render window!!!
					Window::new(im_str!("Render"))
						.size([0.0, 0.0], Condition::Always)
						.resizable(true)
						.position([900.0, 600.0], Condition::FirstUseEver)
						.build(&ui, || {
							let frame_texture = bus.ppu.render();

							let image = glium::texture::RawImage2d::from_raw_rgb(frame_texture, (240, 160));
							let gl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

							let texture = imgui_glium_renderer::Texture {
								texture: Rc::new(gl_texture),
								sampler: SamplerBehavior {
									wrap_function: (SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp),
									..Default::default()
								},
							};
							let texture_id = renderer.textures().insert(texture);
							Image::new(texture_id, [480.0, 320.0]).build(&ui);
						});

					if show_cpu_debug_window {
						build_cpu_debug_window(&cpu, &&mut ui, &mut show_cpu_debug_window);
					}

					if show_memory_debug_window {
						build_memory_debug_window(
							&cpu,
							&bus,
							&mut show_memory_debug_window,
							&mut current_inspected_address,
							&mut debug_mode,
							&mut execute_step,
							&mut breakpoint_set,
							&mut write_flow_to_file,
							&mut breakpoint_address,
							&&mut ui,
						);
					}

					if show_io_registers_window {
						build_io_registers_window(&bus, &mut show_io_registers_window, &mut selected_io_register, &&mut ui);
					}

					if show_tiles_window {
						if let Some(video_mode) = bus.ppu.get_disp_cnt().get_bg_mode() {
							let obj_tiles_start = match video_mode {
								EVideoMode::Mode0 | EVideoMode::Mode1 | EVideoMode::Mode2 => 0x10000,
								EVideoMode::Mode3 | EVideoMode::Mode4 | EVideoMode::Mode5 => 0x14000,
							};

							let mut pixels = vec![0.0; VRAM_SIZE * 3];
							for i in 0..VRAM_SIZE as u32 {
								let palette_color_index = if i >= obj_tiles_start {
									bus.ppu.read_8(VRAM_ADDR + i) as u32 + 256u32
								} else {
									bus.ppu.read_8(VRAM_ADDR + i) as u32
								};
								// One color every 2 bytes
								let color = Color::new(bus.ppu.read_16(PALETTE_RAM_ADDR + (palette_color_index * 2)));

								const TILES_PER_ROW: u32 = 32;
								let tile_offset = ((i / 64) % TILES_PER_ROW) * 8;
								let row_offset = ((i % 64) / 8) * TILES_PER_ROW * 8;
								let tiles_row_offset = ((i / 64) / TILES_PER_ROW) * 64 * TILES_PER_ROW;
								let pixel_index = ((i % 8) + tile_offset + tiles_row_offset + row_offset) * 3;

								pixels[pixel_index as usize] = color.get_red();
								pixels[pixel_index as usize + 1] = color.get_green();
								pixels[pixel_index as usize + 2] = color.get_blue();
							}

							let image = glium::texture::RawImage2d::from_raw_rgb(pixels, (256, 384));
							let gl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

							let texture = imgui_glium_renderer::Texture {
								texture: Rc::new(gl_texture),
								sampler: SamplerBehavior {
									wrap_function: (SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp),
									..Default::default()
								},
							};
							let texture_id = renderer.textures().insert(texture);

							build_tiles_debug_window(&bus, &mut show_tiles_window, &mut tiles_is_palette, texture_id, &&mut ui);
						}
					}

					if show_sprites_window {
						if let Some(video_mode) = bus.ppu.get_disp_cnt().get_bg_mode() {
							let sprite_tiles_start = match video_mode {
								EVideoMode::Mode0 | EVideoMode::Mode1 | EVideoMode::Mode2 => SPRITE_TILES_START_ADDRESS,
								EVideoMode::Mode3 | EVideoMode::Mode4 | EVideoMode::Mode5 => 0x14000,
							};

							let is_1d_mapping = bus.ppu.get_disp_cnt().get_sprite_1d_mapping();
							let sprite_palette_start_address = PALETTE_RAM_ADDR + SPRITE_PALETTE_START_ADDRESS as u32;
							let mut texture_ids = Vec::<TextureId>::with_capacity(128);
							for i in (0..OAM_SIZE as u32).step_by(8) {
								let data = [
									bus.ppu.read_8(OAM_ADDR + i),
									bus.ppu.read_8(OAM_ADDR + i + 1),
									bus.ppu.read_8(OAM_ADDR + i + 2),
									bus.ppu.read_8(OAM_ADDR + i + 3),
									bus.ppu.read_8(OAM_ADDR + i + 4),
									bus.ppu.read_8(OAM_ADDR + i + 5),
									bus.ppu.read_8(OAM_ADDR + i + 6),
									bus.ppu.read_8(OAM_ADDR + i + 7),
								];
								let sprite = SpriteEntry::new(data.view_bits::<Lsb0>());

								let (width, height) = sprite.get_size();
								let tiles_per_row = if sprite.get_is_256_palette() { 16 } else { 32 };
								let tile_length = if sprite.get_is_256_palette() { 64 } else { 32 };
								let start_tile_address = sprite_tiles_start + sprite.get_tile_index() * 32;

								let mut pixels = vec![0.0; width * height * 3];
								let tiles_x = width / 8;
								for tx in 0..tiles_x {
									for ty in 0..height / 8 {
										let tile_address = if is_1d_mapping {
											let tile = tx + ty * tiles_x;
											start_tile_address + tile * tile_length
										} else {
											let tile = tx + ty * tiles_per_row;
											start_tile_address + tile * tile_length
										};

										for x in 0..8 {
											for y in 0..8 {
												let tile_pixel = x + y * 8;
												let palette_entry = bus.ppu.read_8(VRAM_ADDR + tile_address as u32 + tile_pixel) as u32;

												let pixel_index = (tx * 8 + ty * 64 * tiles_x + (x + y * width as u32) as usize) * 3;

												let color;
												if sprite.get_is_256_palette() {
													color = Color::new(bus.ppu.read_16(sprite_palette_start_address + palette_entry * 2));
												} else {
													let palette_offset = sprite.get_palette_number() as u32 * 16;
													let color_address = (palette_offset + palette_entry) * 2;
													color = Color::new(bus.ppu.read_16(sprite_palette_start_address + color_address));
												}

												pixels[pixel_index] = color.get_red();
												pixels[pixel_index + 1] = color.get_green();
												pixels[pixel_index + 2] = color.get_blue();
											}
										}
									}
								}

								let image = glium::texture::RawImage2d::from_raw_rgb(pixels, (width as u32, height as u32));
								let gl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

								let texture = imgui_glium_renderer::Texture {
									texture: Rc::new(gl_texture),
									sampler: SamplerBehavior {
										wrap_function: (SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp),
										..Default::default()
									},
								};
								let texture_id = renderer.textures().insert(texture);

								texture_ids.push(texture_id);
							}

							build_sprites_debug_window(&mut show_sprites_window, &texture_ids, &&mut ui);
						}
					}

					if show_demo_window {
						ui.show_demo_window(&mut show_demo_window);
					}
					// NOTE: UI END!!!

					if !run {
						*control_flow = ControlFlow::Exit;
					}

					let gl_window = display.gl_window();
					let mut target = display.draw();
					target.clear_color_srgb(0.2, 0.2, 0.2, 1.0);
					platform.prepare_render(&ui, gl_window.window());
					let draw_data = ui.render();
					renderer.render(&mut target, draw_data).expect("Rendering failed");
					target.finish().expect("Failed to swap buffers");
				}
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => *control_flow = ControlFlow::Exit,
				Event::WindowEvent {
					event: WindowEvent::KeyboardInput { input, .. },
					..
				} => {
					if !imgui.io().want_capture_keyboard {
						let released = input.state == ElementState::Released;
						if let Some(key_code) = input.virtual_keycode {
							match key_code {
								VirtualKeyCode::A => bus.io_regs.get_mut_key_input().set_button_a(released),
								VirtualKeyCode::S => bus.io_regs.get_mut_key_input().set_button_b(released),
								VirtualKeyCode::Z => bus.io_regs.get_mut_key_input().set_select(released),
								VirtualKeyCode::X => bus.io_regs.get_mut_key_input().set_start(released),
								VirtualKeyCode::Right => bus.io_regs.get_mut_key_input().set_right(released),
								VirtualKeyCode::Left => bus.io_regs.get_mut_key_input().set_left(released),
								VirtualKeyCode::Up => bus.io_regs.get_mut_key_input().set_up(released),
								VirtualKeyCode::Down => bus.io_regs.get_mut_key_input().set_down(released),
								VirtualKeyCode::LShift => bus.io_regs.get_mut_key_input().set_button_r(released),
								VirtualKeyCode::Space => bus.io_regs.get_mut_key_input().set_button_l(released),
								_ => {}
							}
						}
					}
				}
				_ => {
					let gl_window = display.gl_window();
					platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
				}
			}
		});
	} else {
		println!("Cartridge couldn't be read!");
	}
}

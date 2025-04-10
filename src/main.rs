use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use glium;
use imgui::*;

use arm7tdmi::cpu::*;
use system::*;

use crate::debugging::disassembling::disassemble_instruction;
use crate::debugging::{build_cpu_debug_window, build_io_registers_window, build_memory_debug_window, build_tiles_debug_window};
use crate::ppu::{Color, PALETTE_RAM_SIZE, VRAM_SIZE};
use crate::windowing::System;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use glium::uniforms::{SamplerBehavior, SamplerWrapFunction};
use glium::Surface;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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
	let mut bus = SystemBus::new(bios_data.into_boxed_slice());

	let mut cartridge_data = Vec::<u8>::new();
	if File::open("data/demos/hello.gba")
		.expect("Cartridge couldn't be opened!")
		.read_to_end(&mut cartridge_data)
		.is_ok()
	{
		let mut show_cpu_debug_window = true;
		let mut show_memory_debug_window = true;
		let mut show_io_registers_window = true;
		let mut show_tiles_window = true;
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
					if last_frame.elapsed() < target_frame_duration {
						spin_sleep::sleep(target_frame_duration - last_frame.elapsed());
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
							bus.ppu.set_vcount((current_cycle / 1232) as u8);

							arm7tdmi::decode(&mut cpu, &mut bus);
						} else {
							for _ in 0..=CYCLES_PER_FRAME {
								current_cycle = (current_cycle + 1) % CYCLES_PER_FRAME;
								bus.ppu.set_vcount((current_cycle / 1232) as u8);

								if bus.io_regs.get_ime() {
									// H-Blank Interrupt
									if bus.io_regs.get_ie().get_h_blank() && bus.ppu.get_disp_stat().get_h_blank_irq() && (current_cycle.wrapping_sub(960) % 1232 == 0) {
										bus.io_regs.get_if().set_h_blank(true);
										//										bus.ppu.get_disp_stat().set_h_blank(true);
										cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									} else if bus.io_regs.get_ie().get_v_blank() && bus.ppu.get_disp_stat().get_v_blank_irq() && current_cycle == 197120 {
										// V-Blank Interrupt
										bus.io_regs.get_if().set_v_blank(true);
										//										bus.ppu.get_disp_stat().set_v_blank(true);
										cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									}
								}

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

					let gl_window = display.gl_window();
					platform.prepare_frame(imgui.io_mut(), gl_window.window()).expect("Failed to prepare frame");
					gl_window.window().request_redraw();
				}
				Event::RedrawRequested(_) => {
					let mut ui = imgui.frame();

					// NOTE: UI BEGIN!!!
					let mut run = true;
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
						.resizable(false)
						.position([900.0, 600.0], Condition::FirstUseEver)
						.build(&ui, || {
							let frame_texture = bus.ppu.render();

							let mut image = glium::texture::RawImage2d::from_raw_rgb(frame_texture, (240, 160));
							let gl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

							let texture = imgui_glium_renderer::Texture {
								texture: Rc::new(gl_texture),
								sampler: SamplerBehavior {
									wrap_function: (SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp, SamplerWrapFunction::BorderClamp),
									..Default::default()
								},
							};
							let texture_id = renderer.textures().insert(texture);
							Image::new(texture_id, [240.0, 160.0]).build(&ui);
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
						let mut pixels = vec![0u8; VRAM_SIZE * 3];
						for i in 0..VRAM_SIZE as u32 {
							let palette_color_index = bus.ppu.read_8(VRAM_ADDR + i) as u32;
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

						let mut image = glium::texture::RawImage2d::from_raw_rgb(pixels, (256, 384));
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
				event => {
					let gl_window = display.gl_window();
					platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
				}
			}
		});
	} else {
		println!("Cartridge couldn't be read!");
	}
}

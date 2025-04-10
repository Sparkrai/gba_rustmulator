use std::fs::File;
use std::io::Read;

use glium;
use imgui::*;

use arm7tdmi::cpu::*;
use system::*;

use crate::debugging::{build_cpu_debug_window, build_memory_debug_window};
use crate::windowing::System;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use glium::uniforms::SamplerBehavior;
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
		let mut show_demo_window = false;

		let mut debug_mode = true;
		let mut execute_step = false;
		let mut breakpoint_set = false;
		let mut breakpoint_address = 0x0u32;
		let mut current_inspected_address = 0;

		//		let debug_mode = Arc::new(RwLock::new(true));
		//		let execute_step = Arc::new(RwLock::new(false));
		//		let breakpoint_set = Arc::new(RwLock::new(false));
		//		let breakpoint_address = Arc::new(RwLock::new(0x0u32));
		//		let mut current_inspected_address = 0;
		//
		//		let cpu = Arc::new(RwLock::new(cpu_raw));
		//		let bus = Arc::new(RwLock::new(bus_raw));

		//		let main_cpu = cpu.clone();
		//		let main_bus = bus.clone();
		//		let main_debug_mode = debug_mode.clone();
		//		let main_execute_step = execute_step.clone();
		//		let main_breakpoint_set = breakpoint_set.clone();
		//		let main_breakpoint_address = breakpoint_address.clone();
		//		std::thread::spawn(move || loop {
		//			let debug_mode_read = main_debug_mode.read().unwrap();
		//			let execute_step_read = main_execute_step.read().unwrap();
		//			if !*debug_mode_read || *execute_step_read {
		//				drop(debug_mode_read);
		//
		//				if *execute_step_read {
		//					drop(execute_step_read);
		//					*main_execute_step.write().unwrap() = false;
		//				} else {
		//					drop(execute_step_read);
		//				}
		//
		//				let mut cpu_write = main_cpu.write().unwrap();
		//				let mut bus_write = main_bus.write().unwrap();
		//				arm7tdmi::decode(&mut cpu_write, &mut bus_write);
		//
		//				let breakpoint_set_read = main_breakpoint_set.read().unwrap();
		//				if *breakpoint_set_read {
		//					let breakpoint_address_read = main_breakpoint_address.read().unwrap();
		//					if cpu_write.get_current_pc() == *breakpoint_address_read {
		//						*main_debug_mode.write().unwrap() = true;
		//					}
		//				}
		//			}
		//		});

		//		let system_cpu = cpu.clone();
		//		let system_bus = bus.clone();
		//		let system_debug_mode = debug_mode.clone();
		//		let system_execute_step = execute_step.clone();
		//		let system_breakpoint_address = breakpoint_address.clone();
		//		let system_breakpoint_set = breakpoint_set.clone();
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
					if !debug_mode || execute_step {
						if execute_step {
							execute_step = false;
							arm7tdmi::decode(&mut cpu, &mut bus);
						} else {
							const CYCLES_PER_FRAME: u32 = 280_896;
							for cycle in 0..=CYCLES_PER_FRAME {
								if bus.io_regs.get_ime() {
									// H-Blank Interrupt
									if bus.io_regs.get_ie().get_h_blank() && bus.ppu.get_disp_stat().get_h_blank_irq() && (cycle.wrapping_sub(960) % 1232 == 0) {
										bus.io_regs.get_if().set_h_blank(true);
										//										bus.ppu.get_disp_stat().set_h_blank(true);
										cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									} else if bus.io_regs.get_ie().get_v_blank() && bus.ppu.get_disp_stat().get_v_blank_irq() && cycle == 197120 {
										// V-Blank Interrupt
										bus.io_regs.get_if().set_v_blank(true);
										//										bus.ppu.get_disp_stat().set_v_blank(true);
										cpu.exception(crate::arm7tdmi::EExceptionType::Irq);
									}
								}

								arm7tdmi::decode(&mut cpu, &mut bus);

								if breakpoint_set {
									if cpu.get_current_pc() == breakpoint_address {
										debug_mode = true;
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
					let mut run = true;
					ui.main_menu_bar(|| {
						ui.menu(im_str!("Debug"), true, || {
							if MenuItem::new(im_str!("CPU")).build(&ui) {
								show_cpu_debug_window = true;
							}
							if MenuItem::new(im_str!("Memory")).build(&ui) {
								show_memory_debug_window = true;
							}
						});
						ui.menu(im_str!("Help"), true, || {
							if MenuItem::new(im_str!("Demo")).build(&ui) {
								show_demo_window = true;
							}
						});
					});

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
								sampler: SamplerBehavior { ..Default::default() },
							};
							let gl_texture_pointer = texture.texture.clone();
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
							&mut breakpoint_address,
							&&mut ui,
						);
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
				_ => {}
			}
		});
	} else {
		println!("Cartridge couldn't be read!");
	}
}

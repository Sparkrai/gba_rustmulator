use std::fs::File;
use std::io::Read;

use imgui::*;

use arm7tdmi::cpu::*;
use memory::*;

use crate::debugging::{build_cpu_debug_window, build_memory_debug_window};
use std::sync::{Arc, RwLock};
use std::time::Duration;

mod arm7tdmi;
mod debugging;
mod memory;
mod windowing;

fn main() {
	let system = windowing::init("GBA Rustmulator");

	let mut cpu_raw = CPU::new();
	// Start in System mode
	cpu_raw.get_mut_cpsr().set_mode_bits(0x1f);

	let mut bus_raw = MemoryBus::new();

	let mut bios_data = Vec::<u8>::new();
	File::open("data/bios.gba").expect("Bios couldn't be opened!").read_to_end(&mut bios_data).unwrap();
	bus_raw.load_bios(&bios_data);

	let mut cartridge_data = Vec::<u8>::new();
	if File::open("data/demos/hello.gba")
		.expect("Cartridge couldn't be opened!")
		.read_to_end(&mut cartridge_data)
		.is_ok()
	{
		bus_raw.load_cartridge(&cartridge_data);

		let mut show_cpu_debug_window = true;
		let mut show_memory_debug_window = true;
		let mut show_demo_window = false;

		let mut debug_mode = Arc::new(RwLock::new(true));
		let mut execute_step = Arc::new(RwLock::new(false));
		let mut breakpoint_set = false;
		let mut breakpoint_address = Arc::new(RwLock::new(0x0u32));
		let mut current_inspected_address = 0;

		let mut cpu = Arc::new(RwLock::new(cpu_raw));
		let mut bus = Arc::new(RwLock::new(bus_raw));

		let main_cpu = cpu.clone();
		let main_bus = bus.clone();
		let main_debug_mode = debug_mode.clone();
		let main_execute_step = execute_step.clone();
		let main_breakpoint_address = breakpoint_address.clone();
		std::thread::spawn(move || loop {
			let debug_mode_read = main_debug_mode.read().unwrap();
			let execute_step_read = main_execute_step.read().unwrap();
			if !*debug_mode_read || *execute_step_read {
				drop(debug_mode_read);

				if *execute_step_read {
					drop(execute_step_read);
					*main_execute_step.write().unwrap() = false;
				} else {
					drop(execute_step_read);
				}

				let mut cpu_write = main_cpu.write().unwrap();
				let mut bus_write = main_bus.write().unwrap();
				arm7tdmi::decode(&mut cpu_write, &mut bus_write);

				let breakpoint_address_read = main_breakpoint_address.read().unwrap();
				if cpu_write.get_current_pc() == *breakpoint_address_read {
					*main_debug_mode.write().unwrap() = true;
				}
			}
		});

		let system_cpu = cpu.clone();
		let system_bus = bus.clone();
		let system_debug_mode = debug_mode.clone();
		let system_execute_step = execute_step.clone();
		let system_breakpoint_address = breakpoint_address.clone();
		system.main_loop(move |_exit, ui| {
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

			if show_cpu_debug_window {
				let cpu_read = system_cpu.read().unwrap();
				build_cpu_debug_window(&cpu_read, &ui, &mut show_cpu_debug_window);
			}

			if show_memory_debug_window {
				let debug_mode_read = system_debug_mode.read().unwrap();
				let execute_step_read = system_execute_step.read().unwrap();
				let breakpoint_address_read = system_breakpoint_address.read().unwrap();
				let mut new_debug_mode = *debug_mode_read;
				let mut new_execute_step = *execute_step_read;
				let mut new_breakpoint_address = *breakpoint_address_read;

				let cpu_read = system_cpu.read().unwrap();
				let bus_read = system_bus.read().unwrap();
				build_memory_debug_window(
					&cpu_read,
					&bus_read,
					&mut show_memory_debug_window,
					&mut current_inspected_address,
					&mut new_debug_mode,
					&mut new_execute_step,
					&mut breakpoint_set,
					&mut new_breakpoint_address,
					&ui,
				);

				if new_debug_mode != *debug_mode_read {
					drop(debug_mode_read);
					*system_debug_mode.write().unwrap() = new_debug_mode;
				}

				if new_execute_step != *execute_step_read {
					drop(execute_step_read);
					*system_execute_step.write().unwrap() = new_execute_step;
				}

				if new_breakpoint_address != *breakpoint_address_read {
					drop(breakpoint_address_read);
					*system_breakpoint_address.write().unwrap() = new_breakpoint_address;
				}
			}

			if show_demo_window {
				ui.show_demo_window(&mut show_demo_window);
			}
		});
	} else {
		println!("Cartridge couldn't be read!");
	}
}

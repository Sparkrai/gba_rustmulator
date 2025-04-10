use std::fs::File;
use std::io::Read;

use bitvec::prelude::*;
use imgui::*;

use arm7tdmi::cpu::*;
use arm7tdmi::EOperatingMode;
use memory::*;

use crate::debugging::{build_cpu_debug_window, build_memory_debug_window};

mod memory;
mod windowing;
mod debugging;
mod arm7tdmi;

fn main() {
	let system = windowing::init("GBA Rustmulator");

	let mut cpu = CPU::new();
	let mut bus = MemoryBus::new();

	// Start in System mode
	cpu.get_mut_cpsr().set_mode_bits(0x1f);

	let mut bios_data = Vec::<u8>::new();
	File::open("data/bios.gba").expect("Bios couldn't be opened!").read_to_end(&mut bios_data).unwrap();
	bus.load_bios(&bios_data);

	let mut cartridge_data = Vec::<u8>::new();
	if File::open("data/demos/hello.gba").expect("Cartridge couldn't be opened!").read_to_end(&mut cartridge_data).is_ok() {
		bus.load_cartridge(&cartridge_data);

		let mut show_cpu_debug_window = true;
		let mut show_memory_debug_window = true;
		let mut show_demo_window = false;

		let mut debug_mode = true;
		let mut current_inspected_address = 0;

		system.main_loop(move |_, ui| {
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
				build_cpu_debug_window(&cpu, &ui, &mut show_cpu_debug_window);
			}

			if show_memory_debug_window {
				build_memory_debug_window(&mut cpu, &mut bus, &mut show_memory_debug_window, &mut current_inspected_address, &mut debug_mode, &ui);
			}

			if show_demo_window {
				ui.show_demo_window(&mut show_demo_window);
			}

			if !debug_mode {
				arm7tdmi::decode(&mut cpu, &mut bus);
			}
		});
	} else {
		println!("Cartridge couldn't be read!");
	}
}

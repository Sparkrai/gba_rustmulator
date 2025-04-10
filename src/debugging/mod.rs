use imgui::*;

use crate::arm7tdmi::cpu::CPU;
use crate::arm7tdmi::{decode, EOperatingMode};
use crate::debugging::disassembling::{disassemble_arm, disassemble_thumb};
use crate::memory::MemoryBus;

mod disassembling;

pub fn build_memory_debug_window(cpu: &mut CPU, bus: &mut MemoryBus, show_memory_window: &mut bool, address: &mut u32, debug_mode: &mut bool, ui: &&mut Ui) {
	Window::new(im_str!("Current Memory"))
		.size([600.0, 500.0], Condition::FirstUseEver)
		.position([750.0, 100.0], Condition::FirstUseEver)
		.opened(show_memory_window)
		.build(ui, || {
			let pc_offset = if cpu.get_cpsr().get_t() { 4 } else { 8 };

			ui.text("Current instruction highlighted");
			if !*debug_mode {
				*address = cpu.get_current_pc();
			}

			if ui.button(im_str!("Step"), [0.0, 0.0]) {
				decode(cpu, bus);
				*address = cpu.get_current_pc();
			}
			ui.same_line(0.0);
			ui.checkbox(im_str!("Debug"), debug_mode);

			let mut new_address = *address as i32;
			if ui.button(im_str!("Current PC"), [0.0, 0.0]) {
				*address = cpu.get_current_pc();
			}

			ui.same_line(0.0);
			if ui.input_int(im_str!("Address"), &mut new_address).step(4).chars_hexadecimal(true).build() {
				*address = new_address as u32;
			}

			ui.separator();
			if let Some(scroll_token) = ChildWindow::new(im_str!("##ScrollingRegion")).begin(&ui) {
				ui.columns(3, im_str!("memory"), true);
				ui.set_column_width(0, 95.0);

				const ENTRIES: i32 = 300;
				let starting_address = address.saturating_sub(20);
				let mut list_clipper = ListClipper::new(ENTRIES).begin(&ui);
				while list_clipper.step() {
					for row in list_clipper.display_start()..list_clipper.display_end() {
						let address = starting_address + (row as u32 * (pc_offset / 2));

						Selectable::new(&*im_str!("{:#010X}:", address))
							.selected(address == cpu.get_current_pc())
							.span_all_columns(true)
							.build(&ui);
						ui.next_column();

						for j in 0..pc_offset / 2 {
							let value = bus.read_8(address as u32 + j);
							let color = if value == 0 { [0.5, 0.5, 0.5, 0.5] } else { [1.0, 1.0, 1.0, 1.0] };
							ui.text_colored(color, format!("{:02X}", value));
							if j != 3 {
								ui.same_line(0.0);
							}
						}

						ui.next_column();
						ui.text(if cpu.get_cpsr().get_t() {
							disassemble_thumb(bus.read_16(address as u32))
						} else {
							disassemble_arm(bus.read_32(address as u32))
						});
						ui.next_column();
						ui.separator();
					}
				}
				ui.columns(1, im_str!(""), false);
				scroll_token.end(&ui);
			}
		});
}

pub fn build_cpu_debug_window(cpu: &CPU, ui: &&mut Ui, opened: &mut bool) {
	Window::new(im_str!("CPU")).size([650.0, 600.0], Condition::FirstUseEver).opened(opened).build(ui, || {
		ui.text(im_str!("Mode: {:?}", cpu.get_operating_mode()));

		if CollapsingHeader::new(im_str!("GPRs")).default_open(true).build(&ui) {
			ui.columns(2, im_str!("User Registers"), true);
			for (i, register) in cpu.get_registers().iter().enumerate() {
				ui.text(format!("r{}:", i));
				ui.next_column();
				ui.text(format!("{:#X}", register));
				ui.next_column();
				ui.separator();
			}
			ui.columns(1, im_str!(""), false);
		}

		if CollapsingHeader::new(im_str!("CPSRs")).default_open(true).build(&ui) {
			ui.columns(9, im_str!("cpsr"), true);
			ui.next_column();
			ui.text("N");
			ui.next_column();
			ui.text("Z");
			ui.next_column();
			ui.text("C");
			ui.next_column();
			ui.text("V");
			ui.next_column();
			ui.text("I");
			ui.next_column();
			ui.text("F");
			ui.next_column();
			ui.text("T");
			ui.next_column();
			ui.text("Mode");
			ui.separator();

			let cpsr_names = ["CPSR", "SPSR_fiq", "SPSR_svc", "SPSR_abt", "SPSR_irq", "SPSR_und"];
			for (i, cpsr) in [
				cpu.get_spsr(EOperatingMode::UserMode),
				cpu.get_spsr(EOperatingMode::FiqMode),
				cpu.get_spsr(EOperatingMode::SupervisorMode),
				cpu.get_spsr(EOperatingMode::AbortMode),
				cpu.get_spsr(EOperatingMode::IrqMode),
				cpu.get_spsr(EOperatingMode::UndefinedMode),
			]
			.iter()
			.enumerate()
			{
				ui.next_column();
				ui.text(cpsr_names[i]);
				ui.next_column();
				ui.text(cpsr.get_n().to_string());
				ui.next_column();
				ui.text(cpsr.get_z().to_string());
				ui.next_column();
				ui.text(cpsr.get_c().to_string());
				ui.next_column();
				ui.text(cpsr.get_v().to_string());
				ui.next_column();
				ui.text(cpsr.get_i().to_string());
				ui.next_column();
				ui.text(cpsr.get_f().to_string());
				ui.next_column();
				ui.text(cpsr.get_t().to_string());
				ui.next_column();
				ui.text(cpsr.get_mode_bits().to_string());
				ui.separator();
			}

			ui.columns(1, im_str!(""), false);
		}
	});
}

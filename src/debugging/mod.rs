use imgui::*;

use crate::arm7tdmi::cpu::CPU;
use crate::arm7tdmi::EOperatingMode;
use crate::debugging::disassembling::{disassemble_arm, disassemble_thumb};
use crate::system::{MemoryInterface, SystemBus};
use bitvec::prelude::*;

mod disassembling;

pub fn build_memory_debug_window(
	cpu: &CPU,
	bus: &SystemBus,
	show_memory_window: &mut bool,
	address: &mut u32,
	debug_mode: &mut bool,
	execute_step: &mut bool,
	breakpoint_set: &mut bool,
	breakpoint_address: &mut u32,
	ui: &&mut Ui,
) {
	Window::new(im_str!("Current Memory"))
		.size([600.0, 500.0], Condition::FirstUseEver)
		.opened(show_memory_window)
		.position([750.0, 75.0], Condition::Always)
		.build(ui, || {
			if !*debug_mode {
				if *breakpoint_set {
					if *address == cpu.get_current_pc() {
						*debug_mode = true;
					}
				} else {
					*address = cpu.get_current_pc();
				}
			}

			let pc_offset = if cpu.get_cpsr().get_t() { 4 } else { 8 };

			ui.text("Current instruction highlighted");

			if ui.button(im_str!("Step"), [0.0, 0.0]) || ui.is_key_down(Key::DownArrow) && *debug_mode {
				*execute_step = true;
				*address = cpu.get_current_pc();
			}
			ui.same_line(0.0);
			ui.checkbox(im_str!("Debug"), debug_mode);

			let mut new_address = if *breakpoint_set { *breakpoint_address } else { *address } as i32;
			if ui.button(im_str!("Current PC"), [0.0, 0.0]) {
				*address = cpu.get_current_pc();
			}

			ui.same_line(0.0);
			if ui.input_int(im_str!("Address"), &mut new_address).step(4).chars_hexadecimal(true).build() && *debug_mode {
				if *breakpoint_set {
					*breakpoint_address = new_address as u32;
				} else {
					*address = new_address as u32;
				}
			}

			if ui.button(im_str!("Set/Unset Breakpoint"), [0.0, 0.0]) && *debug_mode {
				*breakpoint_set = !*breakpoint_set;
				*breakpoint_address = new_address as u32;
			}

			ui.separator();
			if let Some(scroll_token) = ChildWindow::new(im_str!("##ScrollingRegion")).begin(&ui) {
				ui.columns(3, im_str!("system"), true);
				ui.set_column_width(0, 95.0);

				const ENTRIES: i32 = 20;
				let starting_address = (if *breakpoint_set { cpu.get_current_pc() } else { *address }).saturating_sub((pc_offset / 2) * (ENTRIES / 2) as u32);
				let mut list_clipper = ListClipper::new(ENTRIES).begin(&ui);
				while list_clipper.step() {
					for row in list_clipper.display_start()..list_clipper.display_end() {
						let address = starting_address.saturating_add(row as u32 * (pc_offset / 2));
						if address <= u32::max_value() - (pc_offset / 2) {
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
			ui.columns(2, im_str!("Registers"), true);
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

pub fn build_io_registers_window(bus: &SystemBus, show_io_registers_window: &mut bool, selected_register: &mut usize, ui: &&mut Ui) {
	Window::new(im_str!("I/O Registers"))
		.size([400.0, 150.0], Condition::FirstUseEver)
		.opened(show_io_registers_window)
		.position([200.0, 1300.0], Condition::FirstUseEver)
		.build(ui, || {
			let registers = [
				im_str!("0x40000000: DISPCNT"),
				im_str!("0x40000004: DISPSTAT"),
				im_str!("0x40000006: VCOUNT"),
				im_str!("0x40000008: BG0CNT"),
				im_str!("0x4000000a: BG1CNT"),
				im_str!("0x4000000c: BG2CNT"),
				im_str!("0x4000000e: BG3CNT"),
				im_str!("0x40000010: BG0HOFS"),
				im_str!("0x40000012: BG0VOFS"),
				im_str!("0x40000014: BG1HOFS"),
				im_str!("0x40000016: BG1VOFS"),
				im_str!("0x40000018: BG2HOFS"),
				im_str!("0x4000001a: BG2VOFS"),
				im_str!("0x4000001c: BG3HOFS"),
				im_str!("0x4000001e: BG3VOFS"),
				im_str!("0x40000020: BG2PA"),
				im_str!("0x40000022: BG2PB"),
				im_str!("0x40000024: BG2PC"),
				im_str!("0x40000026: BG2PD"),
				im_str!("0x40000028: BG2X"),
				im_str!("0x4000002c: BG2Y"),
				im_str!("0x40000030: BG3PA"),
				im_str!("0x40000032: BG3PB"),
				im_str!("0x40000034: BG3PC"),
				im_str!("0x40000036: BG3PD"),
				im_str!("0x40000038: BG3X"),
				im_str!("0x4000003c: BG3Y"),
				im_str!("0x40000040: WIN0H"),
				im_str!("0x40000042: WIN1H"),
				im_str!("0x40000044: WIN0V"),
				im_str!("0x40000046: WIN1V"),
				im_str!("0x40000048: WININ"),
				im_str!("0x4000004a: WINOUT"),
				im_str!("0x4000004c: MOSAIC"),
				im_str!("0x40000050: BLDCNT"),
				im_str!("0x40000052: BLDALPHA"),
				im_str!("0x40000054: BLDY"),
				im_str!("0x40000088: SOUNDBIAS"),
				im_str!("0x40000200: IE"),
				im_str!("0x40000202: IF"),
				im_str!("0x40000208: IME"),
			];

			let register_addresses = [
				0x0400_0000,
				0x0400_0004,
				0x0400_0006,
				0x0400_0008,
				0x0400_000a,
				0x0400_000c,
				0x0400_000e,
				0x0400_0010,
				0x0400_0012,
				0x0400_0014,
				0x0400_0016,
				0x0400_0018,
				0x0400_001a,
				0x0400_001c,
				0x0400_001e,
				0x0400_0020,
				0x0400_0022,
				0x0400_0024,
				0x0400_0026,
				0x0400_0028,
				0x0400_002c,
				0x0400_0030,
				0x0400_0032,
				0x0400_0034,
				0x0400_0036,
				0x0400_0038,
				0x0400_003c,
				0x0400_0040,
				0x0400_0042,
				0x0400_0044,
				0x0400_0046,
				0x0400_0048,
				0x0400_004a,
				0x0400_004c,
				0x0400_0050,
				0x0400_0052,
				0x0400_0054,
				0x0400_0088,
				0x0400_0200,
				0x0400_0202,
				0x0400_0208,
			];

			ComboBox::new(im_str!("")).build_simple_string(ui, selected_register, &registers);

			let selected_register_address = register_addresses[*selected_register as usize];
			let register_value = bus.read_16(selected_register_address);
			ui.text(im_str!("{}", register_value));

			ui.columns(16, im_str!("Bits"), true);
			for bit in register_value.view_bits::<Lsb0>() {
				let mut bit_value = *bit;
				ui.checkbox(&*im_str!(""), &mut bit_value);
				ui.next_column();
			}

			ui.separator();

			for i in 0..16 {
				ui.text(im_str!("{}", i));
				ui.next_column();
			}
		});
}

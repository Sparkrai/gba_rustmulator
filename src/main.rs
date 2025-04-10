use std::fs::File;
use std::io::Read;

use bitvec::prelude::*;
use imgui::*;
use num_derive::*;

use cpu::*;
use memory::*;
use num_traits::FromPrimitive;

mod cpu;
mod memory;
mod windowing;

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum EShiftType {
	LSL,
	LSR,
	ASR,
	ROR,
}

fn main() {
	let system = windowing::init("GBA Rustmulator");

	let mut cpu = CPU::new();
	let mut bus = MemoryBus::new();

	// Start in System mode
	cpu.cpsr.set_mode_bits(0x1f);

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
				build_memory_debug_window(&mut cpu, &mut bus, &mut show_memory_debug_window, &ui);
			}

			if show_demo_window {
				ui.show_demo_window(&mut show_demo_window);
			}

			if !debug_mode {
				decode(&mut cpu, &mut bus);
			}
		});
	} else {
		println!("Cartridge couldn't be read!");
	}
}

fn build_memory_debug_window(cpu: &mut CPU, bus: &mut MemoryBus, mut show_memory_window: &mut bool, ui: &&mut Ui) {
	Window::new(im_str!("Current Memory")).size([450.0, 250.0], Condition::FirstUseEver).position([750.0, 100.0], Condition::FirstUseEver).opened(&mut show_memory_window).build(ui, || {
		ui.text("Current instruction highlighted");
		if ui.button(im_str!("Step"), [0.0, 0.0]) {
			decode(cpu, bus);
		}

		ui.separator();
		if let Some(scroll_token) = ChildWindow::new(im_str!("##ScrollingRegion")).begin(&ui) {
			ui.columns(3, im_str!("memory"), true);
			ui.set_column_width(0, 95.0);

			const ENTRIES: i32 = 300;
			let starting_address = cpu.registers[PROGRAM_COUNTER_REGISTER].saturating_sub(20);
			let mut list_clipper = ListClipper::new(ENTRIES).begin(&ui);
			while list_clipper.step() {
				for row in list_clipper.display_start()..list_clipper.display_end() {
					let address = starting_address + (row as u32 * 4);

					Selectable::new(&*im_str!("{:#010X}:", address)).selected(address == cpu.registers[PROGRAM_COUNTER_REGISTER]).span_all_columns(true).build(&ui);
					ui.next_column();

					for j in 0..4 {
						let value = bus.read_8(address as u32 + j);
						let color = if value == 0 {
							[0.5, 0.5, 0.5, 0.5]
						} else {
							[1.0, 1.0, 1.0, 1.0]
						};
						ui.text_colored(color, format!("{:02X}", value));
						if j != 3 {
							ui.same_line(0.0);
						}
					}

					ui.next_column();
					ui.text(disassemble_arm(bus.read_32(address as u32)));
					ui.next_column();
					ui.separator();
				}
			}
			ui.columns(1, im_str!(""), false);
			scroll_token.end(&ui);
		}
	});
}

fn build_cpu_debug_window(cpu: &CPU, ui: &&mut Ui, opened: &mut bool) {
	Window::new(im_str!("CPU")).size([650.0, 600.0], Condition::FirstUseEver).opened(opened).build(ui, || {
		if CollapsingHeader::new(im_str!("GPRs")).default_open(true).build(&ui) {
			ui.columns(2, im_str!("User Registers"), true);
			for (i, register) in cpu.registers.iter().enumerate() {
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
			for (i, cpsr) in [&cpu.cpsr, &cpu.spsr_fiq, &cpu.spsr_svc, &cpu.spsr_abt, &cpu.spsr_irq, &cpu.spsr_und].iter().enumerate() {
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

fn  cond_passed(cpu: &CPU, cond: u8) -> bool {
	match cond {
		0x0 => cpu.cpsr.get_z(), // Equal (Zero)
		0x1 => !cpu.cpsr.get_z(), // Not Equal (Nonzero)
		0x2 => cpu.cpsr.get_c(), // Carry set
		0x3 => !cpu.cpsr.get_c(), // Carry cleared
		0x4 => cpu.cpsr.get_n(), // Signed negative
		0x5 => !cpu.cpsr.get_n(), // Signed positive or zero
		0x6 => cpu.cpsr.get_v(), // Signed overflow
		0x7 => !cpu.cpsr.get_v(), // Signed no overflow
		0x8 => cpu.cpsr.get_c() && !cpu.cpsr.get_z(), // Unsigned higher
		0x9 => !cpu.cpsr.get_c() && cpu.cpsr.get_z(), // Unsigned lower or same
		0xa => cpu.cpsr.get_n() == cpu.cpsr.get_v(), // Signed greater or equal
		0xb => cpu.cpsr.get_n() != cpu.cpsr.get_v(), // Signed less than
		0xc => !cpu.cpsr.get_z() && cpu.cpsr.get_n() == cpu.cpsr.get_v(), // Signed greater than
		0xd => cpu.cpsr.get_z() && cpu.cpsr.get_n() != cpu.cpsr.get_v(), // Signed less or equal
		_ => true,
	}
}

fn decode(cpu: &mut CPU, bus: &mut MemoryBus) {
	let pc = cpu.registers[PROGRAM_COUNTER_REGISTER];

	// NOTE: Read CPU state
	if cpu.cpsr.get_t() {
		//let instruction = bus.read_16(pc);
	} else {
		let instruction = bus.read_32(pc);
		 print_assembly_line(disassemble_arm(instruction), pc);

		let cond = (instruction >> (32 - 4)) as u8;
		if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
			// Activate Thumb Mode
			//cpu.cpsr.set_t(true);
		} else if (0x0e00_0000 & instruction) == 0x0a00_0000 { // Branch
			if 0x0100_0000 & instruction > 0 {
				// Branch with Link
				cpu.registers[LINK_REGISTER_REGISTER] = cpu.registers[PROGRAM_COUNTER_REGISTER] + 4;
			}

			let offset = (0x00ff_ffff & instruction) as i32;
			cpu.registers[PROGRAM_COUNTER_REGISTER] = (cpu.registers[PROGRAM_COUNTER_REGISTER] as i32 + 8 + (offset * 4)) as u32;
			return;
		} else if (0x0c00_0000 & instruction) == 0x0400_0000 {
			let i = (0x0200_0000 & instruction) > 0;
			let u = (0x0080_0000 & instruction) > 0;
			let b = (0x0040_0000 & instruction) > 0;
			let t = (0x0020_0000 & instruction) > 0;
			let l = (0x0010_0000 & instruction) > 0;

			let rn_index = ((instruction & 0x000f_0000) >> 16) as usize;
			let rn = cpu.registers[rn_index];
			let rd_index = ((instruction & 0x0000_f000) >> 12) as usize;
			let address;
			if i {
				let rm = cpu.registers[(instruction & 0x0000_000f) as usize];
				let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 0).unwrap();
				let shift = (0x0000_0f80 & instruction) >> 7;

				let index;
				if shift > 0 || shift_type != EShiftType::LSL {
					match shift_type {
						EShiftType::LSL => {
							index = rm << shift;
						}
						EShiftType::LSR => {
							if shift == 0 {
								index = 0;
							} else {
								index = rm >> shift;
							}
						}
						EShiftType::ASR => {
							if shift == 0 {
								if (rm & 0x8000_0000) > 0 {
									index = 0xffff_ffff;
								} else {
									index = 0;
								}
							} else {
								index = ((rm as i32) >> shift) as u32;
							}
						}
						EShiftType::ROR => {
							if shift == 0 {
								index = ((cpu.cpsr.get_c() as u32) << 31) | (rm >> 1);
							} else {
								index = rm.rotate_right(shift);
							}
						}
					}
				} else {
					index = rm;
				}

				if u {
					address = rn + index;
				} else {
					address = rn - index;
				}
			} else {
				// Immediate
				let offset = instruction & 0x0000_0fff;
				if u {
					address = rn + offset;
				} else {
					address = rn - offset;
				}
			}

			// TODO: Account for pre/post index
			if cond_passed(cpu, cond) {
				if b {
					cpu.registers[rd_index] = bus.read_8(address) as u32;
				} else {
					cpu.registers[rd_index] = bus.read_32(address);
				}

				if p && t {
					cpu.registers[rn_index] = address;
				}
			}

		} else if (0x0c00_0000 & instruction) == 0x0000_0000 {
			let i = (0x0200_0000 & instruction) > 0;
			let s = (0x0010_0000 & instruction) > 0;
			let rn = cpu.registers[((instruction & 0x000f_0000) >> 16) as usize];
			let rd_index = ((instruction & 0x0000_f000) >> 12) as usize;

			let shifter_operand;
			let shifter_carry_out;
			if i {
				let rot = (0x0000_0f00 & instruction) >> 8;
				shifter_operand = (0x0000_00ff & instruction).rotate_right(rot * 2);

				if rot == 0 {
					shifter_carry_out = cpu.cpsr.get_c();
				} else {
					shifter_carry_out = (shifter_operand & 0x800_0000) > 0;
				}
			} else {
				let rm = cpu.registers[(instruction & 0x0000_000f) as usize];
				let r = (instruction & 0x0000_0010) > 0;
				let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 0).unwrap();
				if r {
					let rs = cpu.registers[((0x0000_0f00 & instruction) >> 8) as usize] & 0x0000_00ff;

					match shift_type {
						EShiftType::LSL => {
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.cpsr.get_c();
							} else if rs < 32 {
								shifter_operand = rm << rs;
								shifter_carry_out = rm.view_bits::<Lsb0>()[32 - rs as usize];
							} else if rs == 32 {
								shifter_operand = 0;
								shifter_carry_out = (rm & 0x0000_0001) > 0;
							} else {
								shifter_operand = 0;
								shifter_carry_out = false;
							}
						}
						EShiftType::LSR => {
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.cpsr.get_c();
							} else if rs < 32 {
								shifter_operand = rm >> rs;
								shifter_carry_out = rm.view_bits::<Lsb0>()[(rs - 1) as usize];
							} else if rs == 32 {
								shifter_operand = 0;
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = 0;
								shifter_carry_out = false;
							}
						}
						EShiftType::ASR => {
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.cpsr.get_c();
							} else if rs < 32 {
								shifter_operand = (rm as i32 >> rs) as u32;
								shifter_carry_out = rm.view_bits::<Lsb0>()[(rs - 1) as usize];
							} else {
								if (rm & 0x8000_0000) == 0 {
									shifter_operand = 0;
								} else {
									shifter_operand = 0xffff_ffff;
								}
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							}
						}
						EShiftType::ROR => {
							let rs_shift = rs & 0x1f;
							if rs == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.cpsr.get_c();
							} else if rs_shift == 0 {
								shifter_operand = rm;
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = rm.rotate_right(rs_shift);
								shifter_carry_out = rm.view_bits::<Lsb0>()[(rs_shift - 1) as usize];
							}
						}
					}
				} else {
					let shift = (0x0000_0f80 & instruction) >> 7;
					match shift_type {
						EShiftType::LSL => {
							if shift == 0 {
								shifter_operand = rm;
								shifter_carry_out = cpu.cpsr.get_c();
							} else {
								shifter_operand = rm << shift;
								shifter_carry_out = rm.view_bits::<Lsb0>()[32 - shift as usize];
							}
						}
						EShiftType::LSR => {
							if shift == 0 {
								shifter_operand = 0;
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = rm >> shift;
								shifter_carry_out = rm.view_bits::<Lsb0>()[(shift - 1) as usize];
							}
						}
						EShiftType::ASR => {
							if shift == 0 {
								if (rm & 0x8000_0000) == 0 {
									shifter_operand = 0;
								} else {
									shifter_operand = 0xffff_ffff;
								}
								shifter_carry_out = (rm & 0x8000_0000) > 0;
							} else {
								shifter_operand = (rm as i32 >> shift) as u32;
								shifter_carry_out = rm.view_bits::<Lsb0>()[(shift - 1) as usize];
							}
						}
						EShiftType::ROR => {
							if shift == 0 {
								shifter_operand = ((cpu.cpsr.get_c() as u32) << 31) | (rm >> 1);
								shifter_carry_out = (rm & 0x0000_0001) > 0;
							} else {
								shifter_operand = rm.rotate_right(shift);
								shifter_carry_out = rm.view_bits::<Lsb0>()[(shift - 1) as usize];
							}
						}
					}
				}
			};

			if cond_passed(cpu, cond) {
				match (0x01e0_0000 & instruction) >> 21 {
					// CMP
					0xa => {
						// Borrowed if carries bits over
						let (result, borrowed) = rn.overflowing_sub(shifter_operand as u32);
						// Overflow is sign changes
						let overflow = (rn as i32).signum() != (shifter_operand as i32).signum() && (rn as i32).signum() != (result as i32).signum();

						cpu.cpsr.set_n((result & 0x800_0000) > 0);
						cpu.cpsr.set_z(result == 0);
						cpu.cpsr.set_c(!borrowed);
						cpu.cpsr.set_v(overflow);
					},
					// MOV
					0xd => {
						cpu.registers[rd_index] = shifter_operand;

						let rd = cpu.registers[rd_index];
						if s && rd == PROGRAM_COUNTER_REGISTER as u32 {
							match cpu.cpsr.get_mode_bits().load_le() {
								FIQ_MODE => cpu.cpsr = cpu.spsr_fiq.clone(),
								IRQ_MODE => cpu.cpsr = cpu.spsr_irq.clone(),
								SUPERVISOR_MODE => cpu.cpsr = cpu.spsr_svc.clone(),
								ABORT_MODE => cpu.cpsr = cpu.spsr_abt.clone(),
								UNDEFINED_MODE => cpu.cpsr = cpu.spsr_und.clone(),
								_ => {}
							}
						} else if s {
							cpu.cpsr.set_n((rd & 0x800_0000) > 0);
							cpu.cpsr.set_z(rd == 0);
							cpu.cpsr.set_c(shifter_carry_out);
						}
					}
					_ => {}
				}
			}
		}

		cpu.registers[PROGRAM_COUNTER_REGISTER] += 4;
	}
}

fn print_assembly_line(line: String, pc: u32) {
	println!("{:#06X}| {}", pc, line)
}

fn disassemble_cond(cond: u8) -> &'static str {
	match cond {
		0x0 => "EQ",
		0x1 => "NE",
		0x2 => "CS",
		0x3 => "CC",
		0x4 => "MI",
		0x5 => "PL",
		0x6 => "VS",
		0x7 => "VC",
		0x8 => "HI",
		0x9 => "LS",
		0xa => "GE",
		0xb => "LT",
		0xc => "GT",
		0xd => "LE",
		_ => "",
	}
}

fn disassemble_thumb(instruction: u16, pc: u32) -> String {
	if (0xf800 & instruction) == 0x1800 {
		let op = if (0x0200 & instruction) > 0 { "ADD" } else { "SUB" };
		let i = (0x0400 & instruction) > 0;
		let rn = if i {
			format!("R{}", (0x01c0 & instruction) >> 6)
		} else {
			format!("#{:#X}", (0x01c0 & instruction) >> 6)
		};

		return format!("{} R{}, R{}, {}", op, instruction & 0x0003, (instruction & 0x001c) >> 3, rn);
	} else if (0xe000 & instruction) == 0x0000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "LSL",
			0x1 => op = "LSR",
			0x2 => op = "ASR",
			_ => panic!("ERROR!!!")
		}

		return format!("{} R{}, R{}, #{:#X}", op, instruction & 0x0003, (instruction & 0x001c) >> 3, (instruction & 0x07c0) >> 6);
	} else if (0xe000 & instruction) == 0x2000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "MOV",
			0x1 => op = "CMP",
			0x2 => op = "ADD",
			0x3 => op = "SUB",
			_ => panic!("ERROR!!!")
		}

		return format!("{} R{}, #{:#X}", op, (instruction & 0x0700) >> 8, instruction & 0x00ff);
	} else if (0xfc00 & instruction) == 0x4000 {
		let op;
		match (0x03c0 & instruction) >> 6 {
			0x0 => op = "AND",
			0x1 => op = "EOR",
			0x2 => op = "LSL",
			0x3 => op = "LSR",
			0x4 => op = "ASR",
			0x5 => op = "ADC",
			0x6 => op = "SBC",
			0x7 => op = "ROR",
			0x8 => op = "TST",
			0x9 => op = "NEG",
			0xa => op = "CMP",
			0xb => op = "CMN",
			0xc => op = "ORR",
			0xd => op = "MUL",
			0xe => op = "BIC",
			0xf => op = "MVN",
			_ => panic!("ERROR!!!")
		}

		return format!("{} R{}, R{}", op, instruction & 0x0007, (instruction & 0x001c) >> 3);
	} else if (0xfc00 & instruction) == 0x4400 {
		let op;
		match (0x0300 & instruction) >> 8 {
			0x0 => op = "ADD",
			0x1 => op = "CMP",
			0x2 => op = "MOV",
			0x3 => op = "BX",
			_ => panic!("ERROR!!!")
		}

		let rd = if op == "BX" {
			String::from("")
		} else {
			format!("R{}, ", (instruction & 0x001c) >> 3)
		};

//		if op == "BX" {
//		*thumb_mode = !*thumb_mode;
//		}

		return format!("{} {}R{}", op, rd, instruction & 0x0007);
	} else if (0xf800 & instruction) == 0x4800 {
		return format!("LDR R{}, PC, #{:#X}", (instruction & 0x0700) >> 8, instruction & 0x00ff);
	} else if (0xf200 & instruction) == 0x5000 {
		let op;
		match (0x0c00 & instruction) >> 10 {
			0x0 => op = "STR",
			0x1 => op = "STRB",
			0x2 => op = "LDR",
			0x3 => op = "LDRB",
			_ => panic!("ERROR!!!")
		}

		return format!("{} R{}, R{}, R{}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x01c0) >> 6);
	} else if (0xf200 & instruction) == 0x5200 {
		let op;
		match (0x0c00 & instruction) >> 10 {
			0x0 => op = "STRH",
			0x1 => op = "LDSB",
			0x2 => op = "LDRH",
			0x3 => op = "LDSH",
			_ => panic!("ERROR!!!")
		}

		return format!("{} R{}, R{}, R{}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x01c0) >> 6);
	} else if (0xe000 & instruction) == 0x6000 {
		let op;
		match (0x1800 & instruction) >> 11 {
			0x0 => op = "STR",
			0x1 => op = "LDR",
			0x2 => op = "STRB",
			0x3 => op = "LDRB",
			_ => panic!("ERROR!!!")
		}

		return format!("{} R{}, R{}, #{:#X}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6);
	} else if (0xf000 & instruction) == 0x8000 {
		let op = if (0x0800 & instruction) > 0 { "LDRH" } else { "STRH" };
		return format!("{} R{}, R{}, #{:#X}", op, instruction & 0x0007, (instruction & 0x0038) >> 3, (instruction & 0x07c0) >> 6);
	} else if (0xf000 & instruction) == 0x9000 {
		let op = if (0x0800 & instruction) > 0 { "LDR" } else { "STR" };
		return format!("{} R{}, SP, #{:#X}", op, (instruction & 0x0700) >> 8, instruction & 0x00ff);
	} else if (0xf000 & instruction) == 0xa000 {
		let op = if (0x0800 & instruction) > 0 { "SP" } else { "PC" };
		return format!("ADD R{}, {}, #{:#X}", (instruction & 0x0700) >> 8, op, instruction & 0x00ff);
	} else if (0xff00 & instruction) == 0xb000 {
		let sign = if (0x0080 & instruction) > 0 { "" } else { "-" };
		return format!("ADD SP, #{}{:#X}", sign, instruction & 0x00ef);
	} else if (0xf600 & instruction) == 0xb400 {
		let op = if (0x0800 & instruction) > 0 { "POP" } else { "PUSH" };
		let r = if (0x0100 & instruction) > 0 {
			if op == "PUSH" { ", LR" } else { ", PC" }
		} else {
			""
		};

		let mut regs = String::from("{ ");
		for i in 0..8 {
			if ((1 << i) & instruction) > 0 {
				let comma = if regs.len() > 2 { ", " } else { "" };
				regs += &*format!("{}R{}", comma, i);
			}
		}
		regs = format!("{}{} }}", regs, r);

		return format!("{} {}", op, regs);
	} else if (0xf000 & instruction) == 0xc000 {
		let op = if (0x0800 & instruction) > 0 { "LDMIA" } else { "STMIA" };

		let mut regs = String::from("{ ");
		for i in 0..8 {
			if ((1 << i) & instruction) > 0 {
				let comma = if regs.len() > 2 { ", " } else { "" };
				regs += &*format!("{}R{}", comma, i);
			}
		}
		regs += " }";

		return format!("{} R{}!, {}", op, (instruction & 0x0700) >> 8, regs);
	} else if (0xff00 & instruction) == 0xdf00 {
		return format!("SWI");
	} else if (0xf000 & instruction) == 0xd000 {
		let op;
		match (0x0f00 & instruction) >> 8 {
			0x0 => op = "BEQ",
			0x1 => op = "BNE",
			0x2 => op = "BCS",
			0x3 => op = "BCC",
			0x4 => op = "BMI",
			0x5 => op = "BPL",
			0x6 => op = "BVS",
			0x7 => op = "BVC",
			0x8 => op = "BHI",
			0x9 => op = "BLS",
			0xa => op = "BGE",
			0xb => op = "BLT",
			0xc => op = "BGT",
			0xd => op = "BLE",
			0xe => op = "UNDEFINED",
			_ => panic!("ERROR!!!")
		}

		// TODO: Interpret as signed
		return format!("{} Offset: {:#X}", op, instruction & 0x00ff);
	} else if (0xf800 & instruction) == 0xf000 {
		// TODO: Interpret as signed
		let hi = (instruction & 0x07ff) as u32;

//		pc += 2;
//		let bytes2: [u8; 2] = [data[pc], data[pc + 1]];
//		let instruction2 = u16::from_le_bytes(bytes2);
//		if (0xf800 & instruction2) != 0xf800 {
//			panic!("Instruction after BL is not BL!!!");
//		}
//		let lo = (instruction & 0x07ff) as u32;
//		let offset = (hi << 12) & (lo << 1);
//
		return format!("BL Target: {:#X}", pc as u32 + 4 + hi);
	} else {
		return format!("Missing instruction!");
	}
}

fn disassemble_arm(instruction: u32) -> String {
	let cond = (instruction >> (32 - 4)) as u8;
	if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
		return format!("BX {} R{}", cond, instruction & 0x0000_000f);
	} else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
		if 1 << 24 & instruction > 0 {
			return format!("BL {} #{:#X}", disassemble_cond(cond), instruction & 0x00ff_ffff);
		} else {
			return format!("B {} #{:#X}", disassemble_cond(cond), instruction & 0x00ff_ffff);
		}
	} else if (0xe000_0010 & instruction) == 0x0600_0010 {
		return format!("Undefined instruction!");
	} else if (0x0fb0_0ff0 & instruction) == 0x0100_0090 {
		if 1 << 22 & instruction > 0 {
			return format!("SWPB R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, instruction & 0x0000_000f, (instruction & 0x000f_0000) >> 16);
		} else {
			return format!("SWP R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, instruction & 0x0000_000f, (instruction & 0x000f_0000) >> 16);
		}
	} else if (0x0f00_00f0 & instruction) == 0x0000_0090 {
		let s = if (0x0010_0000 & instruction) > 0 { "S" } else { "" };

		let op;
		match (0x01e0_0000 & instruction) >> 21 {
			0x0 => op = "MUL",
			0x1 => op = "MLA",
			0x4 => op = "UMULL",
			0x5 => op = "UMLAL",
			0x6 => op = "SMULL",
			0x7 => op = "SMALL",
			_ => panic!("ERROR!!!")
		}

		// TODO: Revisit params!!!
		return format!("{}{} R{}, R{}, R{}", op, s, (instruction & 0x000f_0000) >> 16, instruction & 0x0000_000f, (instruction & 0x0000_0f00) >> 8);
	} else if (0x0fbf_0fff & instruction) == 0x010f_0000 {
		if (instruction & 0x0010_0000) > 0 {
			return format!("MRS R{}, CPSR", (instruction & 0x0000_f000) >> 12, );
		} else {
			return format!("MRS R{}, SPSR", (instruction & 0x0000_f000) >> 12, );
		}
	} else if (0x0db0_f000 & instruction) == 0x0129_f000 {
		let mut fields = String::from("");
		if (0x0008_000 & instruction) > 0 {
			fields += "f";
		}
		if (0x0004_0000 & instruction) > 0 {
			fields += "s";
		}
		if (0x0002_0000 & instruction) > 0 {
			fields += "x";
		}
		if (0x0001_0000 & instruction) > 0 {
			fields += "c";
		}
		if fields.len() > 0 {
			fields = String::from("_") + &*fields;
		}
		let psr = if (instruction & 0x0010_0000) > 0 { "CPSR" } else { "SPSR" };
		if (instruction & 0x0200_0000) > 0 {
			return format!("MSR {}{}, #{:#X}", psr, fields, instruction & 0x0000_00ff);
		} else {
			return format!("MSR {}{}, R{}", psr, fields, instruction & 0x0000_00ff);
		}
	} else if (0x0c00_0000 & instruction) == 0x0400_0000 {
		let i = (0x0200_0000 & instruction) > 0;
		let u = if (0x0080_0000 & instruction) > 0 { "+" } else { "-" };
		let b = if (0x0040_0000 & instruction) > 0 { "B" } else { "" };
		let t = if (0x0020_0000 & instruction) > 0 { "T" } else { "" };
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };

		let rn = (instruction & 0x000f_0000) >> 16;
		let address;
		if i {
			let rm = instruction & 0x0000_000f;
			let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 0).unwrap();
			let shift = (0x0000_0f80 & instruction) >> 7;

			address = format!("[R{}, R{}, {:?} #{:#X}]", rn, rm, shift_type, shift);
		} else {
			address = format!("[R{}, #{}{}]", rn, u, instruction & 0x0000_0fff);
		}

		return format!("{}{}{} R{}, {}", l, b, t, (instruction & 0x0000_f000) >> 12, address);
	} else if (0x0e40_0F90 & instruction) == 0x0000_0090 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
		let op;
		if (0x0000_0020 & instruction) > 0 {
			op = "H"
		} else if (0x0000_0030 & instruction) > 0 {
			op = "SB"
		} else if (0x0000_0040 & instruction) > 0 {
			op = "SH"
		} else {
			panic!("ERROR!!!");
		}

		return format!("{}{} R{}", l, op, instruction & 0x0000_000f);
	} else if (0x0e40_0090 & instruction) == 0x0040_0090 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
		let op;
		if (0x0000_0020 & instruction) > 0 {
			op = "H"
		} else if (0x0000_0030 & instruction) > 0 {
			op = "SB"
		} else if (0x0000_0040 & instruction) > 0 {
			op = "SH"
		} else {
			panic!("ERROR!!!");
		}

		return format!("{}{} #{:#X}", l, op, (instruction & 0x0000_0f00) >> 4 | instruction & 0x0000_000f);
	} else if (0x0e00_0000 & instruction) == 0x0800_0000 {
		let l = if (0x0010_0000 & instruction) > 0 { "LDM" } else { "STM" };
		let w = if (0x0020_0000 & instruction) > 0 { "!" } else { "" };
		let s = if (0x0040_0000 & instruction) > 0 { "^" } else { "" };
		let u = if (0x0080_0000 & instruction) > 0 { "I" } else { "D" };
		let p = if (0x0100_0000 & instruction) > 0 { "B" } else { "A" };

		let mut regs = String::from("{ ");
		for i in 0..16 {
			if ((1 << i) & instruction) > 0 {
				let comma = if regs.len() > 2 { ", " } else { "" };
				regs += &*format!("{}R{}", comma, i);
			}
		}
		regs += " }";

		return format!("{}{}{} R{}{}, {}{}", l, u, p, (instruction & 0x000f_0000) >> 16, w, regs, s);
	} else if (0x0f00_0000 & instruction) == 0x0f00_0000 {
		return format!("SWI");
	} else if (0x0c00_0000 & instruction) == 0x0000_0000 {
		let i = (0x0200_0000 & instruction) > 0;
		let mut s = if (0x0010_0000 & instruction) > 0 { "S" } else { "" };
		let mut rn = &*format!("R{},", (instruction & 0x000f_0000) >> 16);
		let mut rd = &*format!("R{},", (instruction & 0x0000_f000) >> 12);

		let op;
		match (0x01e0_0000 & instruction) >> 21 {
			0x0 => op = "AND",
			0x1 => op = "EOR",
			0x2 => op = "SUB",
			0x3 => op = "RSB",
			0x4 => op = "ADD",
			0x5 => op = "ADC",
			0x6 => op = "SBC",
			0x7 => op = "RSC",
			0x8 => {
				op = "TST";
				rd = "";
				s = "";
			},
			0x9 => {
				op = "TEQ";
				rd = "";
				s = "";
			},
			0xa => {
				op = "CMP";
				rd = "";
				s = "";
			},
			0xb => {
				op = "CMN";
				rd = "";
				s = "";
			},
			0xc => op = "ORR",
			0xd => {
				op = "MOV";
				rn = "";
			},
			0xe => op = "BIC",
			0xf => {
				op = "MVN";
				rn = "";
			},
			_ => panic!("ERROR!!!")
		}

		let shifter_operand;
		if i {
			let rot = (0x0000_0f00 & instruction) >> 8;
			shifter_operand = format!("#{:#X}", (0x0000_00ff & instruction).rotate_right(rot * 2));
		} else {
			let rm = instruction & 0x0000_000f;
			let r = (instruction & 0x0000_0010) > 0;
			let shift_type: EShiftType = FromPrimitive::from_u32((instruction & 0x0000_0060) >> 0).unwrap();
			if r {
				let rs = (0x0000_0f00 & instruction) >> 8;
				shifter_operand = format!("R{}, {:?}, R{}", rm, shift_type, rs);
			} else {
				let shift = (0x0000_0f80 & instruction) >> 7;
				shifter_operand = format!("R{}, {:?}, #{:#X}", rm, shift_type, shift);
			}
		}

		return format!("{}{} {} {}{} {}", op, s, disassemble_cond(cond), rd, rn, shifter_operand);
	} else {
		return format!("Missing instruction!");
	}
}

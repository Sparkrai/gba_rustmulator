use criterion::{criterion_group, criterion_main, Criterion};
use gba_rustmulator::arm7tdmi::{EExceptionType, cpu::CPU};
use gba_rustmulator::system::SystemBus;
use std::fs::File;
use std::io::Read;

fn frame(bus: &mut SystemBus, cpu: &mut CPU) {
    // NOTE: Advance GBA by one frame
    const CYCLES_PER_FRAME: u32 = 280_896;
    let mut current_cycle = 0u32;
    for _ in 0..=CYCLES_PER_FRAME {
        current_cycle = (current_cycle + 1) % CYCLES_PER_FRAME;
        let (h_blank_irq, v_blank_irq) = bus.ppu.step(current_cycle);

        if bus.ppu.get_disp_stat().get_v_counter_flag()
            && bus.io_regs.get_ime() && bus.io_regs.get_ie().get_v_counter_match()
            && bus.ppu.get_disp_stat().get_v_counter_irq()
        {
            bus.io_regs.get_mut_if().set_v_counter_match(true);
            cpu.exception(EExceptionType::Irq);
            bus.io_regs.halted = false;
        }

        // H-Blank
        if h_blank_irq && bus.io_regs.get_ime() && bus.io_regs.get_ie().get_h_blank() && bus.ppu.get_disp_stat().get_h_blank_irq() {
            bus.io_regs.get_mut_if().set_h_blank(true);
            cpu.exception(EExceptionType::Irq);
            bus.io_regs.halted = false;
        } else if v_blank_irq && bus.io_regs.get_ime() && bus.io_regs.get_ie().get_v_blank() && bus.ppu.get_disp_stat().get_v_blank_irq() {
            // V-Blank
            bus.io_regs.get_mut_if().set_v_blank(true);
            cpu.exception(EExceptionType::Irq);
            bus.io_regs.halted = false;
        }

        cpu.step(bus);
    }
    
    bus.ppu.render();
}

fn bench_bios(c: &mut Criterion) {
	let mut cpu = CPU::new();
	// Start in System mode
	cpu.get_mut_cpsr().set_mode_bits(0x1f);

	let mut bios_data = Vec::<u8>::new();
	File::open("data/bios.gba").expect("Bios couldn't be opened!").read_to_end(&mut bios_data).unwrap();

	let mut bus = SystemBus::new(bios_data.into_boxed_slice());

	c.bench_function("Bios", |b| b.iter(|| frame(&mut bus, &mut cpu)));
}

criterion_group!(benches, bench_bios);
criterion_main!(benches);

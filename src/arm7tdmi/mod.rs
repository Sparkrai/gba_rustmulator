use num_derive::*;
use num_traits::{AsPrimitive, PrimInt};

use crate::arm7tdmi::cpu::{CPU, PROGRAM_COUNTER_REGISTER};
use crate::system::{MemoryInterface, SystemBus};

mod arm;
pub mod cpu;
mod psr;
mod thumb;

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum EOperatingMode {
	UserMode = 0x10,
	FiqMode = 0x11,
	IrqMode = 0x12,
	SupervisorMode = 0x13,
	AbortMode = 0x17,
	UndefinedMode = 0x1b,
	SystemMode = 0x1f,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EExceptionType {
	Reset,
	Undefined,
	SoftwareInterrupt,
	//	PrefetchAbort,
	//	DataAbort,
	Irq,
	Fiq,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum EShiftType {
	LSL,
	LSR,
	ASR,
	ROR,
}

pub fn sign_extend<T>(x: T, bits: u8) -> i32
where
	T: PrimInt + AsPrimitive<i32>,
{
	let m = (1u32 << (bits - 1)) as i32;
	(x.as_() ^ m) - m
}

pub fn cond_passed(cpu: &CPU, cond: u8) -> bool {
	match cond {
		0x0 => cpu.get_cpsr().get_z(),                                                      // Equal (Zero)
		0x1 => !cpu.get_cpsr().get_z(),                                                     // Not Equal (Nonzero)
		0x2 => cpu.get_cpsr().get_c(),                                                      // Carry set
		0x3 => !cpu.get_cpsr().get_c(),                                                     // Carry cleared
		0x4 => cpu.get_cpsr().get_n(),                                                      // Signed negative
		0x5 => !cpu.get_cpsr().get_n(),                                                     // Signed positive or zero
		0x6 => cpu.get_cpsr().get_v(),                                                      // Signed overflow
		0x7 => !cpu.get_cpsr().get_v(),                                                     // Signed no overflow
		0x8 => cpu.get_cpsr().get_c() && !cpu.get_cpsr().get_z(),                           // Unsigned higher
		0x9 => !cpu.get_cpsr().get_c() || cpu.get_cpsr().get_z(),                           // Unsigned lower or same
		0xa => cpu.get_cpsr().get_n() == cpu.get_cpsr().get_v(),                            // Signed greater or equal
		0xb => cpu.get_cpsr().get_n() != cpu.get_cpsr().get_v(),                            // Signed less than
		0xc => !cpu.get_cpsr().get_z() && cpu.get_cpsr().get_n() == cpu.get_cpsr().get_v(), // Signed greater than
		0xd => cpu.get_cpsr().get_z() || cpu.get_cpsr().get_n() != cpu.get_cpsr().get_v(),  // Signed less or equal
		_ => true,
	}
}

pub fn load_32_from_memory(bus: &SystemBus, address: u32) -> u32 {
	let data;
	if (address & 0x0000_0003) == 0 {
		data = bus.read_32(address);
	} else {
		// NOTE: Forced alignment and rotation of data! (UNPREDICTABLE)
		data = bus.read_32(address & !0x0000_0003).rotate_right((address & 0x0000_0003) * 8);
	}

	data
}

pub fn decode(cpu: &mut CPU, bus: &mut SystemBus) {
	// NOTE: Read CPU state
	let pc = cpu.get_current_pc();
	if cpu.get_cpsr().get_t() {
		let instruction = bus.read_16(pc);
		thumb::operate_thumb(instruction, cpu, bus);

		if cpu.get_current_pc() == pc {
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, cpu.get_current_pc() + 2);
		} else {
			// NOTE: Force alignment!!!
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, cpu.get_current_pc() & !0x1);
		}
	} else {
		let instruction = bus.read_32(pc);
		arm::operate_arm(cpu, bus, instruction);

		if cpu.get_current_pc() == pc {
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, cpu.get_current_pc() + 4);
		} else {
			// NOTE: Force alignment!!!
			cpu.set_register_value(PROGRAM_COUNTER_REGISTER, cpu.get_current_pc() & !0x1);
		}
	}
}

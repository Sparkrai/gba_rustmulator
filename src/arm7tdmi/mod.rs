use bitvec::array::BitArray;
use bitvec::order::Lsb0;
use bitvec::prelude::BitSlice;
use num_derive::*;
use num_traits::{AsPrimitive, PrimInt};

use crate::arm7tdmi::cpu::CPU;
use crate::system::{MemoryInterface, SystemBus};

mod arm;
pub mod cpu;
mod psr;
mod thumb;

pub type Gba32BitSlice = BitSlice<Lsb0, u32>;
pub type Gba8BitSlice = BitSlice<Lsb0, u8>;
pub type Gba32BitRegister = BitArray<Lsb0, [u32; 1]>;

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
	PrefetchAbort,
	DataAbort,
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
		0x9 => !cpu.get_cpsr().get_c() && cpu.get_cpsr().get_z(),                           // Unsigned lower or same
		0xa => cpu.get_cpsr().get_n() == cpu.get_cpsr().get_v(),                            // Signed greater or equal
		0xb => cpu.get_cpsr().get_n() != cpu.get_cpsr().get_v(),                            // Signed less than
		0xc => !cpu.get_cpsr().get_z() && cpu.get_cpsr().get_n() == cpu.get_cpsr().get_v(), // Signed greater than
		0xd => cpu.get_cpsr().get_z() && cpu.get_cpsr().get_n() != cpu.get_cpsr().get_v(),  // Signed less or equal
		_ => true,
	}
}

pub fn decode(cpu: &mut CPU, bus: &mut SystemBus) {
	// NOTE: Read CPU state
	let pc = cpu.get_current_pc();
	if cpu.get_cpsr().get_t() {
		let instruction = bus.read_16(pc);
		//		print_assembly_line(disassemble_thumb(instruction), pc);

		thumb::operate_thumb(instruction, cpu, bus);
	} else {
		let instruction = bus.read_32(pc);
		//		print_assembly_line(disassemble_arm(instruction), pc);

		arm::operate_arm(cpu, bus, instruction);
	}
}

use bitvec::array::BitArray;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitSlice, BitView};
use num_traits::{FromPrimitive, PrimInt};

use crate::arm7tdmi::cpu::{CPU, LINK_REGISTER_REGISTER, PROGRAM_COUNTER_REGISTER};
use crate::memory::MemoryBus;
use num_derive::*;

pub mod cpu;
mod psr;
mod thumb;
mod arm;

pub type Gba32BitSlice = BitSlice<Lsb0, u32>;
pub type GbaRegisterBits = BitArray<Lsb0, [u32; 1]>;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum EShiftType {
	LSL,
	LSR,
	ASR,
	ROR,
}

fn sign_extend(x: u32) -> i32 {
	(x as i32 ^ 0x80_0000) - 0x80_0000
}

pub fn decode(cpu: &mut CPU, bus: &mut MemoryBus) {
	// NOTE: Read CPU state
	if cpu.get_cpsr().get_t() {
		let pc = cpu.get_register_value(PROGRAM_COUNTER_REGISTER) - 4;
		let instruction = bus.read_16(pc);
//		print_assembly_line(disassemble_thumb(instruction), pc);

		thumb::operate_thumb(instruction, cpu, bus);
	} else {
		let pc = cpu.get_register_value(PROGRAM_COUNTER_REGISTER) - 8;
		let instruction = bus.read_32(pc);
//		print_assembly_line(disassemble_arm(instruction), pc);

		arm::operate_arm(cpu, bus, instruction);
	}
}

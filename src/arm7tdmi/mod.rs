use bitvec::array::BitArray;
use bitvec::order::Lsb0;
use bitvec::prelude::BitSlice;
use num_derive::*;
use num_traits::{AsPrimitive, PrimInt};

use crate::arm7tdmi::cpu::CPU;
use crate::memory::MemoryBus;

mod arm;
pub mod cpu;
mod psr;
mod thumb;

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

fn sign_extend<T>(x: T) -> i32
where
	T: PrimInt + AsPrimitive<i32>,
{
	let bit = (1u32 << (31 - x.leading_zeros())) as i32;
	(x.as_() ^ bit) - bit
}

pub fn decode(cpu: &mut CPU, bus: &mut MemoryBus) {
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

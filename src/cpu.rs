use bitvec::prelude::*;

pub type Gba32BitSlice = BitSlice<Msb0, u32>;
pub type GbaRegisterBits = bitarr![for 32, in Msb0, u32];

pub struct CPU {
	// General Purpose Registers
	r0: u32,
	r1: u32,
	r2: u32,
	r3: u32,
	r4: u32,
	r5: u32,
	r6: u32,
	r7: u32,
	r8: u32,
	r9: u32,
	r10: u32,
	r11: u32,
	r12: u32,

	// FIQ Registers
	r8_fiq: u32,
	r9_fiq: u32,
	r10_fiq: u32,
	r11_fiq: u32,
	r12_fiq: u32,

	// Stack Pointer Registers
	r13: u32,
	r13_fiq: u32,
	r13_svc: u32,
	r13_abt: u32,
	r13_irq: u32,
	r13_und: u32,

	// Link Registers
	r14: u32,
	r14_fiq: u32,
	r14_svc: u32,
	r14_abt: u32,
	r14_irq: u32,
	r14_und: u32,

	// Program Counter
	r15: u32,

	// Current Program Status Register
	cpsr: GbaRegisterBits,

	// Saved Program Status Registers
	spsr_fiq: GbaRegisterBits,
	spsr_svc: GbaRegisterBits,
	spsr_abt: GbaRegisterBits,
	spsr_irq: GbaRegisterBits,
	spsr_und: GbaRegisterBits,
}

struct CPSR {
	bits: GbaRegisterBits,
}

impl CPSR {
	
}
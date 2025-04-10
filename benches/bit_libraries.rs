use bitfield::*;
use bitvec::prelude::*;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

struct BitVecTest {
	data: BitArray<Lsb0, [u16; 1]>,
}

impl BitVecTest {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u16; 0; 16] }
	}

	pub fn get_test_bit(&self) -> bool {
		self.data[3]
	}

	pub fn set_test_bit(&mut self, value: bool) {
		self.data.set(3, value);
	}

	pub fn set_other_bit(&mut self, value: bool) {
		self.data.set(1, value);
	}

	pub fn get_bits(&self) -> u8 {
		self.data[0..4].load_le()
	}

	pub fn set_bits(&mut self, value: u8) {
		self.data[0..4].store_le(value);
	}
}

bitfield! {
	pub struct BitfieldTest {
		data: u16
	}
	impl Debug;
	// The fields default to u16
	pub get_test_bit, set_test_bit: 3;
	pub _, set_other_bit: 1;
	pub u8, get_bits, set_bits: 3, 0;
}

fn test_register_bitvec(value: u8) -> u8 {
	let mut test_register = BitVecTest::new();

	test_register.set_test_bit(black_box(true));
	test_register.set_other_bit(black_box(true));
	test_register.set_bits(value);

	let mut result = 0;
	result |= (test_register.get_test_bit() as u8) | test_register.get_bits();

	result
}

fn test_register_bitfield(value: u8) -> u8 {
	let mut test_register = BitfieldTest(0);

	test_register.set_test_bit(black_box(true));
	test_register.set_other_bit(black_box(true));
	test_register.set_bits(value);

	let mut result = 0;
	result |= (test_register.get_test_bit() as u8) | test_register.get_bits();

	result
}

fn bench_bit_registers(c: &mut Criterion) {
	let mut group = c.benchmark_group("Bit Register");
	for i in (0..=255).step_by(32) {
		group.bench_with_input(BenchmarkId::new("bitvec", i), &i, |b, i| b.iter(|| test_register_bitvec(*i)));
		group.bench_with_input(BenchmarkId::new("bitfield", i), &i, |b, i| b.iter(|| test_register_bitfield(*i)));
	}
	group.finish();
}

fn test_number_bitvec(value: u8) -> u8 {
	let mut number = 0u32;
	let test_number = number.view_bits_mut::<Lsb0>();

	test_number.set(3, black_box(true));
	test_number.set(1, black_box(true));
	test_number[0..4].store_le(value);

	let mut result = 0;
	result |= (test_number[3] as u8) | test_number[0..4].load_le::<u8>();

	result
}

fn test_number_bitfield(value: u8) -> u8 {
	let mut test_number = 0u32;

	test_number.set_bit(3, black_box(true));
	test_number.set_bit(1, black_box(true));
	test_number.set_bit_range(3, 0, value);

	let mut result = 0;
	result |= (test_number.bit(3) as u8) | BitRange::<u8>::bit_range(&test_number, 3, 0);

	result
}

fn bench_bit_number(c: &mut Criterion) {
	let mut group = c.benchmark_group("Bit Number");
	for i in (0..=255).step_by(32) {
		group.bench_with_input(BenchmarkId::new("bitvec", i), &i, |b, i| b.iter(|| test_number_bitvec(*i)));
		group.bench_with_input(BenchmarkId::new("bitfield", i), &i, |b, i| b.iter(|| test_number_bitfield(*i)));
	}
	group.finish();
}

criterion_group!(benches, bench_bit_registers, bench_bit_number);
criterion_main!(benches);

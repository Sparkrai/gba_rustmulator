use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bitvec::prelude::*;

struct TestRegister {
    data: BitArray<Lsb0, [u16; 1]>,
}

impl TestRegister {
	pub fn new() -> Self {
		Self { data: bitarr![Lsb0, u16; 0; 16] }
	}

	pub fn get_bit(&self) -> bool {
		self.data[3]
	}

    pub fn set_bit(&mut self, value: bool) {
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

fn test_register_bitvec(value: u8) -> u8 {
    let mut test_register = TestRegister::new();

    test_register.set_bit(black_box(true));
    test_register.set_other_bit(black_box(true));
    test_register.set_bits(value);

    let mut result = 0;
    result |= (test_register.get_bit() as u8) | test_register.get_bits();

    result
}

pub fn bit_register_benchmark(c: &mut Criterion) {
    c.bench_function("Bit Register", |b| b.iter(|| test_register_bitvec(black_box(9))));
}

criterion_group!(benches, bit_register_benchmark);
criterion_main!(benches);
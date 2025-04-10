pub const MEMORY_SIZE: usize = u32::max_value() as usize;

pub struct MemoryBus {
	memory: Vec<u8>,
}

impl MemoryBus {
	pub fn new() -> Self {
		Self {
			memory: vec![0; MEMORY_SIZE],
		}
	}

	pub fn get8(&self, addr: u32) -> u8 {
		self.memory[addr as usize]
	}

	pub fn set8(&mut self, addr: u32, value: u8) {
		self.memory[addr as usize] = value;
	}

	pub fn get16(&self, addr: u32) -> u16 {
		unsafe {
			self.memory.as_ptr().offset(addr as isize) as u16
		}
	}

	pub fn set16(&mut self, addr: u32, value: u16) {
		unsafe {
			*(self.memory.as_mut_ptr().offset(addr as isize) as *mut u16) = value;
		}
	}

	pub fn get32(&self, addr: u32) -> u32 {
		unsafe {
			self.memory.as_ptr().offset(addr as isize) as u32
		}
	}

	pub fn set32(&mut self, addr: u32, value: u32) {
		unsafe {
			*(self.memory.as_mut_ptr().offset(addr as isize) as *mut u32) = value;
		}
	}

	pub fn load_bios(&mut self, bios: &[u8]) {
		self.memory[0..=0x0000_3FFF].copy_from_slice(bios);
	}

	pub fn load_cartridge(&mut self, rom: &[u8]) {
		// Mirrored twice in hardware
		self.memory[0x0800_0000..0x0800_0000 + rom.len()].copy_from_slice(rom);
		self.memory[0x0A00_0000..0x0A00_0000 + rom.len()].copy_from_slice(rom);
		self.memory[0x0E01_0000..0x0E01_0000 + rom.len()].copy_from_slice(rom);
	}
}
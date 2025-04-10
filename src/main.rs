use std::fs::File;
use std::io::Read;

fn main() {
    let mut data = Vec::<u8>::new();
    if File::open("data/demos/hello.gba").unwrap().read_to_end(&mut data).is_ok() {
        let op_bytes = 1;
        let pc = 0;
        while pc < data.len() {
            println!("Current PC: {}", pc);
            let bytes: [u8; 4] = [data[pc], data[pc+1], data[pc+2], data[pc+3]];
            let instruction = u32::from_ne_bytes(bytes);
            if instruction.bit_range(25..27) == 0b101 {
                println!("BL: {}", pc);
            }
        }
    }
}

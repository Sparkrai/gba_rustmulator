use std::fs::File;
use std::io::Read;

fn main() {
    let mut data = Vec::<u8>::new();
    if File::open("data/demos/hello.gba").unwrap().read_to_end(&mut data).is_ok() {
        let mut pc = 0;
        while pc < data.len() {
            println!("Current PC: {}", pc);
            let bytes: [u8; 4] = [data[pc], data[pc+1], data[pc+2], data[pc+3]];
            let instruction = u32::from_ne_bytes(bytes);
            let cond = instruction >> (32 - 4);
            if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
                println!("BX {} {:#X}", cond, instruction & 0x0000_000f);
            } else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
                if 1 << 24 & instruction > 0 {
                    println!("BL {} {:#X}", cond, instruction & 0x0000_000f);
                } else {
                    println!("B {} {:#X}", cond, instruction & 0x0000_000f);
                }
            } else if (0xe000_0010 & instruction) == 0x0600_0010 {
                println!("Undefined instruction!");
            } else if (0x0fb0_0ff0 & instruction) == 0x0100_0090 {
                if 1 << 22 & instruction > 0 {
                    println!("SWPB {:#X}, {:#X}, {:#X}", instruction & 0x0000_f000, instruction & 0x0000_000f, instruction & 0x000f_0000);
                } else {
                    println!("SWP {:#X}, {:#X}, {:#X}", instruction & 0x0000_f000, instruction & 0x0000_000f, instruction & 0x000f_0000);
                }
            }

//            else if (0x0fc0_00f0 & instruction) == 0x0000_0090 {
//                Multiply
//            } else if (0x0f80_00f0 & instruction) == 0x0080_0090 {
//                MultiplyLong
//            } else if (0x0fbf_0fff & instruction) == 0x010f_0000 {
//                MoveFromStatus
//            } else if (0x0fbf_fff0 & instruction) == 0x0129_f000 {
//                MoveToStatus
//            } else if (0x0dbf_f000 & instruction) == 0x0128_f000 {
//                MoveToFlags
//            } else if (0x0c00_0000 & instruction) == 0x0400_0000 {
//                SingleDataTransfer
//            } else if (0x0e40_0F90 & instruction) == 0x0000_0090 {
//                HalfwordDataTransferRegOffset
//            } else if (0x0e40_0090 & instruction) == 0x0040_0090 {
//                HalfwordDataTransferImmediateOffset
//            } else if (0x0e00_0000 & instruction) == 0x0800_0000 {
//                BlockDataTransfer
//            } else if (0x0f00_0000 & instruction) == 0x0f00_0000 {
//                SoftwareInterrupt
//            } else if (0x0c00_0000 & instruction) == 0x0000_0000 {
//                DataProcessing
//            }

            else {
                println!("Missing instruction!");
            }

            pc += 4;
        }
    }
}

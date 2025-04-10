use std::fs::File;
use std::io::Read;

fn main() {
    let mut data = Vec::<u8>::new();
    if File::open("data/demos/hello.gba").unwrap().read_to_end(&mut data).is_ok() {
        let mut pc = 0;
        while pc < data.len() {
//            println!("Current PC: {}", pc);
            let bytes: [u8; 4] = [data[pc], data[pc+1], data[pc+2], data[pc+3]];
            let instruction = u32::from_ne_bytes(bytes);
            let cond = instruction >> (32 - 4);
            if (0x0fff_fff0 & instruction) == 0x012f_ff10 {
                println!("BX {} R{}", cond, instruction & 0x0000_000f);
            } else if (0x0e00_0000 & instruction) == 0x0a00_0000 {
                if 1 << 24 & instruction > 0 {
                    println!("BL {} R{}", cond, instruction & 0x0000_000f);
                } else {
                    println!("B {} R{}", cond, instruction & 0x0000_000f);
                }
            } else if (0xe000_0010 & instruction) == 0x0600_0010 {
                println!("Undefined instruction!");
            } else if (0x0fb0_0ff0 & instruction) == 0x0100_0090 {
                if 1 << 22 & instruction > 0 {
                    println!("SWPB R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, instruction & 0x0000_000f, (instruction & 0x000f_0000) >> 16);
                } else {
                    println!("SWP R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, instruction & 0x0000_000f, (instruction & 0x000f_0000) >> 16);
                }
            } else if (0x0fc0_00f0 & instruction) == 0x0000_0090 {
                println!("MUL R{}, R{}, R{}", (instruction & 0x000f_0000) >> 16, instruction & 0x0000_000f, (instruction & 0x0000_0f00) >> 8);
            } else if (0x0f80_00f0 & instruction) == 0x0080_0090 {
                println!("UMULL R{}, R{}, R{}, R{}", (instruction & 0x0000_f000) >> 12, (instruction & 0x000f_0000) >> 16, instruction & 0x0000_000f, (instruction & 0x0000_0f00) >> 8);
            } else if (0x0fbf_0fff & instruction) == 0x010f_0000 {
                if (instruction & 0x0010_0000) > 0 {
                    println!("MRS R{}, CPSR", (instruction & 0x0000_f000) >> 12, );
                } else {
                    println!("MRS R{}, SPSR", (instruction & 0x0000_f000) >> 12, );
                }
            } else if (0x0db0_f000 & instruction) == 0x0129_f000 {
                let mut fields = String::from("");
                if (0x0008_000 & instruction) > 0 {
                    fields += "f";
                }
                if (0x0004_0000 & instruction) > 0 {
                    fields += "s";
                }
                if (0x0002_0000 & instruction) > 0 {
                    fields += "x";
                }
                if (0x0001_0000 & instruction) > 0 {
                    fields += "c";
                }
                if fields.len() > 0 {
                    fields = String::from("_") + &*fields;
                }
                let psr = if (instruction & 0x0010_0000) > 0 { "CPSR" } else { "SPSR" };
                if (instruction & 0x0200_0000) > 0 {
                    println!("MSR {}{}, {:#X}", psr, fields, instruction & 0x0000_00ff);
                } else {
                    println!("MSR {}{}, R{}", psr, fields, instruction & 0x0000_00ff);
                }
            } else if (0x0c00_0000 & instruction) == 0x0400_0000 {
                let b = if (0x0040_0000 & instruction) > 0 { "B" } else { "" };
                let t = if (0x0020_0000 & instruction) > 0 { "T" } else { "" };
                let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };

                println!("{}{}{} R{}", l, b, t, (instruction & 0x0000_f000) >> 12);
            } else if (0x0e40_0F90 & instruction) == 0x0000_0090 {
                let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
                let op;
                if (0x0000_0020 & instruction) > 0 {
                    op = "H"
                } else if (0x0000_0030 & instruction) > 0 {
                    op = "SB"
                } else if (0x0000_0040 & instruction) > 0 {
                    op = "SH"
                } else {
                    panic!("ERROR!!!");
                }

                println!("{}{} R{}", l, op, instruction & 0x0000_000f);
            } else if (0x0e40_0090 & instruction) == 0x0040_0090 {
                let l = if (0x0010_0000 & instruction) > 0 { "LDR" } else { "STR" };
                let op;
                if (0x0000_0020 & instruction) > 0 {
                    op = "H"
                } else if (0x0000_0030 & instruction) > 0 {
                    op = "SB"
                } else if (0x0000_0040 & instruction) > 0 {
                    op = "SH"
                } else {
                    panic!("ERROR!!!");
                }

                println!("{}{} {:#X}", l, op, (instruction & 0x0000_0f00) >> 4 | instruction & 0x0000_000f);
            } else if (0x0e00_0000 & instruction) == 0x0800_0000 {
                let l = if (0x0010_0000 & instruction) > 0 { "LDM" } else { "STM" };
                let w = if (0x0020_0000 & instruction) > 0 { "!" } else { "" };
                let s = if (0x0040_0000 & instruction) > 0 { "^" } else { "" };
                let u = if (0x0080_0000 & instruction) > 0 { "I" } else { "D" };
                let p = if (0x0100_0000 & instruction) > 0 { "B" } else { "A" };

                let mut regs = String::from("{ ");
                for i in 0..16 {
                    if (i & instruction) > 0 {
                        let comma = if regs.len() > 2 { ", " } else { "" };
                        regs += &*format!("{}R{}", comma, i);
                    }
                }
                regs += " }";

                println!("{}{}{} R{}{}, {}{}", l, u, p, (instruction & 0x000f_0000) >> 16, w, regs, s);
            } else if (0x0f00_0000 & instruction) == 0x0f00_0000 {
//                SoftwareInterrupt
            } else if (0x0c00_0000 & instruction) == 0x0000_0000 {
//                DataProcessing
            } else {
//                println!("Missing instruction!");
            }

            pc += 4;
        }
    }
}

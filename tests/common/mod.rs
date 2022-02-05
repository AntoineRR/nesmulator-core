use nes_emulator::nes::NES;

pub const ROM_PATH_PREFIX: &str = "../ROM/Tests/nes-test-roms/";
const RETURN_CODE_ADDRESS: u16 = 0x6000;
const MESSAGE_ADDRESS: u16 = 0x6004;
const END_OF_MESSAGE: u8 = 0x00;
const TEST_RUNNING_ADDRESSES: [u16; 3] = [0x6001, 0x6002, 0x6003];
const TEST_RUNNING_BYTES: [u8; 3] = [0xDE, 0xB0, 0x61];
const RESET_DELAY: u32 = 1_000_000;

pub fn run_rom(rom_path: &str) {
    let mut nes = NES::new();
    nes.insert_cartdrige(rom_path).unwrap();

    let mut should_reset = false;
    let mut reset_delay = RESET_DELAY;

    // Run ROM
    'test: loop {
        // If 0x81 is in 0x6000, reset the NES after approximately 100 ms
        if should_reset {
            nes.reset();
            should_reset = false;
        }

        nes.clock();

        // Check if the data at 0x6000 has a valid value
        // This happens when 0x6001-0x6003 = [0xDE, 0xB0, 0x61]
        for i in 0..3 {
            if nes.read_memory_at(TEST_RUNNING_ADDRESSES[i]) != TEST_RUNNING_BYTES[i] {
                continue 'test;
            }
        }

        // 0x80 means the code is still processing
        let return_code = nes.read_memory_at(RETURN_CODE_ADDRESS);
        match return_code {
            0x80 => continue,
            0x81 => {
                if !should_reset {
                    reset_delay -= 1;
                    if reset_delay == 0 {
                        should_reset = true;
                        reset_delay = RESET_DELAY;
                    }
                }
            }
            _ => break,
        }
    }

    println!("{}", read_message(&mut nes));
    assert_eq!(nes.read_memory_at(RETURN_CODE_ADDRESS), 0x00);
}

fn read_message(nes: &mut NES) -> String {
    let mut msg_bytes = vec![];
    for i in MESSAGE_ADDRESS.. {
        let byte = nes.read_memory_at(i);
        if byte == END_OF_MESSAGE {
            break;
        }
        msg_bytes.push(byte);
    }
    String::from_utf8(msg_bytes).unwrap()
}

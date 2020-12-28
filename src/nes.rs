use crate::cpu::cpu::CPU;
use crate::bus::Bus;

#[derive(Debug)]
pub struct NES {
    pub bus: Bus,
    pub cpu: CPU
}

impl NES {
    pub const fn new() -> Self {
        NES {
            bus: Bus::new(),
            cpu: CPU::new()
        }
    }

    // Simulates the insertion of a NES cartridge
    pub fn insert_cartdrige(&mut self) {
        let test_code: [u8;28] = [
            0xA2,0x0A,0x8E,0x00,0x00,0xA2,0x03,0x8E,0x01,0x00,0xAC,0x00,0x00,0xA9,
            0x00,0x18,0x6D,0x01,0x00,0x88,0xD0,0xFA,0x8D,0x02,0x00,0xEA,0xEA,0xEA
        ];
        let cartridge_offset: u16 = 0x8000;
        let mut counter: u16 = 0;
        for inst in test_code.iter() {
            self.bus.write(cartridge_offset + counter, *inst);
            counter += 1;
        }
        self.bus.write(0xFFFC, 0x00);
        self.bus.write(0xFFFD, 0x80);
        println!("ram : {:?}",self.bus.ram[0x8000]);
    }

    // Resets the CPU and launches the game
    pub fn launch_game(&mut self) {
        self.cpu.reset();
        let mut counter = 200;
        while counter != 0 {
            self.cpu.clock();
            println!("{:?}",self.cpu);
            counter -= 1;
        }
        println!("{}",self.bus.ram[0x0002]);
        println!("DONE.");
    }
}
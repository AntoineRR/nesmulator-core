#[derive(Debug, Clone, Copy)]
pub struct Bus {
    pub ram: [u8;64*1024]
}

impl Bus {
    pub const fn new() -> Self {
        Bus {
            ram: [0;64*1024]
        }
    }

    // Read data from the bus at the specified address
    pub fn read(self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    // Write data to the bus at the specified address
    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[address as usize] = data;
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Controller {
    pub buffer: u8,
    shifter: u8,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            buffer: 0,
            shifter: 0,
        }
    }

    pub fn check_shifter(&mut self) -> u8 {
        let value: u8 = (self.shifter & 0x80 > 0) as u8;
        self.shifter <<= 1;
        value
    }

    pub fn update_shifter(&mut self) {
        self.shifter = self.buffer;
    }
}

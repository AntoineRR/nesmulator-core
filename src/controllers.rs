pub enum ControllerInput {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Down = 0b0000_0100,
    Up = 0b0000_1000,
    Start = 0b0001_0000,
    Select = 0b0010_0000,
    B = 0b0100_0000,
    A = 0b1000_0000
}

#[derive(Debug, Clone, Copy)]
pub struct Controller {
    pub buffer: u8,
    pub shifter: u8
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            buffer: 0,
            shifter: 0
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
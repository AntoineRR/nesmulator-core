const DMC_RATE: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

pub struct DMC {
    timer: u16,
    period: u16,

    output_level: u8,
}

impl DMC {
    pub fn new() -> Self {
        DMC {
            timer: 0,
            period: 0,

            output_level: 0,
        }
    }

    pub fn set_output_level(&mut self, value: u8) {
        self.output_level = value & 0x7F;
    }

    fn clock_sequencer(&mut self) {

    }

    pub fn clock(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.timer = self.period + 1;
            self.clock_sequencer();
        }
    }

    pub fn get_output(&self) -> u8 {
        self.output_level
    }
}

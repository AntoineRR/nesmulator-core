const LENGHT_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct WaveForm {
    lenght_counter: u8,
    lenght_counter_halt: bool,
}

impl WaveForm {
    pub fn new() -> Self {
        WaveForm {
            lenght_counter: 0,
            lenght_counter_halt: true,
        }
    }

    pub fn set_lenght_counter_halt(&mut self, lenght_counter_halt: bool) {
        self.lenght_counter_halt = lenght_counter_halt;
    }

    pub fn set_length_counter(&mut self, value: u8) {
        self.lenght_counter = LENGHT_TABLE[value as usize];
    }

    pub fn is_channel_silenced(&self) -> bool {
        self.lenght_counter == 0
    }

    pub fn clock(&mut self) {
        if !self.lenght_counter_halt {
            if self.lenght_counter != 0 {
                self.lenght_counter -= 1;
            }
        }
    }
}

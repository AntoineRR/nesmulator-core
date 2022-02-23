use serde::{Deserialize, Serialize};

const LENGHT_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Clone, Serialize, Deserialize)]
pub struct LengthCounter {
    length_counter: u8,
    length_counter_halt: bool,
    enabled: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        LengthCounter {
            length_counter: 0,
            length_counter_halt: true,
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length_counter = 0;
        }
        self.enabled = enabled;
    }

    pub fn set_lenght_counter_halt(&mut self, lenght_counter_halt: bool) {
        self.length_counter_halt = lenght_counter_halt;
    }

    pub fn set_length_counter(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = LENGHT_TABLE[value as usize];
        }
    }

    pub fn is_channel_silenced(&self) -> bool {
        self.length_counter == 0
    }

    pub fn clock(&mut self) {
        if !self.length_counter_halt && self.length_counter != 0 {
            self.length_counter -= 1;
        }
    }
}

pub struct Envelope {
    constant: bool,
    volume: u8,
    divider: u8,
    delay_level_counter: u8,
    pub start_flag: bool,
    pub loop_flag: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            constant: false,
            volume: 0,
            divider: 0,
            delay_level_counter: 0,
            start_flag: false,
            loop_flag: false,
        }
    }

    pub fn set_volume(&mut self, constant: bool, volume: u8) {
        self.constant = constant;
        self.volume = volume;
    }

    pub fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.delay_level_counter = 15;
            self.divider = self.volume;
        } else if self.divider != 0 {
            self.divider -= 1;
        } else {
            self.divider = self.volume;
            if self.delay_level_counter != 0 {
                self.delay_level_counter -= 1;
            } else if self.loop_flag {
                self.delay_level_counter = 15;
            }
        }
    }

    pub fn get_output(&self) -> u8 {
        if self.constant {
            self.volume
        } else {
            self.delay_level_counter
        }
    }
}

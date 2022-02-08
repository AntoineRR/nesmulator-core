use super::length_counter::LengthCounter;

const TRIANGLE_STEPS: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

pub struct Triangle {
    pub length_counter: LengthCounter,

    control: bool,
    linear_period: u8,
    linear_counter: u8,
    linear_reload: bool,

    timer: u16,
    period: u16,

    step: usize,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            length_counter: LengthCounter::new(),

            control: false,
            linear_period: 0,
            linear_counter: 0,
            linear_reload: false,

            timer: 0,
            period: 0,

            step: 0,
        }
    }

    pub fn reset(&mut self) {
        self.step = 0;
    }

    pub fn set_linear_counter(&mut self, value: u8) {
        self.control = value & 0x80 > 0;
        self.length_counter
            .set_lenght_counter_halt(value & 0x80 > 0);
        self.linear_period = value & 0x7F;
    }

    pub fn set_low_timer(&mut self, value: u8) {
        self.period = (self.period & 0xFF00) | (value as u16);
    }

    pub fn set_high_timer(&mut self, value: u8) {
        self.length_counter.set_length_counter((value & 0xF8) >> 3);
        self.period = (self.period & 0x00FF) | ((value & 0x07) as u16) << 8;
        self.linear_reload = true;
    }

    fn clock_sequencer(&mut self) {
        if self.step < 31 {
            self.step += 1;
        } else {
            self.step = 0;
        }
    }

    pub fn clock(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.timer = self.period + 1;
            if self.linear_counter != 0 && !self.length_counter.is_channel_silenced() {
                self.clock_sequencer();
            }
        }
    }

    pub fn clock_linear_counter(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_period;
        } else {
            if self.linear_counter != 0 {
                self.linear_counter -= 1;
            }
        }
        if !self.control {
            self.linear_reload = false;
        }
    }

    pub fn get_output(&self) -> u8 {
        if !self.length_counter.is_channel_silenced() {
            TRIANGLE_STEPS[self.step]
        } else {
            0
        }
    }
}

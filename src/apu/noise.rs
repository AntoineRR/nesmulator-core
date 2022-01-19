use super::{length_counter::LengthCounter, envelope::Envelope};

const NOISE_PERIOD: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

pub struct Noise {
    pub length_counter: LengthCounter,
    pub envelope: Envelope,
    shift: u16,
    mode: bool,

    timer: u16,
    period: u16,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),
            shift: 1,
            mode: false,

            timer: 0,
            period: 0,
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.length_counter.set_lenght_counter_halt(value & 0x20 > 0);
        self.envelope.loop_flag = value & 0x20 > 0;
        self.envelope.set_volume(value & 0x10 > 0, value & 0x0F);
    }

    pub fn set_period(&mut self, value: u8) {
        self.mode = value & 0x80 > 0;
        self.period = NOISE_PERIOD[(value & 0x0F) as usize];
    }

    pub fn set_length_counter(&mut self, value: u8) {
        self.length_counter.set_length_counter((value & 0xF8) >> 3);
        self.envelope.start_flag = true;
    }

    fn clock_shift(&mut self) {
        let feedback = (self.shift & 0x0001) ^
            if self.mode {
                (self.shift & 0x0020) >> 5
            } else {
                (self.shift & 0x0002) >> 1
            };
        self.shift = (self.shift >> 1) | (feedback << 14);
    }

    pub fn clock(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.timer = self.period + 1;
            self.clock_shift();
        }
    }

    pub fn get_output(&self) -> u8 {
        if self.shift & 0x01 > 0 && !self.length_counter.is_channel_silenced() {
            self.envelope.get_output()
        } else {
            0
        }
    }
}

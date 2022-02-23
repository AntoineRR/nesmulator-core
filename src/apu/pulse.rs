use serde::{Deserialize, Serialize};

use super::{envelope::Envelope, length_counter::LengthCounter, sweep::SweepUnit};

const DUTIES: [u8; 4] = [0b0000_0001, 0b0000_0011, 0b0000_1111, 0b1111_1100];

#[repr(usize)]
#[derive(Clone, Copy, Serialize, Deserialize)]
enum Duty {
    Wave125 = 0,
    Wave250 = 1,
    Wave500 = 2,
    WaveInv250 = 3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Pulse {
    pub length_counter: LengthCounter,
    pub sweep: SweepUnit,
    pub envelope: Envelope,

    duty: Duty,
    sequence: u8,

    period: u16,
    timer: u16,
}

impl Pulse {
    pub fn new(second: bool) -> Self {
        Pulse {
            length_counter: LengthCounter::new(),
            sweep: SweepUnit::new(second),
            envelope: Envelope::new(),

            duty: Duty::Wave125,
            sequence: 0,

            period: 0,
            timer: 0,
        }
    }

    pub fn set_control(&mut self, value: u8) {
        match (value & 0xC0) >> 6 {
            0 => self.duty = Duty::Wave125,
            1 => self.duty = Duty::Wave250,
            2 => self.duty = Duty::Wave500,
            3 => self.duty = Duty::WaveInv250,
            _ => unreachable!(),
        }
        self.sequence = DUTIES[self.duty as usize];
        self.length_counter
            .set_lenght_counter_halt(value & 0x20 > 0);
        self.envelope.loop_flag = value & 0x20 > 0;
        self.envelope.set_volume(value & 0x10 > 0, value & 0x0F);
    }

    pub fn set_sweep(&mut self, value: u8) {
        self.sweep.set(value);
    }

    pub fn set_low_timer(&mut self, value: u8) {
        self.period = (self.period & 0xFF00) | (value as u16);
    }

    pub fn set_high_timer(&mut self, value: u8) {
        self.length_counter.set_length_counter((value & 0xF8) >> 3);
        self.period = (self.period & 0x00FF) | ((value & 0x07) as u16) << 8;
        self.sequence = DUTIES[self.duty as usize];
        self.envelope.start_flag = true;
    }

    fn clock_sequencer(&mut self) {
        self.sequence = (self.sequence & 0xFE) >> 1 | (self.sequence & 0x01) << 7;
    }

    pub fn clock_sweep(&mut self) {
        self.sweep.clock(&mut self.period);
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
        if self.sequence & 0x01 > 0
            && !self.sweep.mute
            && !self.length_counter.is_channel_silenced()
            && self.period >= 8
            && self.period < 0x7FF
        {
            self.envelope.get_output()
        } else {
            0
        }
    }
}

use super::{dmc::DMC, noise::Noise, pulse::Pulse, triangle::Triangle};

const STEP_1: u64 = 3729;
const STEP_2: u64 = 7457;
const STEP_3: u64 = 11186;
const STEP_4: u64 = 14915;

pub struct APU {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: DMC,
    status: u8,
    frame_clock: u64,

    pulse_table: [f64; 31],
    tnd_table: [f64; 203],
}

impl APU {
    pub fn new() -> Self {
        let mut pulse_table = [0.0; 31];
        for i in 0..31 {
            pulse_table[i] = 95.52 / (8128.0 / i as f64 + 100.0);
        }

        let mut tnd_table = [0.0; 203];
        for i in 0..203 {
            tnd_table[i] = 163.67 / (24329.0 / i as f64 + 100.0);
        }

        APU {
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: DMC::new(),
            status: 0,
            frame_clock: 0,

            pulse_table,
            tnd_table,
        }
    }

    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0x4015 => self.status,
            _ => panic!("Invalid read on APU"),
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x4000 => self.pulse1.set_control(value),
            0x4001 => self.pulse1.set_sweep(value),
            0x4002 => self.pulse1.set_low_timer(value),
            0x4003 => self.pulse1.set_high_timer(value),
            0x4004 => self.pulse2.set_control(value),
            0x4005 => self.pulse2.set_sweep(value),
            0x4006 => self.pulse2.set_low_timer(value),
            0x4007 => self.pulse2.set_high_timer(value),
            0x4008 => (),
            0x4009 => (),
            0x400A => (),
            0x400B => (),
            0x400C => (),
            0x400D => (),
            0x400E => (),
            0x400F => (),
            0x4010 => (),
            0x4011 => (),
            0x4012 => (),
            0x4013 => (),
            0x4015 => (),
            _ => panic!("Invalid address given to APU: {:#X}", address),
        }
    }

    pub fn clock(&mut self) {
        if self.frame_clock == STEP_1 || self.frame_clock == STEP_3 {
            // Clock envelope and triangle linear counter
            self.pulse1.envelope.clock();
            self.pulse2.envelope.clock();
        }
        if self.frame_clock == STEP_2 || self.frame_clock == STEP_4 {
            // Clock envelope, triangle linear counter, lenght counter, and sweep units
            self.pulse1.envelope.clock();
            self.pulse2.envelope.clock();

            self.pulse1.waveform.clock();
            self.pulse2.waveform.clock();
            self.pulse1.clock_sweep();
            self.pulse2.clock_sweep();
        }

        self.frame_clock = self.frame_clock.wrapping_add(1);
        if self.frame_clock == STEP_4 + 1 {
            self.frame_clock = 0;
        }

        self.pulse1.clock();
        self.pulse2.clock();
    }

    pub fn get_current_amplitude(&self) -> f64 {
        let pulse_out = (self.pulse1.get_output() + self.pulse2.get_output()) as usize;
        self.pulse_table[pulse_out]
    }
}

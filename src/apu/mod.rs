pub mod state;

mod dmc;
mod envelope;
mod filters;
mod length_counter;
mod noise;
mod pulse;
mod sweep;
mod triangle;

use std::{cell::RefCell, error::Error, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    bus::Bus,
    cpu::{enums::Interrupt, Cpu},
    errors::{InvalidAPURegisterReadError, InvalidAPURegisterWriteError},
    state::Stateful,
};

use self::state::ApuState;

use {
    dmc::Dmc,
    filters::{Filter, HighPassFilter, LowPassFilter},
    noise::Noise,
    pulse::Pulse,
    triangle::Triangle,
};

const STEP_1: u64 = 7457;
const STEP_2: u64 = 14913;
const STEP_3: u64 = 22371;
const STEP_4: u64 = 29830;
const STEP_5: u64 = 37281;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Mode {
    Step4,
    Step5,
}

pub struct Apu {
    p_cpu: Option<Rc<RefCell<Cpu>>>,

    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    interrupt_inhibit: bool,
    frame_interrupt: bool,

    sample_rate: u64,
    frame_clock: u64,
    cycles_before_frame_clock_reset: Option<u64>,
    mode: Mode,
    instant_clock: bool,

    last_4017_value: u8,

    pulse_table: [f32; 31],
    tnd_table: [f32; 203],

    filters: [Box<dyn Filter>; 3],
}

impl Apu {
    pub fn new(ppu_clock_frequency: u64) -> Self {
        let clock_frequency = ppu_clock_frequency / 3;
        let sample_rate = clock_frequency as f32 / 44_100.0;

        let mut pulse_table = [0.0; 31];
        for (i, elt) in pulse_table.iter_mut().enumerate() {
            *elt = 95.52 / (8128.0 / i as f32 + 100.0);
        }

        let mut tnd_table = [0.0; 203];
        for (i, elt) in tnd_table.iter_mut().enumerate() {
            *elt = 163.67 / (24329.0 / i as f32 + 100.0);
        }

        Apu {
            p_cpu: None,

            pulse1: Pulse::new(false),
            pulse2: Pulse::new(true),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),

            interrupt_inhibit: false,
            frame_interrupt: false,

            sample_rate: sample_rate as u64,
            frame_clock: 0,
            cycles_before_frame_clock_reset: None,
            mode: Mode::Step4,
            instant_clock: false,

            last_4017_value: 0,

            pulse_table,
            tnd_table,

            filters: [
                Box::new(HighPassFilter::new(90, sample_rate)),
                Box::new(HighPassFilter::new(440, sample_rate)),
                Box::new(LowPassFilter::new(14000, sample_rate)),
            ],
        }
    }

    pub fn from_state(state: &ApuState, ppu_clock_frequency: u64) -> Self {
        let mut apu = Apu::new(ppu_clock_frequency);
        apu.set_state(state);
        apu
    }

    pub fn attach_bus_and_cpu(&mut self, p_bus: Rc<RefCell<Bus>>, p_cpu: Rc<RefCell<Cpu>>) {
        self.p_cpu = Some(p_cpu.clone());
        self.dmc.attach_bus_and_cpu(p_bus, p_cpu);
    }

    pub fn read_register(&mut self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x4015 => {
                let mut status: u8 = 0;
                status |= !self.pulse1.length_counter.is_channel_silenced() as u8;
                status |= (!self.pulse2.length_counter.is_channel_silenced() as u8) << 1;
                status |= (!self.triangle.length_counter.is_channel_silenced() as u8) << 2;
                status |= (!self.noise.length_counter.is_channel_silenced() as u8) << 3;
                status |= (self.dmc.is_active() as u8) << 4;
                status |= (self.frame_interrupt as u8) << 6;
                status |= (self.dmc.interrupt_flag as u8) << 7;
                self.frame_interrupt = false;
                Ok(status)
            }
            _ => Err(Box::new(InvalidAPURegisterReadError(address))),
        }
    }

    pub fn read_only_register(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x4015 => {
                let mut status: u8 = 0;
                status |= !self.pulse1.length_counter.is_channel_silenced() as u8;
                status |= (!self.pulse2.length_counter.is_channel_silenced() as u8) << 1;
                status |= (!self.triangle.length_counter.is_channel_silenced() as u8) << 2;
                status |= (!self.noise.length_counter.is_channel_silenced() as u8) << 3;
                status |= (self.dmc.is_active() as u8) << 4;
                status |= (self.frame_interrupt as u8) << 6;
                status |= (self.dmc.interrupt_flag as u8) << 7;
                Ok(status)
            }
            _ => Err(Box::new(InvalidAPURegisterReadError(address))),
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match address {
            0x4000 => self.pulse1.set_control(value),
            0x4001 => self.pulse1.set_sweep(value),
            0x4002 => self.pulse1.set_low_timer(value),
            0x4003 => self.pulse1.set_high_timer(value),
            0x4004 => self.pulse2.set_control(value),
            0x4005 => self.pulse2.set_sweep(value),
            0x4006 => self.pulse2.set_low_timer(value),
            0x4007 => self.pulse2.set_high_timer(value),
            0x4008 => self.triangle.set_linear_counter(value),
            0x4009 => (),
            0x400A => self.triangle.set_low_timer(value),
            0x400B => self.triangle.set_high_timer(value),
            0x400C => self.noise.set_control(value),
            0x400D => (),
            0x400E => self.noise.set_period(value),
            0x400F => self.noise.set_length_counter(value),
            0x4010 => self.dmc.set_rate(value),
            0x4011 => self.dmc.set_output_level(value),
            0x4012 => self.dmc.set_sample_address(value),
            0x4013 => self.dmc.set_sample_length(value),
            0x4015 => {
                self.pulse1.length_counter.set_enabled(value & 0x01 > 0);
                self.pulse2.length_counter.set_enabled(value & 0x02 > 0);
                self.triangle.length_counter.set_enabled(value & 0x04 > 0);
                self.noise.length_counter.set_enabled(value & 0x08 > 0);
                self.dmc.set_enabled(value & 0x10 > 0);
            }
            0x4017 => {
                self.last_4017_value = value;
                self.mode = match (value & 0x80) >> 7 {
                    0 => Mode::Step4,
                    1 => {
                        self.instant_clock = true;
                        Mode::Step5
                    }
                    _ => unreachable!(),
                };
                self.interrupt_inhibit = value & 0x40 > 0;
                if self.interrupt_inhibit {
                    self.frame_interrupt = false;
                }
                self.cycles_before_frame_clock_reset = Some(self.frame_clock % 2);
            }
            _ => return Err(Box::new(InvalidAPURegisterWriteError(address))),
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.write_register(0x4015, 0x00).unwrap();
        self.write_register(0x4017, self.last_4017_value).unwrap();
        self.frame_interrupt = false;
        self.triangle.reset();
        self.dmc.reset();
    }

    fn clock_quarter_frame(&mut self) {
        // Clock envelope and triangle linear counter
        self.pulse1.envelope.clock();
        self.pulse2.envelope.clock();
        self.noise.envelope.clock();
        self.triangle.clock_linear_counter();
    }

    fn clock_half_frame(&mut self) {
        // Clock envelope, triangle linear counter, lenght counter, and sweep units
        self.clock_quarter_frame();
        self.pulse1.length_counter.clock();
        self.pulse2.length_counter.clock();
        self.noise.length_counter.clock();
        self.triangle.length_counter.clock();
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    pub fn clock(&mut self) -> Option<f32> {
        if let Some(c) = self.cycles_before_frame_clock_reset {
            if c == 0 {
                self.cycles_before_frame_clock_reset = None;
                self.frame_clock = 0;
            } else {
                self.cycles_before_frame_clock_reset = Some(c - 1);
            }
        }

        if self.instant_clock {
            self.instant_clock = false;
            self.clock_half_frame();
            return None;
        }

        if self.frame_clock == STEP_1 || self.frame_clock == STEP_3 {
            self.clock_quarter_frame();
        }
        if self.frame_clock == STEP_2 {
            self.clock_half_frame();
        }

        if self.frame_clock == STEP_4 && self.mode == Mode::Step4 {
            self.clock_half_frame();
            if !self.interrupt_inhibit {
                self.frame_interrupt = true;
                if let Some(cpu) = &self.p_cpu {
                    cpu.borrow_mut().interrupt(Interrupt::Irq);
                } else {
                    panic!("No CPU attached to the APU");
                }
            }
            self.frame_clock = 0;
        } else if self.frame_clock == STEP_5 && self.mode == Mode::Step5 {
            self.clock_half_frame();
            self.frame_clock = 0;
        }

        if self.frame_clock % 2 == 0 {
            self.pulse1.clock();
            self.pulse2.clock();
            self.noise.clock();
        }
        self.triangle.clock();
        self.dmc.clock();

        self.frame_clock = self.frame_clock.wrapping_add(1);

        // Push the current amplitude to the sample buffer at a rate that is close to the 44100Hz required by sdl2
        // If we produce less samples, the sound will pop and it is horrible to the ear. Instead, producing
        // a bit to much samples may result in a lower tune, but it is better than poping sounds.
        if self.frame_clock % self.sample_rate as u64 == 0 {
            return Some(self.apply_filters(self.get_amplitude()));
        }

        None
    }

    fn get_amplitude(&self) -> f32 {
        let pulse_out = (self.pulse1.get_output() + self.pulse2.get_output()) as usize;
        let tnd_out = (3 * self.triangle.get_output()
            + 2 * self.noise.get_output()
            + self.dmc.get_output()) as usize;
        self.pulse_table[pulse_out] + self.tnd_table[tnd_out]
    }

    fn apply_filters(&mut self, amplitude: f32) -> f32 {
        self.filters
            .iter_mut()
            .fold(amplitude, |acc, filter| filter.process(acc))
    }
}

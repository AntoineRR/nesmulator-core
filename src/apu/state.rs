use serde::{Deserialize, Serialize};

use crate::state::Stateful;

use super::{dmc::Dmc, noise::Noise, pulse::Pulse, triangle::Triangle, Mode};

#[derive(Serialize, Deserialize)]
pub struct DmcState {
    pub interrupt_flag: bool,
    pub irq_enabled: bool,
    pub loop_flag: bool,
    pub sample_address: u16,
    pub sample_length: u16,
    pub sample_buffer: Option<u8>,
    pub current_address: u16,
    pub bytes_remaining: u16,
    pub silence_flag: bool,
    pub output_shift_register: u8,
    pub bits_remaining: u8,
    pub timer: u16,
    pub rate: u16,
    pub output_level: u8,
}

#[derive(Serialize, Deserialize)]
pub struct ApuState {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: DmcState,
    interrupt_inhibit: bool,
    frame_interrupt: bool,
    sample_rate: u64,
    frame_clock: u64,
    mode: Mode,
    instant_clock: bool,
    last_4017_value: u8,
}

impl Stateful for super::Apu {
    type State = ApuState;

    fn get_state(&self) -> Self::State {
        ApuState {
            pulse1: self.pulse1.clone(),
            pulse2: self.pulse2.clone(),
            triangle: self.triangle.clone(),
            noise: self.noise.clone(),
            dmc: self.dmc.get_state(),
            interrupt_inhibit: self.interrupt_inhibit,
            frame_interrupt: self.frame_interrupt,
            sample_rate: self.sample_rate,
            frame_clock: self.frame_clock,
            mode: self.mode.clone(),
            instant_clock: self.instant_clock,
            last_4017_value: self.last_4017_value,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.pulse1 = state.pulse1.clone();
        self.pulse2 = state.pulse2.clone();
        self.triangle = state.triangle.clone();
        self.noise = state.noise.clone();
        self.dmc = Dmc::from_state(&state.dmc);
        self.interrupt_inhibit = state.interrupt_inhibit;
        self.frame_interrupt = state.frame_interrupt;
        self.sample_rate = state.sample_rate;
        self.frame_clock = state.frame_clock;
        self.mode = state.mode.clone();
        self.instant_clock = state.instant_clock;
        self.last_4017_value = state.last_4017_value;
    }
}

use serde::{Deserialize, Serialize};

use crate::apu::state::ApuState;
use crate::bus::BusState;
use crate::cartridge::mapper::MapperState;
use crate::cpu::state::CpuState;
use crate::ppu::state::PpuState;

pub trait Stateful {
    type State;

    fn get_state(&self) -> Self::State;
    fn set_state(&mut self, state: &Self::State);
}

#[derive(Serialize, Deserialize)]
pub struct NesState {
    pub bus: BusState,
    pub cpu: CpuState,
    pub ppu: PpuState,
    pub apu: ApuState,
    pub mapper: Box<dyn MapperState>,
    pub total_clock: u64,
    pub dma_started: bool,
    pub dma_hi_address: u8,
    pub dma_base_address: u8,
    pub dma_address_offset: u8,
    pub dma_data: u8,
    pub add_samples: bool,
}

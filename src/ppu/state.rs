use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::state::Stateful;

use super::{
    bus::{PPUBus, VRAMAddress},
    oam::Oam,
    registers::Registers,
};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct PpuBusState {
    #[serde_as(as = "[[_; 0x0400]; 2]")]
    pub name_tables: [[u8; 0x0400]; 2],
    #[serde_as(as = "[_; 0x20]")]
    pub palette_table: [u8; 0x20],
    pub vram_address: VRAMAddress,
    pub tmp_vram_address: VRAMAddress,
}

#[derive(Serialize, Deserialize)]
pub struct PpuState {
    registers: Registers,
    pattern_table_shifters: [u16; 2],
    palette_shifters: [u16; 2],
    oam: Oam,
    next_sprite_count: u8,
    current_sprite_count: u8,
    next_contains_sprite_0: bool,
    current_contains_sprite_0: bool,
    sprite_shifters: [[u8; 2]; 8],
    sprite_x: [u8; 8],
    sprite_attributes: [u8; 8],
    next_name_table_byte: u8,
    next_attribute_table_byte: u8,
    next_low_background_byte: u8,
    next_high_background_byte: u8,
    is_sprite_0_rendered: bool,
    ppu_bus: PpuBusState,
    cycles: u16,
    scanline: u16,
    odd_frame: bool,
    total_clock: u64,
    is_frame_ready: bool,
}

impl Stateful for super::Ppu {
    type State = PpuState;

    fn get_state(&self) -> Self::State {
        PpuState {
            registers: self.registers.clone(),
            pattern_table_shifters: self.pattern_table_shifters,
            palette_shifters: self.palette_shifters,
            oam: self.oam.clone(),
            next_sprite_count: self.next_sprite_count,
            current_sprite_count: self.current_sprite_count,
            next_contains_sprite_0: self.next_contains_sprite_0,
            current_contains_sprite_0: self.current_contains_sprite_0,
            sprite_shifters: self.sprite_shifters,
            sprite_x: self.sprite_x,
            sprite_attributes: self.sprite_attributes,
            next_name_table_byte: self.next_name_table_byte,
            next_attribute_table_byte: self.next_attribute_table_byte,
            next_low_background_byte: self.next_low_background_byte,
            next_high_background_byte: self.next_high_background_byte,
            is_sprite_0_rendered: self.is_sprite_0_rendered,
            ppu_bus: self.ppu_bus.get_state(),
            cycles: self.cycles,
            scanline: self.scanline,
            odd_frame: self.odd_frame,
            total_clock: self.total_clock,
            is_frame_ready: self.is_frame_ready,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.registers = state.registers.clone();
        self.pattern_table_shifters = state.pattern_table_shifters;
        self.palette_shifters = state.palette_shifters;
        self.oam = state.oam.clone();
        self.next_sprite_count = state.next_sprite_count;
        self.current_sprite_count = state.current_sprite_count;
        self.next_contains_sprite_0 = state.next_contains_sprite_0;
        self.current_contains_sprite_0 = state.current_contains_sprite_0;
        self.sprite_shifters = state.sprite_shifters;
        self.sprite_x = state.sprite_x;
        self.sprite_attributes = state.sprite_attributes;
        self.next_name_table_byte = state.next_name_table_byte;
        self.next_attribute_table_byte = state.next_attribute_table_byte;
        self.next_low_background_byte = state.next_low_background_byte;
        self.next_high_background_byte = state.next_high_background_byte;
        self.is_sprite_0_rendered = state.is_sprite_0_rendered;
        self.ppu_bus = PPUBus::from_state(&state.ppu_bus);
        self.cycles = state.cycles;
        self.scanline = state.scanline;
        self.odd_frame = state.odd_frame;
        self.total_clock = state.total_clock;
        self.is_frame_ready = state.is_frame_ready;
    }
}

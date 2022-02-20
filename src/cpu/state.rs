use serde::{Deserialize, Serialize};

use crate::state::Stateful;

#[derive(Serialize, Deserialize)]
pub struct CpuState {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    p: u8,
    cycles: u8,
    require_add_cycle: bool,
    page_crossed: bool,
    total_clock: u64,
    display_logs: bool,
}

impl Stateful for super::Cpu {
    type State = CpuState;

    fn get_state(&self) -> Self::State {
        CpuState {
            a: self.a,
            x: self.x,
            y: self.y,
            pc: self.pc,
            sp: self.sp,
            p: self.p,
            cycles: self.cycles,
            require_add_cycle: self.require_add_cycle,
            page_crossed: self.page_crossed,
            total_clock: self.total_clock,
            display_logs: self.display_logs,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.a = state.a;
        self.x = state.x;
        self.y = state.y;
        self.pc = state.pc;
        self.sp = state.sp;
        self.p = state.p;
        self.cycles = state.cycles;
        self.require_add_cycle = state.require_add_cycle;
        self.page_crossed = state.page_crossed;
        self.total_clock = state.total_clock;
        self.display_logs = state.display_logs;
    }
}

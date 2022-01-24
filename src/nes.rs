// Represents the NES system

// ===== IMPORTS =====

use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use cartridge::cartridge::Cartridge;
use log::error;
use sdl2::audio::AudioSpecDesired;

use crate::ppu::ppu::PPU;
use crate::{apu::apu::APU, bus::Bus};
use crate::{
    cartridge,
    cpu::{cpu::CPU, enums::Interrupt},
};

// ===== MESSAGES =====

#[derive(PartialEq)]
pub enum Message {
    Input((usize, u8)),
    DrawFrame,
    ResizeWindow(u32, u32),
    ToggleDebugWindow,
}

const MIN_AUDIO_QUEUE_SIZE: u32 = 4 * 4410;

// ===== NES STRUCT =====

pub struct NES {
    // NES components
    p_bus: Rc<RefCell<Bus>>,
    p_cpu: Rc<RefCell<CPU>>,
    p_ppu: Rc<RefCell<PPU>>,
    p_apu: Rc<RefCell<APU>>,

    // Synchronization
    clock_duration_1000: Duration,
    elapsed: Duration,

    // NES clock counter
    total_clock: u64,

    // DMA variables
    dma_started: bool,
    dma_hi_address: u8,
    dma_base_address: u8,
    dma_address_offset: u8,
    dma_data: u8,

    // Audio
    add_samples: bool,
    samples: Vec<f32>,
}

unsafe impl Send for NES {}

impl NES {
    pub fn new(
        p_bus: Rc<RefCell<Bus>>,
        p_cpu: Rc<RefCell<CPU>>,
        p_ppu: Rc<RefCell<PPU>>,
        p_apu: Rc<RefCell<APU>>,
        ppu_clock_frequency: u64,
    ) -> Self {
        let clock_duration_1000 = Duration::from_micros(1_000_000_000 / ppu_clock_frequency);
        NES {
            p_bus,
            p_cpu,
            p_ppu,
            p_apu,

            clock_duration_1000,
            elapsed: Duration::new(0, 0),

            total_clock: 0,

            dma_started: false,
            dma_hi_address: 0,
            dma_base_address: 0,
            dma_address_offset: 0,
            dma_data: 0,

            add_samples: true,
            samples: vec![],
        }
    }

    // Simulates the insertion of a NES cartridge
    // Sets the mapper that is needed to read the data of the cartridge
    pub fn insert_cartdrige(&mut self, cartridge: Cartridge) {
        self.p_bus.borrow_mut().o_p_mapper = Some(cartridge.mapper.clone());
        self.p_ppu.borrow_mut().ppu_bus.o_p_mapper = Some(cartridge.mapper.clone());
    }

    // Resets the CPU and launches the game
    pub fn launch_game(&mut self, rx: Receiver<Message>) {
        self.p_cpu.borrow_mut().reset();
        self.total_clock = 0;

        // Sound
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_audio_specs = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(1024),
        };

        let queue = audio_subsystem
            .open_queue(None, &desired_audio_specs)
            .unwrap();
        queue.resume();

        loop {
            let time = Instant::now();

            // CPU and APU is clocked every 3 PPU cycles
            if self.total_clock % 3 == 0 {
                // If we initialized a DMA, do not clock CPU for nearly 513 cycles
                if self.p_ppu.borrow().registers.perform_dma {
                    self.perform_dma();
                } else {
                    self.p_cpu.borrow_mut().clock();
                }

                if let Some(s) = self.p_apu.borrow_mut().clock() {
                    if self.add_samples {
                        self.samples.push(s);
                    }
                }
            }

            // Clock PPU
            self.p_ppu.borrow_mut().clock();

            // Check if an NMI interrupt should be thrown
            if self.p_ppu.borrow().registers.emit_nmi {
                self.p_ppu.borrow_mut().registers.emit_nmi = false;
                self.p_cpu.borrow_mut().interrupt(Interrupt::NMI);
            }

            // Check inputs
            if let Ok(message) = rx.try_recv() {
                self.handle_message(message);
            }

            // Sound
            if !self.add_samples && queue.size() < MIN_AUDIO_QUEUE_SIZE {
                self.add_samples = true;
            } else if self.add_samples && queue.size() > MIN_AUDIO_QUEUE_SIZE {
                self.add_samples = false;
            }

            if self.total_clock % 89_489 == 0 {
                queue.queue(&self.samples[0..self.samples.len()]);
                self.samples.clear();
                self.add_samples = true;
            }

            self.total_clock = self.total_clock.wrapping_add(1);

            self.elapsed += time.elapsed();
            if self.total_clock % 1000 == 0 {
                if self.elapsed < self.clock_duration_1000 {
                    spin_sleep::sleep(self.clock_duration_1000 - self.elapsed);
                }
                self.elapsed = Duration::new(0, 0);
            }
        }
    }

    // Performs a DMA (transfer of 256 bytes of sprite data to PPU)
    fn perform_dma(&mut self) {
        if !self.dma_started {
            // Wait for an even cycle to start
            if self.total_clock % 2 == 1 {
                self.dma_hi_address = self.p_ppu.borrow().registers.oam_dma;
                self.dma_base_address = self.p_ppu.borrow().registers.oam_addr;
                self.dma_address_offset = 0;
                self.dma_started = true;
            }
        } else {
            // On even cycles, read data from the bus
            if self.total_clock % 2 == 0 {
                let address: u16 =
                    self.dma_address_offset as u16 + ((self.dma_hi_address as u16) << 8);
                self.dma_data = self.p_bus.borrow_mut().read(address);
            }
            // On odd cycles, write data to the PPU OAM
            else {
                self.p_ppu
                    .borrow_mut()
                    .write_register(0x2004, self.dma_data);

                if self.dma_address_offset < 255 {
                    self.dma_address_offset += 1;
                } else {
                    self.dma_address_offset = 0;
                }

                // End DMA
                if self.dma_address_offset == 0 {
                    self.dma_started = false;
                    self.p_ppu.borrow_mut().registers.perform_dma = false;
                }
            }
        }
    }

    fn handle_message(&mut self, message: Message) {
        if let Message::Input((id, input)) = message {
            self.p_bus.borrow_mut().controllers[id].buffer = 0;
            self.p_bus.borrow_mut().controllers[id].buffer |= input;
        } else if let Message::ResizeWindow(width, height) = message {
            self.p_ppu
                .borrow_mut()
                .gui
                .main_pixels
                .resize_surface(width, height);
        } else if message == Message::DrawFrame {
            self.p_ppu
                .borrow_mut()
                .gui
                .main_pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .unwrap();
        } else if message == Message::ToggleDebugWindow {
            self.p_ppu.borrow_mut().gui.toggle_debugging();
        }
    }
}

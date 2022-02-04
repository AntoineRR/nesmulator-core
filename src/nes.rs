// ===== IMPORTS =====

use std::cell::RefCell;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use log::info;

use crate::apu::apu::APU;
use crate::bus::Bus;
use crate::cartridge::cartridge::Cartridge;
use crate::cpu::{cpu::CPU, enums::Interrupt};
use crate::ppu::ppu::PPU;
use crate::utils::ARGBColor;
use crate::Config;

// ===== CONSTANTS =====

/// Frequency at which the PPU of a NTSC NES is clocked (Hz).
pub const PPU_CLOCK_FREQUENCY: u64 = 5_369_318;

// ===== NES STRUCT =====

/// Represent a NES. This will create the NES architecture and provide an API to run the emulation.
pub struct NES {
    // NES components
    p_bus: Rc<RefCell<Bus>>,
    p_cpu: Rc<RefCell<CPU>>,
    p_ppu: Rc<RefCell<PPU>>,
    p_apu: Rc<RefCell<APU>>,

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
    /// Create a NES using the default configuration.
    pub fn new() -> Self {
        let config = Config::default();

        let p_ppu = Rc::new(RefCell::new(PPU::new(config.palette_path)));
        let p_apu = Rc::new(RefCell::new(APU::new(PPU_CLOCK_FREQUENCY)));
        let p_bus = Rc::new(RefCell::new(Bus::new(p_ppu.clone(), p_apu.clone())));
        let p_cpu = Rc::new(RefCell::new(CPU::new(
            p_bus.clone(),
            config.display_cpu_logs,
        )));
        p_apu
            .borrow_mut()
            .attach_bus_and_cpu(p_bus.clone(), p_cpu.clone());

        NES {
            p_bus,
            p_cpu,
            p_ppu,
            p_apu,

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

    /// Create a NES using a custom configuration.
    pub fn from_config(config: Config) -> Self {
        let p_ppu = Rc::new(RefCell::new(PPU::new(config.palette_path)));
        let p_apu = Rc::new(RefCell::new(APU::new(PPU_CLOCK_FREQUENCY)));
        let p_bus = Rc::new(RefCell::new(Bus::new(p_ppu.clone(), p_apu.clone())));
        let p_cpu = Rc::new(RefCell::new(CPU::new(
            p_bus.clone(),
            config.display_cpu_logs,
        )));
        p_apu
            .borrow_mut()
            .attach_bus_and_cpu(p_bus.clone(), p_cpu.clone());

        NES {
            p_bus,
            p_cpu,
            p_ppu,
            p_apu,

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

    /// Load the ROM located at `rom_path` into the NES.
    /// The ROM file must be in a correct iNES or iNES v2 format.
    pub fn insert_cartdrige(&mut self, rom_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(rom_path);

        let cartridge = Cartridge::new(path)?;
        let p_mapper = Rc::new(RefCell::new(cartridge.mapper));

        self.p_bus.borrow_mut().set_mapper(p_mapper.clone());
        self.p_ppu.borrow_mut().set_mapper(p_mapper.clone());
        self.reset();

        info!("ROM {} successfully loaded.", rom_path);

        Ok(())
    }

    /// Reset the NES components.
    /// This will throw a reset interrupt at the NES emulated CPU.
    pub fn reset(&mut self) {
        self.p_cpu.borrow_mut().reset();
    }

    /// Read the bus memory at the given address
    /// You should know what you are doing when calling this method as it can easily
    /// be an invalid read
    pub fn read_memory_at(&mut self, address: u16) -> u8 {
        self.p_bus.borrow_mut().read(address)
    }

    /// Return if the NES is currently adding samples produced by the APU to the samples buffer.
    pub fn is_producing_samples(&self) -> bool {
        self.add_samples
    }

    /// Tell the NES if it should add the samples produced by the APU to the samples buffer.
    /// This is useful for audio synchronization purposes.
    pub fn produce_samples(&mut self, produce: bool) {
        self.add_samples = produce;
    }

    /// Gets the samples buffer and cleans it.
    pub fn get_samples(&mut self) -> Vec<f32> {
        let samples = self.samples.clone();
        self.samples.clear();
        samples
    }

    /// Get the Duration of 1000 clock cycles.
    /// The duration of a clock cycle is calculated based on the duration of a real NTSC clock cycle
    /// It is useful to get 1000 clock cycles duration instead of just one for synchronizing the emulation
    pub fn get_1000_clock_duration(&self) -> Duration {
        Duration::from_micros(1_000_000_000 / PPU_CLOCK_FREQUENCY)
    }

    /// Clock the NES for one PPU cycle.
    /// The CPU and the APU are clocked every 3 PPU cycles.
    /// This call may have to be delayed to achieve an emulation running at the desired speed.
    ///
    /// # Example
    ///
    /// For emulating the NES at the speed of a real NES, one might do the following:
    /// ```
    /// let target_time = nes.get_1000_clock_duration();
    /// let mut elapsed_time = Duration::new(0, 0);
    /// let mut clocks = 0;
    ///
    /// loop {
    ///     let time = Instant::now();
    ///     
    ///     // Run one clock of emulation
    ///     nes.clock();
    ///
    ///     // Get frame
    ///     // Synchronize sound
    ///     // Handle inputs
    ///
    ///     // Synchronize to emulate at the desired speed
    ///     elapsed_time += time.elapsed();
    ///     clocks += 1;
    ///     if clocks >= 1000 {
    ///         if elapsed_time < target_time {
    ///             spin_sleep::sleep(target_time - elapsed_time);
    ///         }
    ///         elapsed_time = Duration::new(0, 0);
    ///         clocks = 0;
    ///     }
    /// }
    /// ```
    pub fn clock(&mut self) {
        // CPU and APU are clocked every 3 PPU cycles
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

        // Check if an NMI interrupt should be thrown
        if self.p_ppu.borrow().registers.emit_nmi {
            self.p_ppu.borrow_mut().registers.emit_nmi = false;
            self.p_cpu.borrow_mut().interrupt(Interrupt::NMI);
        }

        // Clock PPU
        self.p_ppu.borrow_mut().clock();

        self.total_clock = self.total_clock.wrapping_add(1);
    }

    /// If a frame has been completely calculated, get the frame buffer and cleans it.
    /// Else this will return None.
    pub fn get_frame_buffer(&mut self) -> Option<[ARGBColor; 61_440]> {
        if self.p_ppu.borrow().is_frame_ready() {
            Some(self.p_ppu.borrow_mut().get_frame_buffer())
        } else {
            None
        }
    }

    /// Handle an input from the controller id.
    /// Will return an error if the id is not 0 or 1.
    pub fn input(&mut self, id: usize, input: u8) -> Result<(), Box<dyn Error>> {
        if id > 1 {
            Err("Controller id must be either 0 or 1")?;
        }
        self.p_bus.borrow_mut().set_input(id, input);
        Ok(())
    }

    /// Get the current pattern table.
    /// The number parameter allows to choose a pattern table.
    /// Will return an error if number is not 0 or 1.
    pub fn get_pattern_table(&self, number: u16) -> Result<[ARGBColor; 16384], Box<dyn Error>> {
        if number > 1 {
            Err("Pattern table number must be either 0 or 1")?;
        }
        self.p_ppu.borrow().get_pattern_table(number)
    }

    /// Get the colors of the palette that are currently stored in memory.
    pub fn get_palette(&self) -> Result<[ARGBColor; 32], Box<dyn Error>> {
        self.p_ppu.borrow().get_palette()
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
}

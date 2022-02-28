// ===== IMPORTS =====

use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::rc::Rc;
use std::time::Duration;

use log::debug;

use crate::apu::Apu;
use crate::bus::Bus;
use crate::cartridge::mapper::{get_mapper, Mapper};
use crate::cpu::{enums::Interrupt, Cpu};
use crate::ppu::Ppu;
use crate::state::{NesState, Stateful};
use crate::utils::ARGBColor;
use crate::Config;

// ===== CONSTANTS =====

/// Frequency at which the PPU of a NTSC NES is clocked (Hz).
pub const PPU_CLOCK_FREQUENCY: u64 = 5_369_318;

type MapperRc = Rc<RefCell<Box<dyn Mapper>>>;

// ===== NES STRUCT =====

/// Represent a NES. This will create the NES architecture and provide an API to run the emulation.
pub struct NES {
    // NES components
    p_bus: Rc<RefCell<Bus>>,
    p_cpu: Rc<RefCell<Cpu>>,
    p_ppu: Rc<RefCell<Ppu>>,
    p_apu: Rc<RefCell<Apu>>,

    // Mapper
    o_p_mapper: Option<MapperRc>,

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

    // Configuration
    config: Config,
}

unsafe impl Send for NES {}

impl Default for NES {
    fn default() -> Self {
        NES::new()
    }
}

impl NES {
    /// Create a NES using the default configuration.
    pub fn new() -> Self {
        let config = Config::default();

        NES::from_config(config)
    }

    /// Create a NES using a custom configuration.
    pub fn from_config(config: Config) -> Self {
        let p_ppu = Rc::new(RefCell::new(Ppu::new(&config.palette_path)));
        let p_apu = Rc::new(RefCell::new(Apu::new(PPU_CLOCK_FREQUENCY)));
        let p_bus = Rc::new(RefCell::new(Bus::new(p_ppu.clone(), p_apu.clone())));
        let p_cpu = Rc::new(RefCell::new(Cpu::new(
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

            o_p_mapper: None,

            total_clock: 0,

            dma_started: false,
            dma_hi_address: 0,
            dma_base_address: 0,
            dma_address_offset: 0,
            dma_data: 0,

            add_samples: true,
            samples: Vec::with_capacity(1024),

            config,
        }
    }

    /// Restart the NES. This is different from reseting it.
    pub fn restart(&mut self) {
        *self = NES::from_config(self.config.clone());
    }

    /// Load the ROM located at `rom_path` into the NES.
    /// The ROM file must be in a correct iNES or iNES v2 format.
    pub fn insert_cartdrige(&mut self, rom_path: &str) -> Result<(), Box<dyn Error>> {
        let mapper = get_mapper(rom_path)?;
        let p_mapper = Rc::new(RefCell::new(mapper));

        self.p_bus.borrow_mut().set_mapper(p_mapper.clone());
        self.p_ppu.borrow_mut().set_mapper(p_mapper.clone());
        self.o_p_mapper = Some(p_mapper.clone());
        self.reset();

        Ok(())
    }

    /// Reset the NES components.
    /// This will throw a reset interrupt at the NES emulated CPU.
    pub fn reset(&mut self) {
        self.p_cpu.borrow_mut().reset();
        self.p_apu.borrow_mut().reset();
    }

    /// Read the bus memory at the given address
    /// You should know what you are doing when calling this method as it can easily
    /// be an invalid read
    pub fn read_memory_at(&mut self, address: u16) -> Result<u8, Box<dyn Error>> {
        self.p_bus.borrow_mut().read(address)
    }

    /// Set the program counter of the CPU at a specific address
    /// You should know what you are doing when calling this method as it can easily
    /// result in a crash of the emulator
    pub fn set_program_counter_at(&mut self, address: u16) {
        self.p_cpu.borrow_mut().set_program_counter_at(address);
    }

    /// Set the palette to use for displaying the pattern tables
    pub fn set_debug_palette_id(&mut self, debug_palette_id: u8) -> Result<(), Box<dyn Error>> {
        if debug_palette_id > 7 {
            return Err("Palette id must be between 0 and 7".into());
        }
        self.p_ppu
            .borrow_mut()
            .set_debug_palette_id(debug_palette_id);
        Ok(())
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

    /// Get the Duration of a frame.
    pub fn get_one_frame_duration(&self) -> Duration {
        Duration::from_micros(1_000_000 / 60)
    }

    /// Clock the NES for one PPU cycle.
    /// The CPU and the APU are clocked every 3 PPU cycles.
    /// This call may have to be delayed to achieve an emulation running at the desired speed.
    ///
    /// # Example
    ///
    /// For emulating the NES at the speed of a real NES, one might do the following:
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use nes_emulator::nes::NES;
    ///
    /// let mut nes = NES::new();
    ///
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
    ///
    ///     // Break the loop when you want to stop emulation
    ///     break;
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
            self.p_cpu.borrow_mut().interrupt(Interrupt::Nmi);
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
            return Err("Controller id must be either 0 or 1".into());
        }
        self.p_bus.borrow_mut().set_input(id, input);
        Ok(())
    }

    /// Load a save in the ".sav" format.
    pub fn load_save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        if let Some(m) = &self.o_p_mapper {
            m.borrow_mut().load_persistent_memory(save_path)
        } else {
            Err("Insert a cartridge before trying to save".into())
        }
    }

    /// Save the game in the ".sav" format.
    pub fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        if let Some(m) = &self.o_p_mapper {
            m.borrow().save_persistent_memory(save_path)
        } else {
            Err("Insert a cartridge before trying to save".into())
        }
    }

    /// Load a NES state from a previously saved state.
    pub fn load_state(&mut self, state_path: &str, rom_path: &str) -> Result<(), Box<dyn Error>> {
        debug!("Loading NES state from {}...", state_path);
        let state_file = File::open(state_path)?;
        let state = &serde_json::from_reader(state_file)?;
        self.set_state(state);
        let mut mapper = get_mapper(rom_path)?;
        mapper.set_mapper_state(&state.mapper);
        let p_mapper = Rc::new(RefCell::new(mapper));

        self.p_bus.borrow_mut().set_mapper(p_mapper.clone());
        self.p_ppu.borrow_mut().set_mapper(p_mapper.clone());
        self.o_p_mapper = Some(p_mapper.clone());
        debug!("State successfully loaded.");
        Ok(())
    }

    /// Save the current state of the NES.
    pub fn save_state(&self, state_path: &str) -> Result<(), Box<dyn Error>> {
        debug!("Saving NES state...");
        let state = self.get_state();
        let state_file = File::create(state_path)?;
        serde_json::to_writer(state_file, &state)?;
        debug!("Current NES state saved in {}.", state_path);
        Ok(())
    }

    /// Get the current pattern table.
    /// The number parameter allows to choose a pattern table.
    /// Will return an error if number is not 0 or 1.
    pub fn get_pattern_table(&self, number: u16) -> Result<[ARGBColor; 16384], Box<dyn Error>> {
        if number > 1 {
            return Err("Pattern table number must be either 0 or 1".into());
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
                match self.p_bus.borrow_mut().read(address) {
                    Ok(data) => self.dma_data = data,
                    Err(e) => panic!("{}", e),
                }
            }
            // On odd cycles, write data to the PPU OAM
            else {
                self.p_ppu
                    .borrow_mut()
                    .write_register(0x2004, self.dma_data)
                    .unwrap();

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

impl Stateful for NES {
    type State = NesState;

    fn get_state(&self) -> Self::State {
        NesState {
            bus: self.p_bus.borrow().get_state(),
            cpu: self.p_cpu.borrow().get_state(),
            ppu: self.p_ppu.borrow().get_state(),
            apu: self.p_apu.borrow().get_state(),
            mapper: self
                .o_p_mapper
                .as_ref()
                .unwrap()
                .borrow()
                .get_mapper_state(),
            total_clock: self.total_clock,
            dma_started: self.dma_started,
            dma_hi_address: self.dma_hi_address,
            dma_base_address: self.dma_base_address,
            dma_address_offset: self.dma_address_offset,
            dma_data: self.dma_data,
            add_samples: self.add_samples,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.p_ppu = Rc::new(RefCell::new(Ppu::from_state(
            &state.ppu,
            &self.config.palette_path,
        )));
        self.p_apu = Rc::new(RefCell::new(Apu::from_state(
            &state.apu,
            PPU_CLOCK_FREQUENCY,
        )));
        self.p_bus = Rc::new(RefCell::new(Bus::from_state(
            &state.bus,
            self.p_ppu.clone(),
            self.p_apu.clone(),
        )));
        self.p_cpu = Rc::new(RefCell::new(Cpu::from_state(
            &state.cpu,
            self.p_bus.clone(),
            self.config.display_cpu_logs,
        )));
        self.p_apu
            .borrow_mut()
            .attach_bus_and_cpu(self.p_bus.clone(), self.p_cpu.clone());
        self.total_clock = state.total_clock;
        self.dma_started = state.dma_started;
        self.dma_hi_address = state.dma_hi_address;
        self.dma_base_address = state.dma_base_address;
        self.dma_address_offset = state.dma_address_offset;
        self.dma_data = state.dma_data;
        self.add_samples = state.add_samples;
    }
}

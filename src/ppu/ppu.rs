// Represents the PPU of the NES i.e. a component
// with a similar behaviour as the 2C02

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use crate::gui::GUI;

use super::{bus::PPUBus, enums::{ControlFlag, MaskFlag, StatusFlag, VRAMAddressMask}, palette::{ARGBColor, PALETTE}};

// ===== CONSTANTS =====

const MAX_CYCLES: u16 = 340;
const MAX_SCANLINES: u16 = 261;

// ===== STRUCT =====

#[derive(Debug)]
pub struct PPU {
    // PPU registers
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oam_addr: u8,
    pub oam_data: u8,
    pub scroll: u8,
    pub addr: u8,
    pub data: u8,
    pub oam_dma: u8,

    // decay register
    pub decay: u8,
    pub decay_timer: u32,

    // Data buffer for reading to 2007
    pub data_buffer: u8,

    // Background shifters ([0] => low bits, [1] => high bits)
    pub pattern_table_shifters: [u16;2],
    pub palette_shifters: [u16;2],

    // Data for the next 8 pixels
    pub next_name_table_byte: u8,
    pub next_attribute_table_byte: u8,
    pub next_low_background_byte: u8,
    pub next_high_background_byte: u8,

    // Addressing variables
    pub ppu_bus: PPUBus,
    pub fine_x: u8,
    pub w: bool, // First / Second write toggle

    // NMI
    pub emit_nmi: bool,

    // Variables required for display
    pub cycles: u16,
    pub scanline: u16,
    pub odd_frame: bool,

    // Required to check for inputs
    pub frame_ready: bool,

    pub total_clock: u32,

    // GUI
    pub p_gui: Arc<Mutex<GUI>>
}

impl PPU {
    pub fn new(p_gui: Arc<Mutex<GUI>>) -> Self {
        PPU {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
            oam_dma: 0,

            decay: 0,
            decay_timer: 0,

            data_buffer: 0,

            pattern_table_shifters: [0;2],
            palette_shifters: [0;2],

            next_name_table_byte: 0,
            next_attribute_table_byte: 0,
            next_low_background_byte: 0,
            next_high_background_byte: 0,

            ppu_bus: PPUBus::new(),
            fine_x: 0,
            w: false,

            emit_nmi: false,

            cycles: 0,
            scanline: 0,
            odd_frame: false,

            frame_ready: false,

            total_clock: 0,
            
            p_gui
        }
    }

    // ===== CLOCK =====

    // Executes a clock cycle
    pub fn clock(&mut self) {
        //println!("scanline : {}, cycles : {}, VBL : {}",self.scanline,self.cycles,(self.status & 0x80) != 0);

        // This cycle is skipped
        if self.scanline == 0 && self.cycles == 0 && self.odd_frame {
            self.cycles = 1;
        }

        // Get the next 8 pixels colors
        if self.scanline < 240 || self.scanline == 261 {
            if self.cycles >= 1 && self.cycles <= 257 || (self.cycles > 320 && self.cycles < 338) {
                self.update_shifters();
                // Get the nametable values
                if ((self.cycles - 1) % 8) == 0 {
                    self.load_next_background();
                    // Coarse x and Coarse y index the row and column in the name table
                    // That's why we mask the vram address to get those + the name table index
                    let address: u16 =
                        (self.ppu_bus.vram_address.address
                        & (VRAMAddressMask::CoarseXScroll as u16 | VRAMAddressMask::CoarseYScroll as u16 | VRAMAddressMask::NametableSelect as u16))
                        + 0x2000; // Name table address space offset
                    //println!("NT : {:#X}, {:#X}",address,self.ppu_bus.read(address));
                    self.next_name_table_byte = self.ppu_bus.read(address);
                }
                // Get the attribute table values
                else if ((self.cycles - 1) % 8) == 2 {
                    // One byte in attribute table represents 4 nametables
                    // We have to divide Coarse x and Coarse y by 4 to get the right index
                    let address: u16 =
                        (self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::CoarseXScroll) >> 2)
                        + ((self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::CoarseYScroll) >> 2) << 3)
                        + (self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::NametableSelect) << 10)
                        + 0x23C0; // Attribute table address space offset
                    self.next_attribute_table_byte = self.ppu_bus.read(address);
                    // We have the 4 areas in our next attribute table byte (top left, top right, bottom left and bottom right)
                    // We need to get the right palette value for our next 8 pixels
                    // This depends on the 2 lower bits of coarse Y and coarse X
                    if (self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::CoarseYScroll) & 0x02) > 0 {
                        self.next_attribute_table_byte >>= 4;
                    }
                    if (self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::CoarseXScroll) & 0x02) > 0 {
                        self.next_attribute_table_byte >>= 2;
                    }
                    // Only get the 2 lower bits
                    self.next_attribute_table_byte &= 0x03;
                    //println!("AT : {:#X}, {:#X} ; scanline : {}, cycle {}",address,self.next_attribute_table_byte,self.scanline,self.cycles);
                }
                // Get the low background tile byte
                else if ((self.cycles - 1) % 8) == 4 {
                    // The control flag decides if the data comes from the first or second pattern table
                    // Fine y choose the row
                    let address: u16 =
                        self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::FineY)
                        + ((self.next_name_table_byte as u16)<< 4)
                        + ((self.get_control_flag(ControlFlag::BackgroundPatternTableAddress) as u16) << 12);
                    //println!("LB : {:#X}, {:#X}",address,self.ppu_bus.read(address));
                    self.next_low_background_byte = self.ppu_bus.read(address);
                }
                // Get the high background tile byte
                else if ((self.cycles - 1) % 8) == 6 {
                    // Same as above with a +8 offset for choosing high bits
                    let address: u16 =
                        self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::FineY)
                        + ((self.next_name_table_byte as u16)<< 4)
                        + ((self.get_control_flag(ControlFlag::BackgroundPatternTableAddress) as u16) << 12)
                        + 8;
                    //println!("HB : {:#X}, {:#X}",address,self.ppu_bus.read(address));
                    self.next_high_background_byte = self.ppu_bus.read(address);
                }
                // Increment VRAM Address
                else if ((self.cycles - 1) % 8) == 7 {
                    self.increment_x();
                }
            }
            
            if self.cycles == 256 {
                self.increment_y();
            }
            
            if self.cycles == 257 {
                self.load_next_background();
                self.copy_tmp_x_to_vram_address();
            }

            // Unused read
            if self.cycles == 338 || self.cycles == 340 {
                self.next_name_table_byte = self.ppu_bus.read(0x2000 + (self.ppu_bus.vram_address.address & 0x0FFF));
            }
        }

        // Set the v blank flag at the beginning of the v blank period
        if self.scanline == 241 && self.cycles == 1 {
            self.set_status_flag(StatusFlag::VBlank, true);
            if self.get_control_flag(ControlFlag::VBlank) != 0 {
                self.emit_nmi = true;
            }
        }

        // Clear the v blank flag at the end of the v blank period
        if self.scanline == 261 && self.cycles == 1 {
            self.set_status_flag(StatusFlag::VBlank, false);
        }

        if self.scanline == 261 && (self.cycles > 279 && self.cycles < 305) {
            self.copy_tmp_y_to_vram_address();
        }

        // Set the color of one pixel
        if (self.scanline < 240) && (self.cycles >= 1 && self.cycles < 257) {
            let mut palette: u8 = 0;
            let mut color: u8 = 0;
            if self.get_mask_flag(MaskFlag::ShowBackground) {
                palette = self.get_shifter_value(self.palette_shifters);
                color = self.get_shifter_value(self.pattern_table_shifters);
            }
            self.p_gui
                .lock()
                .unwrap()
                .update_main_buffer(256*self.scanline as u32 + self.cycles as u32 - 1, self.get_pixel_color(palette, color));
        }

        // Increasing cycles and scanlines to reach a 341*262 matrix
        // Only the 256*240 matrix in the top left corner is used for displaying the screen
        self.total_clock += 1;
        self.cycles += 1;
        if self.cycles > MAX_CYCLES {
            self.scanline += 1;
            self.cycles = 0;
            if self.scanline > MAX_SCANLINES {
                self.scanline = 0;
                self.odd_frame = !self.odd_frame;
                self.frame_ready = true;
                // Debugging
                if self.p_gui.lock().unwrap().debug {
                    self.debug(); // Updates debug buffer to display pattern tables
                }
                // A frame is ready to be displayed
                self.p_gui.lock().unwrap().update();
            }
        }

        // Decay timer
        // Reset decay register after less than one second (5 350 000 clocks)
        self.decay_timer += 1;
        if self.decay_timer == 5000000 {
            self.decay = 0;
            self.decay_timer = 0;
        }
    }

    // ===== GET COLOR METHOD =====

    pub fn get_pixel_color(&self, palette: u8, color: u8) -> ARGBColor {
        let address: u16 = ((palette as u16) << 2) + (color as u16) + 0x3F00;
        PALETTE[(self.ppu_bus.read(address) & 0x3F) as usize]
    }

    // ===== REGISTERS METHODS =====

    // Writes value to one of the PPU registers
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => {
                self.ctrl = value;
                if self.get_status_flag(StatusFlag::VBlank) && (value & 0x80) == 0x80 {
                    self.emit_nmi = true;
                }
                self.ppu_bus.tmp_vram_address.set_address_part(VRAMAddressMask::NametableSelect, (value & 0x03) as u16);
            }
            0x2001 => self.mask = value,
            0x2002 => (),
            0x2003 => self.oam_addr = value,
            0x2004 => self.oam_data = value,
            0x2005 => {
                self.scroll = value;
                if self.w {
                    self.ppu_bus.tmp_vram_address.set_address_part(VRAMAddressMask::FineY, (value & 0x07) as u16);
                    self.ppu_bus.tmp_vram_address.set_address_part(VRAMAddressMask::CoarseYScroll, ((value & 0xF8) as u16) >> 3);
                    self.w = false;
                }
                else {
                    self.ppu_bus.tmp_vram_address.set_address_part(VRAMAddressMask::CoarseXScroll, ((value & 0xF8) as u16) >> 3);
                    self.fine_x = value & 0x07;
                    self.w = true;
                }
            }
            0x2006 => {
                self.addr = value;
                if self.w {
                    self.ppu_bus.tmp_vram_address.set_address_part(VRAMAddressMask::SW2006, value as u16);
                    self.ppu_bus.vram_address.address = self.ppu_bus.tmp_vram_address.address;
                    self.w = false;
                }
                else {
                    self.ppu_bus.tmp_vram_address.set_address_part(VRAMAddressMask::FW2006, (value & 0x3F) as u16);
                    self.ppu_bus.tmp_vram_address.address &= 0x3FFF; // Sets the 2 higher bits to 0
                    self.w = true;
                }
            }
            0x2007 => {
                self.data = value;
                self.ppu_bus.write(self.ppu_bus.vram_address.address & 0x3FFF, value);
                if self.get_control_flag(ControlFlag::VRAMAddressIncrement) == 0 {
                    self.ppu_bus.vram_address.address += 1; // Horizontal scrolling
                }
                else {
                    self.ppu_bus.vram_address.address += 32; // Vertical scrolling
                }
            }
            0x4014 => self.oam_dma = value,
            _ => panic!("Wrong address given to PPU : {:#x}",address)
        }
        self.decay = value;
        self.decay_timer = 0;
    }

    // Reads value from one of the PPU registers
    pub fn read_register(&mut self, address: u16) -> u8 {
        let mut value: u8;
        match address {
            0x2000 => value = self.decay,
            0x2001 => value = self.decay,
            0x2002 => {
                value = (self.status & 0xE0) | (self.decay & 0x1F);
                self.decay = value;
                self.set_status_flag(StatusFlag::VBlank, false);
                self.w = false;
            },
            0x2003 => value = self.decay,
            0x2004 => {
                value = self.oam_data;
                self.decay = value;
            }
            0x2005 => value = self.decay,
            0x2006 => value = self.decay,
            0x2007 => {
                // Read to 2007 is delayed by one read except for the palette
                value = self.data_buffer;
                self.data_buffer = self.ppu_bus.read(self.ppu_bus.vram_address.address);
                if self.ppu_bus.vram_address.address >= 0x3F00 {
                    value = (self.decay & 0xC0) | (self.data_buffer & 0x3F);
                    // Fill the buffer with the mirrored nametable "under" palette RAM
                    self.data_buffer = self.ppu_bus.read(self.ppu_bus.vram_address.address & 0x2FFF);
                }
                self.decay = value;
                // Increment VRAM Address
                if self.get_control_flag(ControlFlag::VRAMAddressIncrement) == 0 {
                    self.ppu_bus.vram_address.address += 1; // Horizontal scrolling
                }
                else {
                    self.ppu_bus.vram_address.address += 32; // Vertical scrolling
                }
            }
            0x4014 => panic!("4014 is not readable !"),
            _ => panic!("Wrong address given to PPU : {:#x}",address)
        }
        value
    }

    // Sets the flags for the status register
    pub fn set_status_flag(&mut self, flag: StatusFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        }
        else {
            self.status &= !(flag as u8);
        }
    }

    pub fn get_status_flag(&self, flag: StatusFlag) -> bool {
        (self.status & (flag as u8)) == (flag as u8)
    }

    // Get the flags from the control register
    pub fn get_control_flag(&mut self, flag: ControlFlag) -> u8 {
        if flag != ControlFlag::NametableAddress {
            ((self.ctrl & (flag as u8)) == (flag as u8)) as u8
        }
        else {
            (self.ctrl & 0x03) as u8 // Last two bits
        }
    }

    // Get the flags from the mask register
    pub fn get_mask_flag(&self, flag: MaskFlag) -> bool {
        (self.mask & (flag as u8)) == (flag as u8)
    }

    // ===== BACKGROUND SHIFTERS METHODS =====

    // Loads the next 8 bits inside the background shifters
    pub fn load_next_background(&mut self) {
        self.pattern_table_shifters[0] = (self.pattern_table_shifters[0] & 0xFF00) | (self.next_low_background_byte as u16);
        self.pattern_table_shifters[1] = (self.pattern_table_shifters[1] & 0xFF00) | (self.next_high_background_byte as u16);
        if (self.next_attribute_table_byte & 0x03) == 0x00 {
            self.palette_shifters[0] = (self.palette_shifters[0] & 0xFF00) | 0x0000;
            self.palette_shifters[1] = (self.palette_shifters[1] & 0xFF00) | 0x0000;
        }
        else if (self.next_attribute_table_byte & 0x03) == 0x01 {
            self.palette_shifters[0] = (self.palette_shifters[0] & 0xFF00) | 0x00FF;
            self.palette_shifters[1] = (self.palette_shifters[1] & 0xFF00) | 0x0000;
        }
        else if (self.next_attribute_table_byte & 0x03) == 0x02 {
            self.palette_shifters[0] = (self.palette_shifters[0] & 0xFF00) | 0x0000;
            self.palette_shifters[1] = (self.palette_shifters[1] & 0xFF00) | 0x00FF;
        }
        else if (self.next_attribute_table_byte & 0x03) == 0x03 {
            self.palette_shifters[0] = (self.palette_shifters[0] & 0xFF00) | 0x00FF;
            self.palette_shifters[1] = (self.palette_shifters[1] & 0xFF00) | 0x00FF;
        }
    }

    // Shifts the background shifters one bit left
    pub fn update_shifters(&mut self) {
        if self.get_mask_flag(MaskFlag::ShowBackground) {
            self.pattern_table_shifters[0] <<= 1;
            self.pattern_table_shifters[1] <<= 1;
            self.palette_shifters[0] <<= 1;
            self.palette_shifters[1] <<= 1;
        }
    }

    // Get the right value from the shifters
    pub fn get_shifter_value(&self, shifter: [u16;2]) -> u8 {
        let offset_mask: u16 = 0x8000 >> self.fine_x;
        let low: u8;
        if (shifter[0] & offset_mask) == 0 {
            low = 0;
        }
        else {
            low = 1;
        }
        let high: u8;
        if (shifter[1] & offset_mask) == 0 {
            high = 0;
        }
        else {
            high = 1;
        }
        low + (high << 1)
    }

    // ===== VRAM ADDRESS MODIFICATION METHODS =====

    // Increments the VRAM address to point to the next 8 bits to render
    pub fn increment_x(&mut self) {
        if self.get_mask_flag(MaskFlag::ShowSprites) || self.get_mask_flag(MaskFlag::ShowBackground) {
            let x: u16 = self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::CoarseXScroll);
            if x == 31 {
                self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseXScroll, 0);
                let nametable_x: u16 = self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::NametableX);
                self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::NametableX, (nametable_x == 0) as u16);
            }
            else {
                self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseXScroll, x + 1);
            }
        }
    }

    pub fn increment_y(&mut self) {
        if self.get_mask_flag(MaskFlag::ShowSprites) || self.get_mask_flag(MaskFlag::ShowBackground) {
            let y: u16 = self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::FineY);
            if y >= 7 {
                self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::FineY, 0);
                let c_y: u16 = self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::CoarseYScroll);
                if c_y == 29 {
                    self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseYScroll, 0);
                    let nametable_y: u16 = self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::NametableY);
                    self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::NametableY, (nametable_y == 0) as u16);
                }
                else if c_y == 31 {
                    self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseYScroll, 0);
                }
                else {
                    self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseYScroll, c_y + 1);
                }
            }
            else {
                self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::FineY, y + 1);
            }
        }
    }

    pub fn copy_tmp_x_to_vram_address(&mut self) {
        if self.get_mask_flag(MaskFlag::ShowSprites) || self.get_mask_flag(MaskFlag::ShowBackground) {
            // Set coarse x to tmp value
            let tmp_c_x: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::CoarseXScroll);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseXScroll, tmp_c_x);
            // Set nametable x to tmp value
            let tmp_nt_x: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::NametableX);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::NametableX, tmp_nt_x);
        }
    }

    pub fn copy_tmp_y_to_vram_address(&mut self) {
        if self.get_mask_flag(MaskFlag::ShowSprites) || self.get_mask_flag(MaskFlag::ShowBackground) {
            // Set coarse y to tmp value
            let tmp_c_y: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::CoarseYScroll);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseYScroll, tmp_c_y);
            // Set fine y to tmp value
            let tmp_f_y: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::FineY);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::FineY, tmp_f_y);
            // set nametable y to tmp value
            let tmp_nt_y: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::NametableY);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::NametableY, tmp_nt_y);
        }
    }

    // ===== DEBUGGING =====

    pub fn debug(&self) {
        self.display_pattern_table(0);
        self.display_pattern_table(1);
        self.display_separation();
        self.display_palette();
    }

    pub fn display_pattern_table(&self, number: u16) {
        for n_tile_y in 0..16 {
            for n_tile_x in 0..16 {
                self.display_tile(n_tile_y, n_tile_x, number);
            }
        }
    }

    pub fn display_tile(&self, n_tile_y: u16, n_tile_x: u16, number: u16) {
        let n_offset = n_tile_y*256 + n_tile_x*16;
        for row in 0..8 {
            let mut tile_low: u8 = self.ppu_bus.read(number*0x1000 + n_offset + row);
            let mut tile_high: u8 = self.ppu_bus.read(number*0x1000 + n_offset + row + 0x0008);
            for col in 0..8 {
                let color: u8 = (tile_low & 0x01) + (tile_high & 0x01);
                tile_high >>= 1;
                tile_low >>= 1;
                let c: ARGBColor = self.get_pixel_color(0, color);
                self.p_gui.lock().unwrap().update_debug_buffer((n_tile_x*8+(7-col) + number*128 + (n_tile_y*8+row)*256) as u32, c);
            }
        }
    }

    pub fn display_separation(&self) {
        for i in 0..512 {
            self.p_gui.lock().unwrap().update_debug_buffer(
                256*128 + i,
                ARGBColor::new(255, 50, 50, 50)
            );
        }
    }

    pub fn display_palette(&self) {
        for address in 0x3F00..0x3F20 {
            let offset = address & 0x00FF;
            for i in 0..6 {
                for j in 0..6 {
                    let index = 258*128 + (offset * 6) + (((offset % 4) == 0) as u32)*2 + i + j*256;
                    self.p_gui
                        .lock()
                        .unwrap()
                        .update_debug_buffer(index, PALETTE[(self.ppu_bus.read(address as u16) & 0x3F) as usize]);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn read_register_without_modification(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x2000 => value = self.ctrl,
            0x2001 => value = self.mask,
            0x2002 => value = self.status,
            0x2003 => value = self.oam_addr,
            0x2004 => value = self.oam_data,
            0x2005 => value = self.scroll,
            0x2006 => value = self.addr,
            0x2007 => value = self.data_buffer,
            0x4014 => value = self.oam_dma,
            _ => panic!("Wrong address given to PPU : {:#x}",address)
        }
        value
    }
}
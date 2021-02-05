// Represents the PPU of the NES i.e. a component
// with a similar behaviour as the 2C02

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use crate::gui::GUI;

use super::{bus::PPUBus, enums::{ControlFlag, MaskFlag, SpriteAttribute, StatusFlag, VRAMAddressMask}, oam::OAM, palette::{ARGBColor, PALETTE}, registers::Registers};

// ===== CONSTANTS =====

const MAX_CYCLES: u16 = 340;
const MAX_SCANLINES: u16 = 261;

// ===== STRUCT =====

pub struct PPU {
    // PPU registers
    pub registers: Registers,

    // Background shifters ([0] => low bits, [1] => high bits)
    pub pattern_table_shifters: [u16;2],
    pub palette_shifters: [u16;2],

    // OAM
    pub oam: OAM,

    pub next_sprite_count: u8,
    pub current_sprite_count: u8,
    pub next_contains_sprite_0: bool,
    pub current_contains_sprite_0: bool,

    // Variables for displaying sprites
    pub sprite_shifters: [[u8;2];8],
    pub sprite_x: [u8;8],
    pub sprite_attributes: [u8;8],

    // Sprite evaluation variables
    pub eval_index: u8,
    pub eval_data: u8,
    pub eval_secondary_index: u8,
    pub copy_sprite: bool,

    // Data for the next 8 pixels
    pub next_name_table_byte: u8,
    pub next_attribute_table_byte: u8,
    pub next_low_background_byte: u8,
    pub next_high_background_byte: u8,
    pub is_sprite_0_rendered: bool,

    // Addressing variables
    pub ppu_bus: PPUBus,

    // Variables required for display
    pub cycles: u16,
    pub scanline: u16,
    pub odd_frame: bool,

    pub total_clock: u64,

    // GUI
    pub p_gui: Arc<Mutex<GUI>>
}

impl PPU {
    pub fn new(p_gui: Arc<Mutex<GUI>>) -> Self {
        PPU {
            registers: Registers::new(),

            pattern_table_shifters: [0;2],
            palette_shifters: [0;2],

            oam: OAM::new(),

            next_sprite_count: 0,
            current_sprite_count: 0,
            next_contains_sprite_0: false,
            current_contains_sprite_0: false,
            is_sprite_0_rendered: false,

            sprite_shifters: [[0;2];8],
            sprite_x: [0;8],
            sprite_attributes: [0;8],

            eval_index: 0,
            eval_data: 0,
            eval_secondary_index: 0,
            copy_sprite: false,

            next_name_table_byte: 0,
            next_attribute_table_byte: 0,
            next_low_background_byte: 0,
            next_high_background_byte: 0,

            ppu_bus: PPUBus::new(),

            cycles: 0,
            scanline: 0,
            odd_frame: false,

            total_clock: 0,
            
            p_gui
        }
    }

    // ===== CLOCK =====

    // Executes a clock cycle
    pub fn clock(&mut self) {

        // This cycle is skipped
        if self.scanline == 0 && self.cycles == 0 && self.odd_frame {
            if self.registers.get_mask_flag(MaskFlag::ShowBackground) {
                self.cycles = 1;
            }
        }

        // Get the next 8 pixels colors
        if self.scanline < 240 || self.scanline == 261 {

            // === BACKGROUND ===

            if self.cycles >= 2 && self.cycles <= 257 || (self.cycles > 320 && self.cycles < 338) {
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
                }
                // Get the low background tile byte
                else if ((self.cycles - 1) % 8) == 4 {
                    // The control flag decides if the data comes from the first or second pattern table
                    // Fine y choose the row
                    let address: u16 =
                        self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::FineY)
                        + ((self.next_name_table_byte as u16)<< 4)
                        + ((self.registers.get_control_flag(ControlFlag::BackgroundPatternTableAddress) as u16) << 12);
                    self.next_low_background_byte = self.ppu_bus.read(address);
                }
                // Get the high background tile byte
                else if ((self.cycles - 1) % 8) == 6 {
                    // Same as above with a +8 offset for choosing high bits
                    let address: u16 =
                        self.ppu_bus.vram_address.get_address_part(VRAMAddressMask::FineY)
                        + ((self.next_name_table_byte as u16)<< 4)
                        + ((self.registers.get_control_flag(ControlFlag::BackgroundPatternTableAddress) as u16) << 12)
                        + 8;
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
                self.next_name_table_byte = self.ppu_bus.read(
                    0x2000
                    + (self.ppu_bus.vram_address.address & 0x0FFF));
            }

            // === SPRITES ===

            if self.cycles == 0 {
                self.current_sprite_count = self.next_sprite_count;
                self.current_contains_sprite_0 = self.next_contains_sprite_0;
            }

            // Initializes secondary OAM with FF
            if self.cycles > 0 && self.cycles < 65 {
                if self.cycles % 2 == 1 {
                    self.oam.write_secondary(((self.cycles - 1) / 2)  as u8, 0xFF);
                }
            }

            // Sprite evaluation
            if self.cycles > 64 && self.cycles < 257 {
                if self.cycles == 65 {
                    self.next_sprite_count = 0;
                    self.next_contains_sprite_0 = false;
                }
                self.evaluate_sprites();
            }

            // Sprite data fetch
            if self.cycles > 256 && self.cycles < 321 {
                if self.cycles == 257 {
                    self.sprite_shifters = [[0;2];8];
                    self.sprite_x = [0;8];
                    self.sprite_attributes = [0;8];
                }
                self.fetch_sprite_data();
            }
        }

        // Set the v blank flag at the beginning of the v blank period
        if self.scanline == 241 && self.cycles == 1 {
            self.registers.set_status_flag(StatusFlag::VBlank, true);
            if self.registers.get_control_flag(ControlFlag::VBlank) != 0 {
                self.registers.emit_nmi = true;
            }
        }

        // Clear the v blank flag at the end of the v blank period
        if self.scanline == 261 && self.cycles == 1 {
            self.registers.set_status_flag(StatusFlag::VBlank, false);
            self.registers.set_status_flag(StatusFlag::Sprite0Hit, false);
            self.registers.set_status_flag(StatusFlag::SpriteOverflow, false);
        }

        if self.scanline == 261 && (self.cycles > 279 && self.cycles < 305) {
            self.copy_tmp_y_to_vram_address();
        }

        // Set the color of one pixel
        if (self.scanline < 240) && (self.cycles >= 1 && self.cycles < 257) {

            // Calculates background color
            let mut bg_palette: u8 = 0;
            let mut bg_pattern: u8 = 0;
            if self.registers.get_mask_flag(MaskFlag::ShowBackground) {
                bg_palette = self.get_shifter_value(self.palette_shifters);
                bg_pattern = self.get_shifter_value(self.pattern_table_shifters);
            }

            let mut fg_palette: u8 = 0;
            let mut fg_pattern: u8 = 0;
            let mut fg_priority: bool = false;
            // Calculates foreground (sprite) color
            if self.registers.get_mask_flag(MaskFlag::ShowSprites) {
                self.is_sprite_0_rendered = false;
                for i in 0..self.current_sprite_count {
                    if self.sprite_x[i as usize] == 0 {
                        fg_palette = (self.sprite_attributes[i as usize] & 0x03) + 0x04;
                        fg_pattern = self.get_sprite_shifters_value(i as usize);
                        fg_priority = (self.sprite_attributes[i as usize] & (SpriteAttribute::Priority as u8)) == 0;

                        if fg_pattern !=0 {
                            if i ==0 {
                                self.is_sprite_0_rendered = true;
                            }
                            break;
                        }
                    }
                }
            }

            // Calculates the final pixel color
            let palette: u8;
            let pattern: u8;
            if bg_pattern == 0 && fg_pattern == 0 {
                palette = 0;
                pattern = 0;
            }
            else if bg_pattern == 0 && fg_pattern != 0 {
                palette = fg_palette;
                pattern = fg_pattern;
            }
            else if bg_pattern != 0 && fg_pattern == 0 {
                palette = bg_palette;
                pattern = bg_pattern;
            }
            else {
                if fg_priority {
                    palette = fg_palette;
                    pattern = fg_pattern;
                }
                else {
                    palette = bg_palette;
                    pattern = bg_pattern;
                }

                // Detect sprite 0 hit
                if self.current_contains_sprite_0 && self.is_sprite_0_rendered {
                    if self.registers.get_mask_flag(MaskFlag::ShowBackground)
                        && self.registers.get_mask_flag(MaskFlag::ShowSprites) {
                        if self.cycles != 256 {
                            if !self.registers.get_mask_flag(MaskFlag::ShowLeftScreenBackground)
                                || !self.registers.get_mask_flag(MaskFlag::ShowLeftScreenSprites) {
                                if self.cycles >= 9 {
                                    self.registers.set_status_flag(StatusFlag::Sprite0Hit, true);
                                }
                            }
                            else {
                                self.registers.set_status_flag(StatusFlag::Sprite0Hit, true);
                            }
                        }
                    }
                }
            }

            // Renders pixel
            self.p_gui
                .lock()
                .unwrap()
                .update_main_buffer(
                    (256*self.scanline as u32 + self.cycles as u32 - 1) as usize,
                    self.get_pixel_color(palette, pattern)
                );
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
        self.registers.decay_timer += 1;
        if self.registers.decay_timer == 5000000 {
            self.registers.decay = 0;
            self.registers.decay_timer = 0;
        }
    }

    // ===== GET COLOR METHOD =====

    pub fn get_pixel_color(&self, palette: u8, color: u8) -> ARGBColor {
        let address: u16 = ((palette as u16) << 2) + (color as u16) + 0x3F00;
        PALETTE[(self.ppu_bus.read(address) & 0x3F) as usize]
    }

    // ===== REGISTERS METHODS =====

    pub fn write_register(&mut self, address: u16, value: u8) {
        self.registers.write_register(&mut self.ppu_bus, &mut self.oam, address, value);
    }

    pub fn read_register(&mut self, address: u16) -> u8 {
        self.registers.read_register(&mut self.ppu_bus, &self.oam, address)
    }

    // ===== SPRITE RELATED METHODS =====

    // Performs the sprite evaluation for the next scanline
    // This is not cycle accurate with a real NES
    pub fn evaluate_sprites(&mut self) {
        // 3 cycles are available for each 64 sprites
        if (self.cycles - 65) % 3 == 0 {
            let sprite_index: usize = ((self.cycles - 65) / 3) as usize;
            let sprite_size: u16;
            match self.registers.get_control_flag(ControlFlag::SpriteSize) {
                0 => sprite_size = 8,
                1 => sprite_size = 16,
                _ => panic!("Invalid sprite size value")
            }
            // If the sprite should appear on the next scanline
            if self.scanline % 261 >= (self.oam.primary[sprite_index].y as u16)
                && self.scanline % 261 < (self.oam.primary[sprite_index].y as u16) + sprite_size {
                // If more than 8 sprites has been found
                if self.next_sprite_count >= 8 {
                    if self.registers.get_mask_flag(MaskFlag::ShowSprites) || self.registers.get_mask_flag(MaskFlag::ShowBackground) {
                        self.registers.set_status_flag(StatusFlag::SpriteOverflow, true);
                    }
                }
                else {
                    if self.scanline != 261 {
                        self.oam.secondary[self.next_sprite_count as usize] = self.oam.primary[sprite_index];
                        if sprite_index == 0 {
                            self.next_contains_sprite_0 = true;
                        }
                        self.next_sprite_count += 1;
                    }
                }
            }
        }

        // The first empty entry in the secondary OAM has the 63 sprite y as its y coordinate
        if self.cycles == 256 {
            if self.next_sprite_count < 8 {
                self.oam.write_secondary(self.next_sprite_count*4, self.oam.primary[63].y);
            }
        }
    }

    // Populates the sprite shifters with the data required for next scanline
    // This doesn't work exactly as in a real NES
    pub fn fetch_sprite_data(&mut self) {
        let sprite_index: usize = ((self.cycles - 257) / 8) as usize;
        if (sprite_index as u8) < self.next_sprite_count {
            match (self.cycles - 257) % 8 {
                // Populate sprite shifters
                0 => {
                    let lo_address: u16; // Address of the low byte of the sprite
                    let v_flip: bool = self.oam.secondary[sprite_index].get_attribute_flag(SpriteAttribute::FlipVertically) == 1;
                    // 8x8 sprites
                    if self.registers.get_control_flag(ControlFlag::SpriteSize) == 0 {
                        // Do not flip sprite vertically
                        if !v_flip {
                            lo_address = ((self.registers.get_control_flag(ControlFlag::SpritePatternTableAddress) as u16) << 12)
                                | ((self.oam.secondary[sprite_index].id as u16) << 4)
                                | ((self.scanline as i16 - (self.oam.secondary[sprite_index].y as i16))) as u16;
                        }
                        // Flip sprite vertically
                        else {
                            lo_address = ((self.registers.get_control_flag(ControlFlag::SpritePatternTableAddress) as u16) << 12)
                                | ((self.oam.secondary[sprite_index].id as u16) << 4)
                                | ((7 - (self.scanline as i16 - (self.oam.secondary[sprite_index].y as i16)))) as u16;
                        }
                    }
                    // 8x16 sprites
                    else {
                        // Do not flip sprite vertically
                        if !v_flip {
                            // First half of the sprite
                            if self.scanline - (self.oam.secondary[sprite_index].y as u16) < 8 {
                                lo_address = (((self.oam.secondary[sprite_index].id & 0x01) as u16) << 12)
                                    | (((self.oam.secondary[sprite_index].id & 0xFE) as u16) << 4)
                                    | ((self.scanline as i16 - (self.oam.secondary[sprite_index].y as i16)) & 0x07) as u16;
                            }
                            // Second half of the sprite
                            else {
                                lo_address = (((self.oam.secondary[sprite_index].id & 0x01) as u16) << 12)
                                    | ((((self.oam.secondary[sprite_index].id & 0xFE) as u16) + 1) << 4)
                                    | ((self.scanline as i16 - (self.oam.secondary[sprite_index].y as i16)) & 0x07) as u16;
                            }
                        }
                        // Flip sprite vertically
                        else {
                            // Second half of the sprite
                            if self.scanline - (self.oam.secondary[sprite_index].y as u16) < 8 {
                                lo_address = (((self.oam.secondary[sprite_index].id & 0x01) as u16) << 12)
                                    | ((((self.oam.secondary[sprite_index].id & 0xFE) as u16) + 1) << 4)
                                    | ((7 - (self.scanline as i16 - (self.oam.secondary[sprite_index].y as i16))) & 0x07) as u16;
                            }
                            // First half of the sprite
                            else {
                                lo_address = (((self.oam.secondary[sprite_index].id & 0x01) as u16) << 12)
                                    | (((self.oam.secondary[sprite_index].id & 0xFE) as u16) << 4) as u16
                                    | (7 - ((self.scanline as i16 - (self.oam.secondary[sprite_index].y as i16)) & 0x07)) as u16;
                            }
                        }
                    }

                    // Get low and high bytes of the sprite
                    let mut lo_sprite: u8 = self.ppu_bus.read(lo_address);
                    let mut hi_sprite: u8 = self.ppu_bus.read(lo_address + 8);

                    // Flip horizontally
                    if self.oam.secondary[sprite_index].get_attribute_flag(SpriteAttribute::FlipHorizontally) == 1 {
                        lo_sprite = ((lo_sprite & 0xF0) >> 4) | ((lo_sprite & 0x0F) << 4);
                        lo_sprite = ((lo_sprite & 0xCC) >> 2) | ((lo_sprite & 0x33) << 2);
                        lo_sprite = ((lo_sprite & 0xAA) >> 1) | ((lo_sprite & 0x55) << 1);
    
                        hi_sprite = ((hi_sprite & 0xF0) >> 4) | ((hi_sprite & 0x0F) << 4);
                        hi_sprite = ((hi_sprite & 0xCC) >> 2) | ((hi_sprite & 0x33) << 2);
                        hi_sprite = ((hi_sprite & 0xAA) >> 1) | ((hi_sprite & 0x55) << 1);
                    }
    
                    // Finally write the result into our shifters
                    self.sprite_shifters[sprite_index][0] = lo_sprite;
                    self.sprite_shifters[sprite_index][1] = hi_sprite;
                }
                // Populate X sprite shifters
                1 => {
                    self.sprite_x[sprite_index] = self.oam.secondary[sprite_index].x;
                }
                // Populate sprite attributes
                2 => {
                    self.sprite_attributes[sprite_index] = self.oam.secondary[sprite_index].attribute;
                }
                3..=7 => (),
                _ => panic!("Unreachable pattern")
            }
        }
    }

    pub fn get_sprite_shifters_value(&self, sprite_index: usize) -> u8 {
        let offset_mask: u8 = 0x80;
        let low: u8 = ((self.sprite_shifters[sprite_index][0] & offset_mask) > 0) as u8;
        let high: u8 = ((self.sprite_shifters[sprite_index][1] & offset_mask) > 0) as u8;
        low + (high << 1)
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
        if self.registers.get_mask_flag(MaskFlag::ShowBackground) {
            self.pattern_table_shifters[0] <<= 1;
            self.pattern_table_shifters[1] <<= 1;
            self.palette_shifters[0] <<= 1;
            self.palette_shifters[1] <<= 1;
        }

        if self.registers.get_mask_flag(MaskFlag::ShowSprites) && (self.cycles >= 1 && self.cycles <= 257) {
            for i in 0..self.current_sprite_count {
                if self.sprite_x[i as usize] != 0 {
                    self.sprite_x[i as usize] -= 1;
                }
                else {
                    self.sprite_shifters[i as usize][0] <<= 1;
                    self.sprite_shifters[i as usize][1] <<= 1;
                }
            }
        }
    }

    // Get the right value from the shifters
    pub fn get_shifter_value(&self, shifter: [u16;2]) -> u8 {
        let offset_mask: u16 = 0x8000 >> self.registers.fine_x;
        let low: u8 = ((shifter[0] & offset_mask) > 0) as u8;
        let high: u8 = ((shifter[1] & offset_mask) > 0) as u8;
        low + (high << 1)
    }

    // ===== VRAM ADDRESS MODIFICATION METHODS =====

    // Increments the VRAM address to point to the next 8 bits to render
    pub fn increment_x(&mut self) {
        if self.registers.get_mask_flag(MaskFlag::ShowSprites) || self.registers.get_mask_flag(MaskFlag::ShowBackground) {
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
        if self.registers.get_mask_flag(MaskFlag::ShowSprites) || self.registers.get_mask_flag(MaskFlag::ShowBackground) {
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
        if self.registers.get_mask_flag(MaskFlag::ShowSprites) || self.registers.get_mask_flag(MaskFlag::ShowBackground) {
            // Set coarse x to tmp value
            let tmp_c_x: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::CoarseXScroll);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::CoarseXScroll, tmp_c_x);
            // Set nametable x to tmp value
            let tmp_nt_x: u16 = self.ppu_bus.tmp_vram_address.get_address_part(VRAMAddressMask::NametableX);
            self.ppu_bus.vram_address.set_address_part(VRAMAddressMask::NametableX, tmp_nt_x);
        }
    }

    pub fn copy_tmp_y_to_vram_address(&mut self) {
        if self.registers.get_mask_flag(MaskFlag::ShowSprites) || self.registers.get_mask_flag(MaskFlag::ShowBackground) {
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
                self.p_gui.lock().unwrap().update_debug_buffer(
                    ((n_tile_x*8+(7-col) + number*128 + (n_tile_y*8+row)*256)) as usize, c
                );
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
                        .update_debug_buffer(
                            index as usize, PALETTE[(self.ppu_bus.read(address as u16) & 0x3F) as usize]
                        );
                }
            }
        }
    }
}
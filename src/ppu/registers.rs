use super::{
    bus::PPUBus,
    enums::{ControlFlag, MaskFlag, StatusFlag, VRAMAddressMask},
    oam::OAM,
};

// Reprensents the PPU registers

#[derive(Debug)]
pub struct Registers {
    // PPU registers
    ctrl: u8,
    mask: u8,
    status: u8,
    pub oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u8,
    data: u8,
    pub oam_dma: u8,

    // Temporary registers
    pub decay: u8,
    pub decay_timer: u64,
    data_buffer: u8,

    // Required to check if it is the first or second write to 2006/2007
    w: bool,
    pub fine_x: u8,

    // Emit an NMI interrupt
    pub emit_nmi: bool,
    pub clocks_before_emiting: u8,

    // Required to handle a special case: reading VBL flag
    // as it would be set causes it to not be set for that frame.
    pub clear_vbl: bool,

    // Perform a DMA
    pub perform_dma: bool,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
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

            w: false,
            fine_x: 0,

            emit_nmi: false,
            clocks_before_emiting: 0,

            clear_vbl: false,

            perform_dma: false,
        }
    }

    // Writes value to one of the PPU registers
    pub fn write_register(&mut self, ppu_bus: &mut PPUBus, oam: &mut OAM, address: u16, value: u8) {
        match address {
            0x2000 => {
                let is_previous_nmi_flag_set = self.ctrl & 0x80 > 0;
                self.ctrl = value;
                if !(value & 0x80 > 0) {
                    self.clocks_before_emiting = 0;
                    self.emit_nmi = false;
                }
                if self.get_status_flag(StatusFlag::VBlank) && (value & 0x80) == 0x80 && !is_previous_nmi_flag_set {
                    if self.clocks_before_emiting == 0 {
                        self.emit_nmi = true;
                    }
                }
                ppu_bus
                    .tmp_vram_address
                    .set_address_part(VRAMAddressMask::NametableSelect, (value & 0x03) as u16);
            }
            0x2001 => self.mask = value,
            0x2002 => (),
            0x2003 => self.oam_addr = value,
            0x2004 => {
                self.oam_data = value;
                oam.write_primary(self.oam_addr, self.oam_data);
                if self.oam_addr != 255 {
                    self.oam_addr += 1;
                } else {
                    self.oam_addr = 0;
                }
            }
            0x2005 => {
                self.scroll = value;
                if self.w {
                    ppu_bus
                        .tmp_vram_address
                        .set_address_part(VRAMAddressMask::FineY, (value & 0x07) as u16);
                    ppu_bus.tmp_vram_address.set_address_part(
                        VRAMAddressMask::CoarseYScroll,
                        ((value & 0xF8) as u16) >> 3,
                    );
                    self.w = false;
                } else {
                    ppu_bus.tmp_vram_address.set_address_part(
                        VRAMAddressMask::CoarseXScroll,
                        ((value & 0xF8) as u16) >> 3,
                    );
                    self.fine_x = value & 0x07;
                    self.w = true;
                }
            }
            0x2006 => {
                self.addr = value;
                if self.w {
                    ppu_bus
                        .tmp_vram_address
                        .set_address_part(VRAMAddressMask::SW2006, value as u16);
                    ppu_bus.vram_address.address = ppu_bus.tmp_vram_address.address;
                    self.w = false;
                } else {
                    ppu_bus
                        .tmp_vram_address
                        .set_address_part(VRAMAddressMask::FW2006, (value & 0x3F) as u16);
                    ppu_bus.tmp_vram_address.address &= 0x3FFF; // Sets the 2 higher bits to 0
                    self.w = true;
                }
            }
            0x2007 => {
                self.data = value;
                ppu_bus.write(ppu_bus.vram_address.address & 0x3FFF, value);
                if self.get_control_flag(ControlFlag::VRAMAddressIncrement) == 0 {
                    ppu_bus.vram_address.address += 1; // Horizontal scrolling
                } else {
                    ppu_bus.vram_address.address += 32; // Vertical scrolling
                }
            }
            0x4014 => {
                self.oam_dma = value;
                self.perform_dma = true;
            }
            _ => panic!("Wrong address given to PPU : {:#x}", address),
        }
        self.decay = value;
        self.decay_timer = 0;
    }

    // Reads value from one of the PPU registers
    pub fn read_register(&mut self, ppu_bus: &mut PPUBus, oam: &OAM, address: u16) -> u8 {
        let mut value: u8;
        match address {
            0x2000 => value = self.decay,
            0x2001 => value = self.decay,
            0x2002 => {
                value = (self.status & 0xE0) | (self.decay & 0x1F);
                self.decay = value;
                self.clear_vbl = true;
                self.emit_nmi = false;
                self.w = false;
            }
            0x2003 => value = self.decay,
            0x2004 => {
                value = oam.read_primary(self.oam_addr);
                self.decay = value;
            }
            0x2005 => value = self.decay,
            0x2006 => value = self.decay,
            0x2007 => {
                // Read to 2007 is delayed by one read except for the palette
                value = self.data_buffer;
                self.data_buffer = ppu_bus.read(ppu_bus.vram_address.address);
                if ppu_bus.vram_address.address >= 0x3F00 {
                    value = (self.decay & 0xC0) | (self.data_buffer & 0x3F);
                    // Fill the buffer with the mirrored nametable "under" palette RAM
                    self.data_buffer = ppu_bus.read(ppu_bus.vram_address.address & 0x2FFF);
                }
                self.decay = value;
                // Increment VRAM Address
                if self.get_control_flag(ControlFlag::VRAMAddressIncrement) == 0 {
                    ppu_bus.vram_address.address += 1; // Horizontal scrolling
                } else {
                    ppu_bus.vram_address.address += 32; // Vertical scrolling
                }
            }
            0x4014 => panic!("4014 is not readable !"),
            _ => panic!("Wrong address given to PPU : {:#x}", address),
        }
        value
    }

    // Sets the flags for the status register
    pub fn set_status_flag(&mut self, flag: StatusFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn get_status_flag(&self, flag: StatusFlag) -> bool {
        (self.status & (flag as u8)) == (flag as u8)
    }

    // Get the flags from the control register
    pub fn get_control_flag(&mut self, flag: ControlFlag) -> u8 {
        if flag != ControlFlag::NametableAddress {
            ((self.ctrl & (flag as u8)) == (flag as u8)) as u8
        } else {
            (self.ctrl & 0x03) as u8 // Last two bits
        }
    }

    // Get the flags from the mask register
    pub fn get_mask_flag(&self, flag: MaskFlag) -> bool {
        (self.mask & (flag as u8)) == (flag as u8)
    }

    // Used for debugging
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
            _ => panic!("Wrong address given to PPU : {:#x}", address),
        }
        value
    }
}

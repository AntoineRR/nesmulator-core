// Reprensents the OAM (Object Attribute Memory) of the PPU

use super::sprite::Sprite;

#[derive(Debug)]
pub struct OAM {
    pub primary: [Sprite;64],
    pub secondary: [Sprite;8]
}

impl OAM {
    pub fn new() -> Self {
        OAM {
            primary: [Sprite::default();64],
            secondary: [Sprite::default();8]
        }
    }

    pub fn write_primary(&mut self, address: u8, data: u8) {
        let sprite_index: usize = (address / 4) as usize;
        match address % 4 {
            // Y
            0 => self.primary[sprite_index].y = data,
            // ID
            1 => self.primary[sprite_index].id = data,
            // Attribute
            2 => self.primary[sprite_index].attribute = data & 0xE3,
            // X
            3 => self.primary[sprite_index].x = data,
            _ => panic!("Impossible to reach pattern")
        }
    }

    pub fn read_primary(&self, address: u8) -> u8 {
        let sprite_index: usize = (address / 4) as usize;
        match address % 4 {
            // Y
            0 => self.primary[sprite_index].y,
            // ID
            1 => self.primary[sprite_index].id,
            // Attribute
            2 => self.primary[sprite_index].attribute & 0xE3,
            // X
            3 => self.primary[sprite_index].x,
            _ => panic!("Impossible to reach pattern")
        }
    }

    pub fn write_secondary(&mut self, address: u8, data: u8) {
        let sprite_index: usize = (address / 4) as usize;
        match address % 4 {
            // Y
            0 => self.secondary[sprite_index].y = data,
            // ID
            1 => self.secondary[sprite_index].id = data,
            // Attribute
            2 => self.secondary[sprite_index].attribute = data,
            // X
            3 => self.secondary[sprite_index].x = data,
            _ => panic!("Unreachable pattern")
        }
    }
}
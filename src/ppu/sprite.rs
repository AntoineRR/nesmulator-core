use serde::{Deserialize, Serialize};

use super::enums::SpriteAttribute;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sprite {
    pub y: u8,
    pub id: u8,
    pub attribute: u8,
    pub x: u8,
}

impl Sprite {
    pub fn default() -> Self {
        Sprite {
            y: 0,
            id: 0,
            attribute: 0,
            x: 0,
        }
    }

    pub fn get_attribute_flag(&self, flag: SpriteAttribute) -> u8 {
        if flag != SpriteAttribute::Palette {
            ((self.attribute & (flag as u8)) > 0) as u8
        } else {
            self.attribute & 0x03
        }
    }
}

// Implements the different color palettes of the NES

use crate::utils::ARGBColor;
use std::{error::Error, fs};

pub struct Palette {
    pub base: [ARGBColor; 64],
    pub emphasize_r: [ARGBColor; 64],
    pub emphasize_g: [ARGBColor; 64],
    pub emphasize_b: [ARGBColor; 64],
    pub emphasize_rg: [ARGBColor; 64],
    pub emphasize_rb: [ARGBColor; 64],
    pub emphasize_gb: [ARGBColor; 64],
    pub emphasize_rgb: [ARGBColor; 64],
}

impl Palette {
    pub fn default() -> Self {
        // No emphasize on default palette for now
        Palette {
            base: PALETTE.clone(),
            emphasize_r: PALETTE.clone(),
            emphasize_g: PALETTE.clone(),
            emphasize_b: PALETTE.clone(),
            emphasize_rg: PALETTE.clone(),
            emphasize_rb: PALETTE.clone(),
            emphasize_gb: PALETTE.clone(),
            emphasize_rgb: PALETTE.clone(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        fn parse_palette_bytes(palette: &[u8]) -> [ARGBColor; 64] {
            let mut p = [ARGBColor::black(); 64];
            for (i, color) in palette.chunks(3).enumerate() {
                p[i] = ARGBColor::new(255, color[0], color[1], color[2]);
            }
            p
        }

        let raw = fs::read(path)?;

        // Palette file can contain a base palette and all emphasized versions (8 in total)
        // Each palette has 64 colors, and each color is composed of 3 bytes (r, g, b) => 1536 bytes
        // Or just contain a base palette, used for all other emphasized components => 192 bytes
        let is_full_palette = match raw.len() {
            1536 => true,
            192 => false,
            _ => Err("Palette file has an incorrect format")?,
        };

        let mut palettes = vec![];
        if is_full_palette {
            for palette in raw.chunks(64 * 3) {
                palettes.push(parse_palette_bytes(palette));
            }
        } else {
            for _ in 0..8 {
                palettes.push(parse_palette_bytes(&raw));
            }
        }

        Ok(Palette {
            base: palettes[0],
            emphasize_r: palettes[1],
            emphasize_g: palettes[2],
            emphasize_b: palettes[3],
            emphasize_rg: palettes[4],
            emphasize_rb: palettes[5],
            emphasize_gb: palettes[6],
            emphasize_rgb: palettes[7],
        })
    }
}

pub const PALETTE: [ARGBColor; 64] = [
    ARGBColor::new(255, 84, 84, 84),    // 0x00
    ARGBColor::new(255, 0, 30, 116),    // 0x01
    ARGBColor::new(255, 8, 16, 144),    // 0x02
    ARGBColor::new(255, 48, 0, 136),    // 0x03
    ARGBColor::new(255, 68, 0, 100),    // 0x04
    ARGBColor::new(255, 92, 0, 48),     // 0x05
    ARGBColor::new(255, 84, 4, 0),      // 0x06
    ARGBColor::new(255, 60, 24, 0),     // 0x07
    ARGBColor::new(255, 32, 42, 0),     // 0x08
    ARGBColor::new(255, 8, 58, 0),      // 0x09
    ARGBColor::new(255, 0, 64, 0),      // 0x0A
    ARGBColor::new(255, 0, 60, 0),      // 0x0B
    ARGBColor::new(255, 0, 50, 60),     // 0x0C
    ARGBColor::new(255, 0, 0, 0),       // 0x0D
    ARGBColor::new(255, 0, 0, 0),       // 0x0E
    ARGBColor::new(255, 0, 0, 0),       // 0x0F
    ARGBColor::new(255, 152, 150, 152), // 0x10
    ARGBColor::new(255, 8, 76, 196),    // 0x11
    ARGBColor::new(255, 48, 50, 236),   // 0x12
    ARGBColor::new(255, 92, 30, 228),   // 0x13
    ARGBColor::new(255, 136, 20, 176),  // 0x14
    ARGBColor::new(255, 160, 20, 100),  // 0x15
    ARGBColor::new(255, 152, 34, 32),   // 0x16
    ARGBColor::new(255, 120, 60, 0),    // 0x17
    ARGBColor::new(255, 84, 90, 0),     // 0x18
    ARGBColor::new(255, 40, 114, 0),    // 0x19
    ARGBColor::new(255, 8, 124, 0),     // 0x1A
    ARGBColor::new(255, 0, 118, 140),   // 0x1B
    ARGBColor::new(255, 0, 102, 120),   // 0x1C
    ARGBColor::new(255, 0, 0, 0),       // 0x1D
    ARGBColor::new(255, 0, 0, 0),       // 0x1E
    ARGBColor::new(255, 0, 0, 0),       // 0x1F
    ARGBColor::new(255, 236, 238, 236), // 0x20
    ARGBColor::new(255, 76, 154, 236),  // 0x21
    ARGBColor::new(255, 120, 124, 236), // 0x22
    ARGBColor::new(255, 176, 98, 236),  // 0x23
    ARGBColor::new(255, 228, 84, 236),  // 0x24
    ARGBColor::new(255, 236, 88, 180),  // 0x25
    ARGBColor::new(255, 236, 106, 100), // 0x26
    ARGBColor::new(255, 212, 136, 32),  // 0x27
    ARGBColor::new(255, 160, 170, 0),   // 0x28
    ARGBColor::new(255, 116, 196, 0),   // 0x29
    ARGBColor::new(255, 76, 208, 32),   // 0x2A
    ARGBColor::new(255, 56, 204, 108),  // 0x2B
    ARGBColor::new(255, 56, 180, 204),  // 0x2C
    ARGBColor::new(255, 60, 60, 60),    // 0x2D
    ARGBColor::new(255, 0, 0, 0),       // 0x2E
    ARGBColor::new(255, 0, 0, 0),       // 0x2F
    ARGBColor::new(255, 236, 238, 236), // 0x30
    ARGBColor::new(255, 168, 204, 236), // 0x31
    ARGBColor::new(255, 188, 188, 236), // 0x32
    ARGBColor::new(255, 212, 178, 236), // 0x33
    ARGBColor::new(255, 236, 174, 236), // 0x34
    ARGBColor::new(255, 236, 174, 212), // 0x35
    ARGBColor::new(255, 236, 180, 176), // 0x36
    ARGBColor::new(255, 228, 196, 144), // 0x37
    ARGBColor::new(255, 204, 210, 120), // 0x38
    ARGBColor::new(255, 180, 222, 120), // 0x39
    ARGBColor::new(255, 168, 226, 144), // 0x3A
    ARGBColor::new(255, 152, 226, 180), // 0x3B
    ARGBColor::new(255, 160, 214, 228), // 0x3C
    ARGBColor::new(255, 160, 162, 160), // 0x3D
    ARGBColor::new(255, 0, 0, 0),       // 0x3E
    ARGBColor::new(255, 0, 0, 0),       // 0x3F
];

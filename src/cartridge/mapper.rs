
#[derive(Clone, Copy)]
pub enum Mirroring {
    Horizontal = 0,
    Vertical = 1,
    OneScreenLower = 2,
    OneScreenUpper = 3
}

pub trait Mapper {
    fn prg_rom_read(&self, address: u16) -> u8;
    fn prg_rom_write(&mut self, address: u16, value: u8);
    fn chr_rom_read(&self, address: u16) -> u8;
    fn chr_rom_write(&mut self, address: u16, value: u8);
    fn get_mirroring(&self) -> Mirroring;
    fn box_clone(&self) -> Box<dyn Mapper>;
}

impl Clone for Box<dyn Mapper> {
    fn clone(&self) -> Box<dyn Mapper> {
        self.box_clone()
    }
}
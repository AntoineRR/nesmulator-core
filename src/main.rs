mod cpu;
mod bus;
mod nes;

use nes::NES;

static mut NES: NES = NES::new();

fn main() {
    println!("Hello, world!");
    unsafe {
        NES.insert_cartdrige();
        NES.launch_game();
    }
}

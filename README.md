# nesmulator

## Desccription

A simple Nintendo Entertainment System (NES) emulator written in Rust language.

The purpose of this emulator is for me to learn how the NES works and to challenge myself.
It is my first real project in Rust.

## Final goal

I do not plan to make this emulator compatible with every NES game.
My goal is to be able to emulate games like Zelda, Dragon Warrior, and Castlevania.
I would also like to pass most of the tests from some tests roms listed in the NES dev wiki.

## Current progress

The CPU is emulated and a cartridge in the iNES format can be loaded into the emulator.
For now, only cartridges using mapper 0, 1, 2 and 3 can be read.
The PPU background and sprite display is emulated.
The GUI is created using [winit](https://github.com/rust-windowing/winit) and [pixels](https://github.com/parasyte/pixels), and displays the game screen.
The first controller is emulated too (see controls).
The APU is partially emulated but not very accurate.
A debugging view (display of pattern tables and palette) can be toggled.

## How to run

You will need a rust stable installation.

* Clone the repository locally.
* Run `cargo run --release -- <ROM_PATH>` in a terminal, in the "nesmulator" folder.

More options can be displayed with the `-h` or `--help` flag.

## Controls

* UP -> Z
* DOWN -> S
* LEFT -> Q
* RIGHT -> D
* A -> I
* B -> O
* START -> X
* SELECT -> C

* Pattern tables and palette display (debugging mode) -> E

## Tests

See [TESTS.md](TESTS.md) for details about tests.

## To do

* Correct MMC1 mapper
* Fix APU emulation, especially DMC
* Pass more tests

## License

This code is distributed under the [MIT license](LICENSE).

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
* [Nintendulator nestest.nes logs](https://www.qmtpro.com/~nes/misc/nestest.log)
* [Test ROMs](https://github.com/christopherpow/nes-test-roms)
* [winit](https://github.com/rust-windowing/winit)
* [pixels](https://github.com/parasyte/pixels)
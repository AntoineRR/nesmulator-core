# nesmulator

## Description

A simple Nintendo Entertainment System (NES) emulator written in Rust language.

The purpose of this emulator is for me to learn how the NES works and to challenge myself.
It is my first real project in Rust.

## Final goal

I do not plan to make this emulator compatible with every NES game.
My goal is to be able to emulate games like Zelda, Dragon Warrior, and Castlevania.
I would also like to pass most of the tests from some tests roms listed in the NES dev wiki (see [TESTS.md](./TESTS.md)).

## Current progress

### NES emulation core

* [X] CPU is emulated and passes most tests
* [X] PPU is emulated and passes most tests
* [X] APU is emulated
* [X] First controller is emulated (see controls below)
* [X] A cartridge in the iNES format can be loaded into the emulator
* [X] Mapper 0, 1, 2 and 3 are implemented
* [X] A palette in the .pal format can be loaded into the emulator, otherwise a default palette is hardcoded into the emulator
* [X] ROM from cartridges that had a saving system can save the game in a file with the .sav extension

### Provided GUI

* [X] Display the game screen
* [X] A debugging view (display of pattern tables and palette) can be toggled
* [X] First Controller mapping for keyboard
* [X] CLI with various flags (see below)

The GUI is created using [winit](https://github.com/rust-windowing/winit) and [pixels](https://github.com/parasyte/pixels).
The sound is handled by [sdl2](https://github.com/Rust-SDL2/rust-sdl2).

## How to run

You will need a rust stable installation.

* Clone the repository locally.
* Run `cargo run --release -- <ROM_PATH>` in a terminal, in the "nesmulator" folder.

More options can be displayed with the `-h` or `--help` flag.

## Controls

### Controller mapping

| Button | Key |
| ------ | --- |
| UP     | Z   |
| DOWN   | S   |
| LEFT   | Q   |
| RIGHT  | D   |
| A      | I   |
| B      | O   |
| START  | X   |
| SELECT | C   |

### Emulator features

| Feature              | Key        |
| -------------------- | ---------- |
| Debugging mode       | E          |
| Choose debug palette | Left/Right |
| Reset CPU            | R          |

## Tests

See [TESTS.md](./TESTS.md) for details about tests.

## To do

* Pass more tests
* Add a configuration file for mapping Controls
* Improve error handling and add more logs
* Add a save state system
* Allow to configure the save path
* Improve mapper 1

## License

This code is distributed under the [MIT license](LICENSE).

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
* [Nintendulator nestest.nes logs](https://www.qmtpro.com/~nes/misc/nestest.log)
* [Test ROMs](https://github.com/christopherpow/nes-test-roms)
* [winit](https://github.com/rust-windowing/winit)
* [pixels](https://github.com/parasyte/pixels)
* [Joel Yliluoma palette generator](https://bisqwit.iki.fi/utils/nespalette.php)
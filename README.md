# Nesmulator core

## Description

A simple Nintendo Entertainment System (NES) emulator written in Rust language.

This repository only contains the core of the NES emulation. It doesn't provide any GUI by itself. For a full GUI project using this crate, check out [nesmulator-gui](https://github.com/AntoineRR/nesmulator-gui).

## Final goal

I do not plan to make this emulator compatible with every NES game.
My goal is to be able to emulate games like Zelda, Dragon Warrior, and Castlevania.
I would also like to pass most of the tests from some tests roms listed in the NES dev wiki (see [TESTS.md](./TESTS.md)).

## Current progress

* [X] CPU is emulated and passes most tests
* [X] PPU is emulated and passes most tests
* [X] APU is emulated
* [X] First controller is emulated (see controls below)
* [X] A cartridge in the iNES format can be loaded into the emulator
* [X] Mapper 0, 1, 2 and 3 are implemented
* [X] A palette in the .pal format can be loaded into the emulator, otherwise a default palette is hardcoded into the emulator
* [X] ROM from cartridges that had a saving system can save the game in a file with the .sav extension
* [X] The current state of the emulator can be saved and loaded back at any moment, allowing saving games that do not support saves otherwise

## How to use

The [nesmulator-gui](https://github.com/AntoineRR/nesmulator-gui) repository can be used as a reference for using this crate.

This crate is not published on crates.io (yet?), so to use it in one of your projects, add the following to your `Cargo.toml`:
```
nesmulator_core = { git = "https://github.com/AntoineRR/nesmulator-core" }
```

An example usage of this crate is provided in `src/bin/main.rs`. It will attempt to load the `nestest.nes` ROM and run it on automation until completion. To run this example, you first have to downolad the [nestest.nes](http://nickmass.com/images/nestest.nes) ROM and change the `rom_path` variable in `src/bin/main.rs` to link to its location on your computer. Then, run the following command:
```
cargo run --release
```
This will display the logs of the CPU.

## Tests

See [TESTS.md](./TESTS.md) for details about tests.

## To do

* Pass more tests
* Improve mapper 1
* Improve general architecture to remove `Rc`
* Expose API to retrieve instructions that are being executed

## License

This code is distributed under the [MIT license](./LICENSE).

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
* [Nintendulator nestest.nes logs](https://www.qmtpro.com/~nes/misc/nestest.log)
* [Test ROMs](https://github.com/christopherpow/nes-test-roms)
* [Joel Yliluoma palette generator](https://bisqwit.iki.fi/utils/nespalette.php)
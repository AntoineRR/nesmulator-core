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

## How to use

The [nesmulator-gui](https://github.com/AntoineRR/nesmulator-gui) repository can be used as a reference for using this crate.

This crate is not published on crates.io (yet?), so to use it in one of your projects, add the following to your `Cargo.toml`:
```
nesmulator_core = { git = "https://github.com/AntoineRR/nesmulator-core" }
```

## Tests

See [TESTS.md](./TESTS.md) for details about tests.

## To do

* Pass more tests
* Improve error handling and add more logs
* Add a save state system
* Allow to configure the save path
* Improve mapper 1

## License

This code is distributed under the [MIT license](./LICENSE).

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
* [Nintendulator nestest.nes logs](https://www.qmtpro.com/~nes/misc/nestest.log)
* [Test ROMs](https://github.com/christopherpow/nes-test-roms)
* [Joel Yliluoma palette generator](https://bisqwit.iki.fi/utils/nespalette.php)
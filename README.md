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
For now, only cartridges using mapper 0 can be read.
A GUI using minifb is launched on start and currently displays noise through a very basic PPU struct.

## To do

Next step is to implement the PPU component.

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
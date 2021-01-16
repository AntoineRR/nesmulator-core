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
The PPU background display is emulated and works well. Sprites are missing for now.
The GUI is created using [minifb](https://docs.rs/minifb/0.19.1/minifb/), and displays the game screen, as well as a debugging screen that can be activated by pressing "e" on the keyboard.
A press on the "r" key resets CPU.
The first controller is emulated too (see controls).

## Controls

* UP -> Z
* DOWN -> S
* LEFT -> Q
* RIGHT -> D
* A -> I
* B -> O
* START -> X
* SELECT -> C

* debug -> E
* reset CPU -> R

## Tests

The CPU has been tested with the nestest.nes ROM.
The logs of the CPU and the PPU, including the disassembly of the ROM code, can be displayed in a similar manner to [Nintendulator](https://www.qmtpro.com/~nes/nintendulator/). This has been useful to compare the logs of my emulator to Nintendulator. The logs are exactly the same on both emulators.
The pattern tables and palettes can be displayed in a separate window when "e" key is pressed.

## To do

Next step is to emulate the sprite display of the PPU.

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
* [Nintendulator nestest.nes logs](https://www.qmtpro.com/~nes/misc/nestest.log)
* [minifb (GUI)](https://docs.rs/minifb/0.19.1/minifb/)
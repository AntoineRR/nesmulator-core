# nesmulator

A simple Nintendo Entertainment System (NES) emulator written in Rust language.

The purpose of this emulator is for me to learn how the NES works and to challenge myself.
It is my first real project in Rust.

My goal is to be able to emulate games like Zelda, Dragon Warrior, and Castlevania.

The CPU is emulated and a cartridge in the iNES format can be loaded into the emulator.
For now, only cartridges using mapper 0 can be read.

Next step is to implement a GUI and a PPU architecture.
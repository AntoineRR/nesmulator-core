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

## How to run

You will need a rust stable installation.

* Clone the repository locally.
* The path to your ROM must be changed in main.rs.
* Run `cargo run` in a terminal, in the "nesmulator" folder.

## Controls

* UP -> Z
* DOWN -> S
* LEFT -> Q
* RIGHT -> D
* A -> I
* B -> O
* START -> X
* SELECT -> C

## Tests

The logs of the CPU and the PPU, including the disassembly of the ROM code, can be displayed in a similar manner to [Nintendulator](https://www.qmtpro.com/~nes/nintendulator/). This has been useful to compare the logs of my emulator to the ones from Nintendulator when nestest.nes is run on automation. The logs are exactly the same on both emulators.

I used several test ROMs for testing my emulator. Those can be found [here](https://github.com/christopherpow/nes-test-roms). 
Some nes_instr_test ROMs fail because some undocumented opcodes are not implemented correctly for now.
Here are the results :

### CPU

* [X] nestest
* [X] branch_timing_tests/1.Branch_Basics
* [X] branch_timing_tests/2.Backward_Branch
* [X] branch_timing_tests/3.Forward_Branch
* [X] cpu_timing_test6/cpu_timing_test
* [X] nes_instr_test/rom_singles/01-implied
* [ ] nes_instr_test/rom_singles/02-immediate
* [ ] nes_instr_test/rom_singles/03-zero_page
* [ ] nes_instr_test/rom_singles/04-zp_xy
* [ ] nes_instr_test/rom_singles/05-absolute
* [ ] nes_instr_test/rom_singles/06-abs_xy
* [ ] nes_instr_test/rom_singles/07-ind_x
* [ ] nes_instr_test/rom_singles/08-ind_y
* [X] nes_instr_test/rom_singles/09-branches
* [X] nes_instr_test/rom_singles/10-stack
* [ ] nes_instr_test/rom_singles/11-special

### PPU

* [X] blargg_ppu_tests_2005.09.15b/palette_ram
* [ ] blargg_ppu_tests_2005.09.15b/sprite_ram
* [ ] blargg_ppu_tests_2005.09.15b/vbl_clear_time
* [X] blargg_ppu_tests_2005.09.15b/vram_access
* [ ] ppu_open_bus/ppu_open_bus
* [ ] scanline/scanline
* [X] vbl_nmi_timing/1.frame_basics
* [ ] vbl_nmi_timing/2.vbl_timing
* [ ] vbl_nmi_timing/3.even_odd_frames
* [X] vbl_nmi_timing/4.vbl_clear_timing
* [ ] vbl_nmi_timing/5.nmi_suppression
* [ ] vbl_nmi_timing/6.nmi_disable
* [ ] vbl_nmi_timing/7.nmi_timing
* [X] oam_read/oam_read
* [ ] oam_stress/oam_stress
* [X] sprite_hit_tests_2005.10.05/01.basics
* [X] sprite_hit_tests_2005.10.05/02.alignment
* [X] sprite_hit_tests_2005.10.05/03.corners
* [X] sprite_hit_tests_2005.10.05/04.flip
* [X] sprite_hit_tests_2005.10.05/05.left_clip
* [X] sprite_hit_tests_2005.10.05/06.right_edge
* [X] sprite_hit_tests_2005.10.05/07.screen_bottom
* [X] sprite_hit_tests_2005.10.05/08.double_height
* [X] sprite_hit_tests_2005.10.05/09.timing_basics
* [X] sprite_hit_tests_2005.10.05/10.timing_order
* [X] sprite_hit_tests_2005.10.05/11.edge_timing
* [X] sprite_overflow_tests/1.Basics
* [ ] sprite_overflow_tests/2.Details
* [ ] sprite_overflow_tests/3.Timing
* [ ] sprite_overflow_tests/4.Obscure
* [ ] sprite_overflow_tests/5.Emulator

## To do

* Correct scroll for MMC1
* Add APU emulation for sound

## References

* [NES dev wiki](http://wiki.nesdev.com/w/index.php/Nesdev)
* [javidx9 tutorial](https://www.youtube.com/watch?v=F8kx56OZQhg&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&index=2)
* [Nintendulator nestest.nes logs](https://www.qmtpro.com/~nes/misc/nestest.log)
* [Test ROMs](https://github.com/christopherpow/nes-test-roms)
* [winit](https://github.com/rust-windowing/winit)
* [pixels](https://github.com/parasyte/pixels)
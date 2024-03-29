# Nesmulator Tests

## Displaying logs

The logs of the CPU and the PPU, including the disassembly of the ROM code, can be displayed in a similar manner to [Nintendulator](https://www.qmtpro.com/~nes/nintendulator/). This has been useful to compare the logs of my emulator to the ones from Nintendulator when nestest.nes is run on automation. The logs are the same on both emulators. They can be displayed using the `-l` flag.

Additionally, a level of debugging can be chosen with the `d` flag, followed by a debug level, from 1 to 4 included, 4 being the highest level of debugging.

## Test ROMS

I used several test ROMs for testing my emulator. Those can be found [here](https://github.com/christopherpow/nes-test-roms), and the test ROMS for the mappers are [here](https://pineight.com/nes/holydiverbatman-bin-0.01.7z).

Here are some of the results :

### CPU

#### Automatically tested (see [Integration tests](#Integration-tests))

* [X] nes_instr_test/rom_singles/01-implied
* [X] nes_instr_test/rom_singles/02-immediate
* [X] nes_instr_test/rom_singles/03-zero_page
* [X] nes_instr_test/rom_singles/04-zp_xy
* [X] nes_instr_test/rom_singles/05-absolute
* [X] nes_instr_test/rom_singles/06-abs_xy
* [X] nes_instr_test/rom_singles/07-ind_x
* [X] nes_instr_test/rom_singles/08-ind_y
* [X] nes_instr_test/rom_singles/09-branches
* [X] nes_instr_test/rom_singles/10-stack
* [ ] nes_instr_test/rom_singles/11-special
* [X] instr_timing/rom_singles/1-instr_timing
* [X] instr_timing/rom_singles/2-branch_timing
* [X] instr_misc/rom_singles/01-abs_x_wrap
* [X] instr_misc/rom_singles/02-branch_wrap
* [ ] instr_misc/rom_singles/03-dummy_reads
* [X] instr_misc/rom_singles/04-dummy_reads_apu
* [X] cpu_reset/registers
* [X] cpu_reset/ram_after_reset
* [ ] cpu_exec_space/test_cpu_exec_space_apu
* [ ] cpu_exec_space/test_cpu_exec_space_ppuio
* [ ] cpu_interrupts_v2/rom_singles/1-cli_latency
* [ ] cpu_interrupts_v2/rom_singles/2-nmi_and_brk
* [ ] cpu_interrupts_v2/rom_singles/3-nmi_and_irq
* [ ] cpu_interrupts_v2/rom_singles/4-irq_and_dma
* [ ] cpu_interrupts_v2/rom_singles/5-branch_delays_irq

#### Manually tested

* [X] nestest
* [X] branch_timing_tests/1.Branch_Basics
* [X] branch_timing_tests/2.Backward_Branch
* [X] branch_timing_tests/3.Forward_Branch
* [X] cpu_timing_test6/cpu_timing_test

### PPU

#### Automatically tested (see [Integration tests](#Integration-tests))

* [X] ppu_vbl_nmi/rom_singles/01-vbl_basics
* [X] ppu_vbl_nmi/rom_singles/02-vbl_set_time
* [X] ppu_vbl_nmi/rom_singles/03-vbl_clear_time
* [ ] ppu_vbl_nmi/rom_singles/04-nmi_control
* [ ] ppu_vbl_nmi/rom_singles/05-nmi_timing
* [X] ppu_vbl_nmi/rom_singles/06-suppression
* [ ] ppu_vbl_nmi/rom_singles/07-nmi_on_timing
* [X] ppu_vbl_nmi/rom_singles/08-nmi_off_timing
* [X] ppu_vbl_nmi/rom_singles/09-even_odd_frames
* [ ] ppu_vbl_nmi/rom_singles/10-even_odd_timing
* [ ] ppu_read_buffer/test_ppu_read_buffer
* [X] ppu_open_bus/ppu_open_bus
* [X] oam_read/oam_read
* [X] oam_stress/oam_stress

#### Manually tested

* [X] blargg_ppu_tests_2005.09.15b/palette_ram
* [X] blargg_ppu_tests_2005.09.15b/sprite_ram
* [ ] blargg_ppu_tests_2005.09.15b/power_up_palette
* [ ] blargg_ppu_tests_2005.09.15b/vbl_clear_time
* [X] blargg_ppu_tests_2005.09.15b/vram_access
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
* [X] sprite_overflow_tests/2.Details
* [ ] sprite_overflow_tests/3.Timing
* [ ] sprite_overflow_tests/4.Obscure
* [X] sprite_overflow_tests/5.Emulator
* [ ] scanline/scanline

### APU

#### Automatically tested (see [Integration tests](#Integration-tests))

* [X] apu_test/rom_singles/1-len_ctr
* [X] apu_test/rom_singles/2-len_table
* [X] apu_test/rom_singles/3-irq_flag
* [X] apu_test/rom_singles/4-jitter
* [ ] apu_test/rom_singles/5-len_timing
* [ ] apu_test/rom_singles/6-irq_flag_timing
* [X] apu_test/rom_singles/7-dmc_basics
* [ ] apu_test/rom_singles/8-dmc_rates
* [X] apu_reset/4015_cleared
* [ ] apu_reset/4017_timing
* [X] apu_reset/4017_written
* [X] apu_reset/irq_flag_cleared
* [ ] apu_reset/len_ctrs_enabled
* [X] apu_reset/works_immediately

#### Manually tested

* [X] blargg_apu_2005.07.30/01.len_ctr
* [X] blargg_apu_2005.07.30/02.len_table
* [X] blargg_apu_2005.07.30/03.irq_flag
* [X] blargg_apu_2005.07.30/04.clock_jitter
* [ ] blargg_apu_2005.07.30/05.len_timing_mode0
* [ ] blargg_apu_2005.07.30/06.len_timing_mode1
* [ ] blargg_apu_2005.07.30/07.irq_flag_timing
* [ ] blargg_apu_2005.07.30/08.irq_timing
* [ ] blargg_apu_2005.07.30/09.reset_timing
* [ ] blargg_apu_2005.07.30/10.len_halt_timing
* [ ] blargg_apu_2005.07.30/11.len_reload_timing

### Mappers

#### Manually tested

* [X] M0_P32K_C8K_V
* [ ] M1_P128K
* [ ] M1_P128K_C32K
* [ ] M1_P128K_C32K_S8K
* [ ] M1_P128K_C32K_W8K
* [ ] M1_P128K_C128K
* [ ] M1_P128K_C128K_S8K
* [ ] M1_P128K_C128K_W8K
* [ ] M1_P512K_S8K
* [ ] M1_P512K_S32K
* [X] M2_P128K_V
* [X] M3_P32K_C32K_H

## Integration tests

Some test ROMs write their output strating at address 0x6000, allowing for automation in running tests. Some of those ROMs can be run automatically using integration tests.

### How to setup

Here are the step to follow to run those tests:

* Download the test ROMs used from [this](https://github.com/christopherpow/nes-test-roms) repository.
* Modify the `ROM_PATH_PREFIX` in `tests/common/mod.rs` to link to were you cloned the repository.
* Run tests using `cargo test`

### Running specific tests

Some tests might fail as the emulator is still in development. This will prevent other tests from running.

```
cargo test --test cpu  // Run tests in cpu.rs
cargo test nes_instr_test::  // Run test in the nes_instr_test module
cargo test nes_instr_test::implied  // Run test nes_instr_test::implied
```

## Benchmark

A benchmark can be run using `cargo bench`. The benchmark uses the [criterion](https://github.com/bheisler/criterion.rs) crate.
This will repeatedly run the `nestest.nes` ROM in automation, restarting the nes emulator when it reaches the end of the ROM.
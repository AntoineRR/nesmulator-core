mod common;

mod nes_instr_test {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "nes_instr_test/rom_singles/";

    #[test]
    fn implied() {
        run_rom(&get_path("01-implied.nes"));
    }

    #[test]
    fn immediate() {
        run_rom(&get_path("02-immediate.nes"));
    }

    #[test]
    fn zero_page() {
        run_rom(&get_path("03-zero_page.nes"));
    }

    #[test]
    fn zero_page_xy() {
        run_rom(&get_path("04-zp_xy.nes"));
    }

    #[test]
    fn absolute() {
        run_rom(&get_path("05-absolute.nes"));
    }

    #[test]
    fn absolute_xy() {
        run_rom(&get_path("06-abs_xy.nes"));
    }

    #[test]
    fn indirect_x() {
        run_rom(&get_path("07-ind_x.nes"));
    }

    #[test]
    fn indirect_y() {
        run_rom(&get_path("08-ind_y.nes"));
    }

    #[test]
    fn branches() {
        run_rom(&get_path("09-branches.nes"));
    }

    #[test]
    fn stack() {
        run_rom(&get_path("10-stack.nes"));
    }

    #[test]
    fn special() {
        run_rom(&get_path("11-special.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod instr_timing {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "instr_timing/rom_singles/";

    #[test]
    fn instr_timing() {
        run_rom(&get_path("1-instr_timing.nes"));
    }

    #[test]
    fn branch_timing() {
        run_rom(&get_path("2-branch_timing.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod instr_misc {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "instr_misc/rom_singles/";

    #[test]
    fn absolute_x_wrap() {
        run_rom(&get_path("01-abs_x_wrap.nes"));
    }

    #[test]
    fn branch_wrap() {
        run_rom(&get_path("02-branch_wrap.nes"));
    }

    #[test]
    fn dummy_reads() {
        run_rom(&get_path("03-dummy_reads.nes"));
    }

    #[test]
    fn dummy_reads_apu() {
        run_rom(&get_path("04-dummy_reads_apu.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod cpu_reset {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "cpu_reset/";

    #[test]
    fn registers() {
        run_rom(&get_path("registers.nes"))
    }

    #[test]
    fn ram_after_reset() {
        run_rom(&get_path("ram_after_reset.nes"))
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

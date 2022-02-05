mod common;

mod apu_test {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "apu_test/rom_singles/";

    #[test]
    fn length_counter() {
        run_rom(&get_path("1-len_ctr.nes"));
    }

    #[test]
    fn length_table() {
        run_rom(&get_path("2-len_table.nes"));
    }

    #[test]
    fn irq_flag() {
        run_rom(&get_path("3-irq_flag.nes"));
    }

    #[test]
    fn jitter() {
        run_rom(&get_path("4-jitter.nes"));
    }

    #[test]
    fn length_timing() {
        run_rom(&get_path("5-len_timing.nes"));
    }

    #[test]
    fn irq_flag_timing() {
        run_rom(&get_path("6-irq_flag_timing.nes"));
    }

    #[test]
    fn dmc_basics() {
        run_rom(&get_path("7-dmc_basics.nes"));
    }

    #[test]
    fn dmc_rates() {
        run_rom(&get_path("8-dmc_rates.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

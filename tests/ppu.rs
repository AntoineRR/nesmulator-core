mod common;

mod ppu_vbl_nmi {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "ppu_vbl_nmi/rom_singles/";

    #[test]
    fn vbl_basics() {
        run_rom(&get_path("01-vbl_basics.nes"));
    }

    #[test]
    fn vbl_set_time() {
        run_rom(&get_path("02-vbl_set_time.nes"));
    }

    #[test]
    fn vbl_clear_time() {
        run_rom(&get_path("03-vbl_clear_time.nes"));
    }

    #[test]
    fn nmi_control() {
        run_rom(&get_path("04-nmi_control.nes"));
    }

    #[test]
    fn nmi_timing() {
        run_rom(&get_path("05-nmi_timing.nes"));
    }

    #[test]
    fn suppression() {
        run_rom(&get_path("06-suppression.nes"));
    }

    #[test]
    fn nmi_on_timing() {
        run_rom(&get_path("07-nmi_on_timing.nes"));
    }

    #[test]
    fn nmi_off_timing() {
        run_rom(&get_path("08-nmi_off_timing.nes"));
    }

    #[test]
    fn even_odd_frames() {
        run_rom(&get_path("09-even_odd_frames.nes"));
    }

    #[test]
    fn even_odd_timing() {
        run_rom(&get_path("10-even_odd_timing.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod ppu_read_buffer {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "ppu_read_buffer/";

    #[test]
    fn ppu_read_buffer() {
        run_rom(&get_path("test_ppu_read_buffer.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod ppu_open_bus {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "ppu_open_bus/";

    #[test]
    fn ppu_open_bus() {
        run_rom(&get_path("ppu_open_bus.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod oam_read {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "oam_read/";

    #[test]
    fn oam_read() {
        run_rom(&get_path("oam_read.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

mod oam_stress {
    use crate::common::run_rom;
    use crate::common::ROM_PATH_PREFIX;

    const DIR_PATH: &str = "oam_stress/";

    #[test]
    fn oam_stress() {
        run_rom(&get_path("oam_stress.nes"));
    }

    fn get_path(rom: &str) -> String {
        format!("{}{}{}", ROM_PATH_PREFIX, DIR_PATH, rom)
    }
}

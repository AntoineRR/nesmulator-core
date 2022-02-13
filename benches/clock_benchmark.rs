use criterion::{criterion_group, criterion_main, Criterion};
use nesmulator_core::nes::NES;

const NESTEST_ROM_PATH: &str = "../ROM/Tests/nestest.nes";
const NESTEST_ROM_CLOCKS_TO_REACH_END: u32 = 26560 * 3;

fn criterion_benchmark(c: &mut Criterion) {
    let mut nes = NES::new();
    nes.insert_cartdrige(NESTEST_ROM_PATH).unwrap();
    nes.set_program_counter_at(0xC000);

    // This benchmark runs the nestest ROM in automation repeatedly
    let mut cycle_count = 0;
    c.bench_function("nestest rom", |b| b.iter(|| {
        nes.clock();
        cycle_count += 1;
        if cycle_count % NESTEST_ROM_CLOCKS_TO_REACH_END == 0 {
            nes.restart();
            nes.insert_cartdrige(NESTEST_ROM_PATH).unwrap();
            nes.set_program_counter_at(0xC000);
            cycle_count = 0;
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
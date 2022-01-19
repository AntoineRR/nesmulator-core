use std::sync::{Arc, Mutex};

use sdl2::audio::AudioCallback;

use super::{apu::APU, filters::{HighPassFilter, LowPassFilter}};

pub struct Audio {
    p_apu: Arc<Mutex<APU>>,
    filter1: HighPassFilter,
    filter2: HighPassFilter,
    filter3: LowPassFilter,
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        let mut buffer = self.p_apu.lock().unwrap().get_buffer();
        if buffer.len() == 0 {
            buffer.push(0.0);
        }
        for (i, x) in out.iter_mut().enumerate() {
            *x = buffer[i % buffer.len()]; // The modulo is here for now because the buffer and out are not the same size
            *x = self.apply_filters(*x);
        }
    }
}

impl Audio {
    pub fn new(p_apu: Arc<Mutex<APU>>, sample_rate: f32) -> Self {
        let filter1 = HighPassFilter::new(90, sample_rate);
        let filter2 = HighPassFilter::new(440, sample_rate);
        let filter3 = LowPassFilter::new(14000, sample_rate);
        Audio { p_apu, filter1, filter2, filter3 }
    }

    fn apply_filters(&mut self, amplitude: f32) -> f32 {
        self.filter1.process(self.filter2.process(self.filter3.process(amplitude)))
    }
}
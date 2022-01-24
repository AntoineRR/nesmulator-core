pub trait Filter {
    fn process(&mut self, amplitude: f32) -> f32;
}

pub struct LowPassFilter {
    previous_output: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(frequency: u32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * frequency as f32);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);

        LowPassFilter {
            previous_output: 0.0,
            alpha,
        }
    }
}

impl Filter for LowPassFilter {
    fn process(&mut self, amplitude: f32) -> f32 {
        let processed = self.previous_output + self.alpha * (amplitude - self.previous_output);
        self.previous_output = processed;
        processed
    }
}

pub struct HighPassFilter {
    previous_output: f32,
    previous_input: f32,
    alpha: f32,
}

impl HighPassFilter {
    pub fn new(frequency: u32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * frequency as f32);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);

        HighPassFilter {
            previous_output: 0.0,
            previous_input: 0.0,
            alpha,
        }
    }
}

impl Filter for HighPassFilter {
    fn process(&mut self, amplitude: f32) -> f32 {
        let processed = self.alpha * (self.previous_output + amplitude - self.previous_input);
        self.previous_input = amplitude;
        self.previous_output = processed;
        processed
    }
}

pub struct SweepUnit {
    second: bool,
    enabled: bool,
    period: u8,
    divider: u8,
    negate: bool,
    shift: u8,
    reload: bool,
    pub mute: bool,
}

impl SweepUnit {
    pub fn new(second: bool) -> Self {
        SweepUnit {
            second,
            enabled: false,
            period: 0,
            divider: 0,
            negate: false,
            shift: 0,
            reload: false,
            mute: false,
        }
    }

    pub fn set(&mut self, value: u8) {
        self.enabled = value & 0x80 > 0;
        self.period = (value & 0x70) >> 4;
        self.negate = value & 0x08 > 0;
        self.shift = value & 0x07;
        self.reload = true;
    }

    pub fn clock(&mut self, period: &mut u16) {
        let change = *period >> self.shift;
        let change: i16 = if self.negate {
            if self.second {
                -(change as i16)
            } else {
                -(change as i16) - 1
            }
        } else {
            change as i16
        };
        let target_period = (*period as i16 + change) as u16;
        self.mute = !(8..=0x7FF).contains(&target_period);

        if self.divider == 0 && self.enabled && !self.mute {
            *period = target_period;
        }
        if self.divider == 0 || self.reload {
            self.divider = self.period;
            self.reload = false;
        } else {
            self.divider -= 1;
        }
    }
}

use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// Shrinks `area` to the given constraints and centers it, for popups and
/// login boxes.
pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// Small xorshift64 generator so the simulation widgets do not need the rand
/// crate. Statistical quality is irrelevant here; it only feeds fake data.
pub struct XorShift(u64);

impl XorShift {
    pub fn new(seed: u64) -> Self {
        // Xorshift gets stuck at zero forever, so a zero seed is replaced.
        Self(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// Approximately standard normal, via the sum of twelve uniforms. The sum
    /// has variance exactly 1 and the tails are clipped at +-6, which is fine
    /// for animating prices.
    pub fn gaussian(&mut self) -> f64 {
        (0..12).map(|_| self.unit()).sum::<f64>() - 6.0
    }
}

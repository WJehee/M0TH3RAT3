use std::collections::VecDeque;

use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Bar, BarChart, BarGroup, Block, Chart, Dataset, GraphType},
};

const SIGNAL_LEN: usize = 120;
const BAR_COUNT: usize = 12;
const Y_BOUND: f64 = 2.5;

pub struct Diagnostics {
    signal_a: VecDeque<f64>,
    signal_b: VecDeque<f64>,
    bars_current: [f64; BAR_COUNT],
    bars_target: [f64; BAR_COUNT],
    t: f64,
    rng: u64,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnostics {
    pub fn new() -> Self {
        let mut s = Self {
            signal_a: VecDeque::with_capacity(SIGNAL_LEN),
            signal_b: VecDeque::with_capacity(SIGNAL_LEN),
            bars_current: [0.0; BAR_COUNT],
            bars_target: [50.0; BAR_COUNT],
            t: 0.0,
            rng: 0x9E3779B97F4A7C15,
        };
        for _ in 0..SIGNAL_LEN {
            s.advance_signal();
        }
        for i in 0..BAR_COUNT {
            s.bars_current[i] = s.rand_range(20.0, 80.0);
            s.bars_target[i] = s.rand_range(20.0, 80.0);
        }
        s
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    fn rand_unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    fn rand_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.rand_unit()
    }

    fn advance_signal(&mut self) {
        self.t += 0.18;
        let jitter_a = (self.rand_unit() - 0.5) * 0.35;
        let jitter_b = (self.rand_unit() - 0.5) * 0.35;
        let a = (self.t * 0.55).sin()
            + 0.45 * (self.t * 2.1).sin()
            + jitter_a;
        let b = (self.t * 0.4 + 1.3).sin() * 0.9
            + 0.4 * (self.t * 1.7).cos()
            + jitter_b;
        if self.signal_a.len() == SIGNAL_LEN {
            self.signal_a.pop_front();
            self.signal_b.pop_front();
        }
        self.signal_a.push_back(a);
        self.signal_b.push_back(b);
    }

    pub fn tick(&mut self) {
        self.advance_signal();
        for i in 0..BAR_COUNT {
            let diff = self.bars_target[i] - self.bars_current[i];
            self.bars_current[i] += diff * 0.18;
            if diff.abs() < 1.5 {
                self.bars_target[i] = self.rand_range(15.0, 95.0);
            }
        }
    }
}

fn bar_color(i: usize) -> Color {
    match i % 4 {
        0 => Color::Green,
        1 => Color::Yellow,
        2 => Color::Magenta,
        _ => Color::Red,
    }
}

impl Widget for &Diagnostics {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [signal_area, bars_area] = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .areas(area);

        let data_a: Vec<(f64, f64)> = self
            .signal_a
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v))
            .collect();
        let data_b: Vec<(f64, f64)> = self
            .signal_b
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v))
            .collect();

        let datasets = vec![
            Dataset::default()
                .name("CH-A")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&data_a),
            Dataset::default()
                .name("CH-B")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&data_b),
        ];

        let x_axis = Axis::default()
            .bounds([0.0, SIGNAL_LEN as f64])
            .style(Style::default().fg(Color::DarkGray));
        let y_axis = Axis::default()
            .bounds([-Y_BOUND, Y_BOUND])
            .style(Style::default().fg(Color::DarkGray));

        let chart = Chart::new(datasets)
            .block(
                Block::bordered()
                    .title("RF SIGNAL".bold())
                    .title_alignment(Alignment::Center),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);
        chart.render(signal_area, buf);

        let labels: Vec<String> = (0..BAR_COUNT).map(|i| format!("{i:02}")).collect();
        let bars: Vec<Bar> = self
            .bars_current
            .iter()
            .enumerate()
            .map(|(i, v)| {
                Bar::default()
                    .value(*v as u64)
                    .label(Line::from(labels[i].clone()))
                    .style(Style::default().fg(bar_color(i)))
            })
            .collect();

        let bar_chart = BarChart::default()
            .block(
                Block::bordered()
                    .title("SPECTRUM".bold())
                    .title_alignment(Alignment::Center),
            )
            .data(BarGroup::default().bars(&bars))
            .max(100)
            .bar_width(3)
            .bar_gap(1);
        bar_chart.render(bars_area, buf);
    }
}

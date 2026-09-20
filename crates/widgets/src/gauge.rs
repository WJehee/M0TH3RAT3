use ratatui::{prelude::*, widgets::*};

/// A bordered gauge with a title and a `value/max` label, used for resource
/// bars, fuel levels and track progress.
#[derive(Debug)]
pub struct LabeledGauge {
    name: String,
    value: f64,
    max_value: f64,
    color: Color,
}

impl LabeledGauge {
    pub fn new(name: &str, value: f64, max_value: f64, color: Color) -> Self {
        Self {
            name: name.to_string(),
            value,
            max_value,
            color,
        }
    }
}

impl Widget for &LabeledGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = format!("{}/{}", self.value, self.max_value);
        // Values beyond the maximum would make Gauge panic on a ratio above 1.
        let ratio = (self.value / self.max_value).clamp(0.0, 1.0);
        let gauge = Gauge::default()
            .block(Block::bordered().title(self.name.clone()))
            .gauge_style(
                Style::default()
                    .fg(self.color)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .label(label)
            .ratio(ratio);
        gauge.render(area, buf);
    }
}

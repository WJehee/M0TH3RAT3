use ratatui::{prelude::*, widgets::*};
use symbols::border;

#[derive(Debug)]
pub struct MyGauge {
    name: String,
    value: f64,
    max_value: f64,
    color: Color,
}

impl MyGauge {
    pub fn new(name: &str, value: f64, max_value: f64, color: Color) -> Self {
        Self {
            name: name.to_string(),
            value,
            max_value,
            color,
        }
    }
}

impl Widget for &MyGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = format!("{}/{}", self.value, self.max_value);
        let gauge = Gauge::default()
           .block(Block::bordered().title(self.name.clone()))
           .gauge_style(
               Style::default()
               .fg(self.color)
               .bg(Color::Black)
               .add_modifier(Modifier::ITALIC),
           )
            .label(label)
            .percent(((self.value / self.max_value) * 100.0) as u16);
        gauge.render(area, buf);
    }
}

#[derive(Default)]
pub struct ShipStatus;

impl Widget for &ShipStatus {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title_bottom(" Ship Status ".bold())
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let [crystals, fuel] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .areas(inner);

        MyGauge::new("Kristallen", 10.0, 100.0, Color::Magenta).render(crystals, buf);
        MyGauge::new("Brandstof", 80.0, 100.0, Color::Red).render(fuel, buf);
    }
}

use ratatui::{prelude::*, widgets::*};

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
pub struct Resources {
    pub crystals: i32,
    pub fuel: i32,
    pub components: i32,
}

impl Widget for &Resources {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [crystals, fuel, components] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .areas(area);

        MyGauge::new("Kristallen", self.crystals as f64, 100.0, Color::Magenta).render(crystals, buf);
        MyGauge::new("Brandstof", self.fuel as f64, 100.0, Color::Red).render(fuel, buf);
        MyGauge::new("Componenten", self.components as f64, 16.0, Color::DarkGray).render(components, buf);
    }
}

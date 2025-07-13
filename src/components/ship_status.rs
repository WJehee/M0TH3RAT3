use ratatui::{prelude::*, widgets::*};
use symbols::border;

#[derive(Debug)]
pub struct MyGauge {
    name: String,
    value: f64,
    color: Color,
}

impl MyGauge {
    pub fn new(name: &str, value: f64, color: Color) -> Self {
        Self {
            name: name.to_string(),
            value,
            color,
        }
    }
}

impl Widget for &MyGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        //let gauge = Gauge::default()
        //    .block(Block::bordered().title(self.name.clone()))
        //    .gauge_style(
        //        Style::default()
        //        .fg(self.color)
        //        .bg(Color::Black)
        //        .add_modifier(Modifier::ITALIC),
        //    )
        //    .percent(self.value);
        //gauge.render(area, buf);
        
        let line_gauge = LineGauge::default()
            .block(Block::bordered().title(self.name.clone()))
            .filled_style(
                Style::default()
                .fg(self.color)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
            )
            .line_set(symbols::line::THICK)
            .ratio(self.value);
        line_gauge.render(area, buf);
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

        let [shields, power, fuel] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .areas(inner);

        MyGauge::new("Shields", 0.9, Color::Blue).render(shields, buf);
        MyGauge::new("Power", 0.5, Color::Yellow).render(power, buf);
        MyGauge::new("Fuel", 0.3, Color::Red).render(fuel, buf);
    }
}

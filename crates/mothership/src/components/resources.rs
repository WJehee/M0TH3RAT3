use ratatui::prelude::*;
use widgets::LabeledGauge;

#[derive(Default)]
pub struct Resources {
    pub crystals: i32,
    pub fuel: i32,
    pub reputation: i32,
    pub components: i32,
}

impl Widget for &Resources {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [crystals, fuel, reputation, components] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .areas(area);

        LabeledGauge::new("Kristallen", self.crystals as f64, 200.0, Color::Magenta).render(crystals, buf);
        LabeledGauge::new("Brandstof", self.fuel as f64, 100.0, Color::Red).render(fuel, buf);
        LabeledGauge::new("Reputatie", self.reputation as f64, 100.0, Color::Yellow).render(reputation, buf);
        LabeledGauge::new("Componenten", self.components as f64, 50.0, Color::DarkGray).render(components, buf);
    }
}


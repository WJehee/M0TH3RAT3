//! Navigation screen. Until a GPS receiver is wired in this shows a compass
//! rose and a fixed demo route so the layout can be tuned on the real screen.
//!
//! TODO: feed `heading`, `speed_kmh` and `position` from gpsd (or a serial NMEA
//! stream) and replace the demo route with tiles or a routed polyline.

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Points},
        Block, Paragraph,
    },
};

const DEMO_ROUTE: [(f64, f64); 6] = [
    (5.0, 5.0),
    (20.0, 12.0),
    (35.0, 30.0),
    (50.0, 45.0),
    (72.0, 60.0),
    (90.0, 88.0),
];

pub struct Navigation {
    /// Degrees clockwise from north.
    pub heading: f64,
    pub speed_kmh: f64,
    /// Index into `DEMO_ROUTE` of the waypoint the car is at.
    waypoint: usize,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            heading: 0.0,
            speed_kmh: 0.0,
            waypoint: 0,
        }
    }
}

impl Navigation {
    /// Keyboard stand-in for the GPS: turn with the arrow keys and step along
    /// the demo route with Enter.
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => self.heading = (self.heading - 15.0).rem_euclid(360.0),
            KeyCode::Right => self.heading = (self.heading + 15.0).rem_euclid(360.0),
            KeyCode::Enter => self.waypoint = (self.waypoint + 1) % DEMO_ROUTE.len(),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        // Fake speed that follows the heading so the readout is not static.
        self.speed_kmh = 50.0 + 30.0 * (self.heading.to_radians()).sin().abs();
    }

    fn cardinal(&self) -> &'static str {
        const NAMES: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
        NAMES[((self.heading + 22.5) / 45.0) as usize % 8]
    }
}

impl Widget for &Navigation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [readout, map] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        let text = Line::from(vec![
            format!(" {:>5.1} km/h ", self.speed_kmh).bold().fg(Color::Green),
            "  heading ".into(),
            format!("{:>3.0}° {}", self.heading, self.cardinal()).bold().fg(Color::Yellow),
            "  next waypoint ".into(),
            format!("{}/{}", self.waypoint + 1, DEMO_ROUTE.len()).bold(),
        ]);
        Paragraph::new(text)
            .block(Block::bordered())
            .render(readout, buf);

        let here = DEMO_ROUTE[self.waypoint];
        let heading = self.heading.to_radians();
        Canvas::default()
            .block(Block::bordered().title("ROUTE").title_alignment(Alignment::Center))
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                for pair in DEMO_ROUTE.windows(2) {
                    ctx.draw(&CanvasLine {
                        x1: pair[0].0,
                        y1: pair[0].1,
                        x2: pair[1].0,
                        y2: pair[1].1,
                        color: Color::DarkGray,
                    });
                }
                ctx.draw(&Points {
                    coords: &DEMO_ROUTE,
                    color: Color::White,
                });
                // Heading indicator: a short line from the car in the direction
                // of travel. Canvas y grows upward, so north is +y.
                ctx.draw(&CanvasLine {
                    x1: here.0,
                    y1: here.1,
                    x2: here.0 + 8.0 * heading.sin(),
                    y2: here.1 + 8.0 * heading.cos(),
                    color: Color::Green,
                });
                ctx.print(here.0, here.1, "●".green().bold());
            })
            .render(map, buf);
    }
}

use std::time::Duration;

use ratatui::{crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{canvas::{Canvas}, Block, Gauge}};

use crate::{objects::Planet, util::{Event, WARP_HOLD_DURATION}};

pub struct StarMap {
    pub planets: Vec<Planet>,
    selected_location: usize,
    current_location: usize,
    warp_progress: f64,
}

impl StarMap {
    pub fn new(locations: Vec<Planet>) -> Self {
        StarMap { 
            planets: locations,
            selected_location: 0,
            current_location: 0,
            warp_progress: 0.0,
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent, last_key_pressed: Option<KeyEvent>, last_press_time: std::time::Instant, username: String) -> Vec<Event> {
        match key_event.code {
            KeyCode::Left   => { self.selected_location = (self.selected_location + self.planets.len() - 1) % self.planets.len() },
            KeyCode::Right  => { self.selected_location = (self.selected_location + self.planets.len() + 1) % self.planets.len() },
            KeyCode::Enter  => { 
                if let Some(key) = last_key_pressed {
                    if self.current_location != self.selected_location {
                        if key == key_event {
                            self.warp_progress = last_press_time.elapsed().as_secs_f64() / Duration::from_secs(WARP_HOLD_DURATION).as_secs_f64();
                            if self.warp_progress > 1.0 {
                                self.warp_progress = 1.0;
                            }
                        }
                        if last_press_time.elapsed() > Duration::from_secs(WARP_HOLD_DURATION) {
                            self.current_location = self.selected_location;
                            self.warp_progress = 0.0;
                        }
                    }
                }
            },
            KeyCode::Char('e') => {
                // Explore planet
                return self.planets[self.current_location].visit(username);
            },
            _ => {},
        }
        Vec::new()
    }
}

impl Widget for &StarMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main, bar] = Layout::vertical([
            Constraint::Percentage(95),
            Constraint::Percentage(5),
        ]).areas(area);

        Canvas::default()
            .paint(|ctx| {
                // Draw each location
                for (i, location) in self.planets.iter().enumerate() {
                    if i == self.current_location { location.draw_current(ctx); }
                    if i == self.selected_location {
                        location.draw(ctx, Some(Color::White));
                    } else {
                        location.draw(ctx, None);
                    }
                }
            })
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .render(main, buf);

        let line_gauge = Gauge::default()
            .block(Block::bordered().title("Warp"))
            .style(
                Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
            )
            .ratio(self.warp_progress);
        line_gauge.render(bar, buf);
    }
}


use std::time::Duration;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{
        canvas::{
            Canvas, Circle, Rectangle,
        }, Block, Gauge, Widget
    }
};

use crate::{objects::SolarSystem, util::{within_radius, Event, ItemDiff, WARP_HOLD_DURATION}};

const MOVE_DISTANCE: f64 = 0.1;
const STAR_DISTANCE: f64 = MOVE_DISTANCE * 2.0;
const WARP_DISTANCE: f64 = MOVE_DISTANCE * 20.0;

pub struct GalacticMap {
    solar_systems: Vec<SolarSystem>,
    pub current_system: Option<SolarSystem>,
    pub current_pos: (f64, f64),
    selected_pos: (f64, f64),
    warp_progress: f64,
    warped: bool,
}

impl GalacticMap {
    pub fn new(solar_systems: Vec<SolarSystem>, pos: (f64, f64)) -> Self {
        GalacticMap {
            solar_systems,
            current_system: None,
            current_pos: pos,
            selected_pos: pos,
            warp_progress: 0.0,
            warped: false,
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent, last_key_pressed: Option<KeyEvent>, last_press_time: std::time::Instant, can_warp: bool) -> Vec<Event> {
        if key_event.code != KeyCode::Enter {
            self.warped = false;
        }
        match key_event.code {
            KeyCode::Char('a') => { self.selected_pos.0 -= MOVE_DISTANCE; },
            KeyCode::Char('d') => { self.selected_pos.0 += MOVE_DISTANCE; },
            KeyCode::Char('w') => { self.selected_pos.1 += MOVE_DISTANCE; },
            KeyCode::Char('s') => { self.selected_pos.1 -= MOVE_DISTANCE; },
            KeyCode::Enter => {
                if let Some(key) = last_key_pressed {
                    if within_radius(self.selected_pos, self.current_pos, WARP_DISTANCE) && can_warp {
                        if key == key_event {
                            self.warp_progress = last_press_time.elapsed().as_secs_f64() / Duration::from_secs(WARP_HOLD_DURATION).as_secs_f64();
                            if self.warp_progress > 1.0 {
                                self.warp_progress = 1.0;
                            }
                        }
                        if last_press_time.elapsed() > Duration::from_secs(WARP_HOLD_DURATION) {
                            self.warp_progress = 0.0;

                            // Stop draining fuel after 1 time
                            if !self.warped {
                                self.warped = true;
                                self.current_pos = self.selected_pos;
                                let mut events = Vec::new();
                                // Deplete fuel
                                events.push(Event::Item(ItemDiff{
                                    crystals: 0,
                                    fuel: -1,
                                    components: 0,
                                }));

                                if let Some(system) = self.check_for_systems() {
                                    self.current_system = system.clone();
                                    events.push(Event::NewSystem(system));
                                }

                                return events;
                            }
                        }
                    }
                }
            },
            _ => {},
        };
        Vec::new()
    }

    pub fn check_for_systems(&self) -> Option<Option<SolarSystem>> {
        let mut none_in_range = true;
        for system in &self.solar_systems {
            if within_radius(self.current_pos, system.pos, STAR_DISTANCE) {
                none_in_range = false;
                if let Some(current) = &self.current_system {
                    if system.name != current.name {
                        return Some(Some(system.clone()));
                    }
                } else {
                    return Some(Some(system.clone()));
                }
            }
        }
        if none_in_range {
            return Some(None);
        }
        None
    }
}

impl Widget for &GalacticMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [pos, main, bar] = Layout::vertical([
            Constraint::Percentage(3),
            Constraint::Percentage(92),
            Constraint::Percentage(5),
        ]).areas(area);

        let [current, _, selected] = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ]).areas(pos);

        Canvas::default()
            .paint(|ctx| {
                for system in &self.solar_systems {
                    ctx.draw(&Rectangle{
                        x: system.pos.0,
                        y: system.pos.1,
                        width: MOVE_DISTANCE,
                        height: MOVE_DISTANCE,
                        color: Color::White,
                    });
                }
                // Draw selected position
                ctx.draw(&Circle{
                    x: self.selected_pos.0,
                    y: self.selected_pos.1,
                    radius: STAR_DISTANCE,
                    color: Color::White,
                });
                // Draw possible warp radius
                ctx.draw(&Circle{
                    x: self.current_pos.0,
                    y: self.current_pos.1,
                    radius: WARP_DISTANCE,
                    color: Color::Gray,
                });
                // Draw current position
                ctx.draw(&Circle{
                    x: self.current_pos.0,
                    y: self.current_pos.1,
                    radius: 0.05,
                    color: Color::Blue,
                });
            })
            .x_bounds([0.0, 30.0])
            .y_bounds([0.0, 30.0])
            .render(main, buf);

        Line::from(format!("[{:.1}, {:.1}]", self.current_pos.0, self.current_pos.1)).alignment(Alignment::Left).render(current, buf);
        Line::from(format!("[{:.1}, {:.1}]", self.selected_pos.0, self.selected_pos.1)).alignment(Alignment::Right).render(selected, buf);

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


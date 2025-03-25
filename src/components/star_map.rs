use std::rc::Rc;

use ratatui::{crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{canvas::{Canvas, Circle, Context, Line}}};

#[derive(Debug)]
struct Location {
    x: f64,
    y: f64,
    radius: f64,
    color: Color,
}

impl Location {
    pub fn new(x: f64, y: f64, radius: f64, color: Color) -> Self {
        Self {
            x, y, radius, color,
        }
    }
    
    pub fn draw(&self, ctx: &mut Context, highlighted: Option<Color>) {
        ctx.draw(&Circle {
            x: self.x,
            y: self.y,
            radius: self.radius,
            color: self.color,
        });
        ctx.draw(&Circle {
            x: self.x,
            y: self.y,
            radius: self.radius * 1.7,
            color: highlighted.unwrap_or(Color::DarkGray),
        });
    }

    pub fn draw_current(&self, ctx: &mut Context) {
        ctx.print(
            self.x-(self.radius/2.0), self.y+(self.radius*2.0),
            "You are here".green().bold()
        );
    }
}

#[derive(Debug)]
pub struct StarMap {
    locations: Vec<Location>,
    selected_location: usize,
    current_location: usize,
}

impl StarMap {
    pub fn new() -> Self {
        let mut locations = Vec::new();
        let p1 = Location::new(50.0, 80.0, 5.0, Color::Magenta);
        let p2 = Location::new(30.0, 20.0, 8.0, Color::Red);
        let p3 = Location::new(80.0, 30.0, 3.0, Color::LightGreen);
        locations.push(p1);
        locations.push(p2);
        locations.push(p3);
        StarMap { 
            locations,
            selected_location: 0,
            current_location: 0,
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left   => { self.selected_location = (self.selected_location + self.locations.len() - 1) % self.locations.len() },
            KeyCode::Right  => { self.selected_location = (self.selected_location + self.locations.len() + 1) % self.locations.len() },
            KeyCode::Enter  => { 
                if self.current_location == self.selected_location {
                    // TODO: display popup with location info
                } else {
                    // TODO: fill up gauge before travelling by holding / not cancelling
                    self.current_location = self.selected_location
                }
            }
            _ => {},
        }
    }
}

impl Widget for &StarMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .paint(|ctx| {
                // Draw each location
                for (i, location) in self.locations.iter().enumerate() {
                    if i == self.current_location { location.draw_current(ctx); }
                    if i == self.selected_location {
                        location.draw(ctx, Some(Color::White));
                    } else {
                        location.draw(ctx, None);
                    }
                }
                // Draw lines between locations
                ctx.draw(&Line {
                    x1: 50.0,
                    y1: 80.0,
                    x2: 30.0,
                    y2: 20.0,
                    color: Color::DarkGray,
                });
                ctx.draw(&Line {
                    x1: 30.0,
                    y1: 20.0,
                    x2: 80.0,
                    y2: 30.0,
                    color: Color::DarkGray,
                });
                // TODO: highlight selected position on link and on location
            })
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .render(area, buf);
    }
}


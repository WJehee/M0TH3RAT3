use std::collections::HashMap;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{
        canvas::{
            Canvas, Circle, Points, 
        }, LineGauge, Widget
    }
};

const MOVE_DISTANCE: f64 = 0.1;

#[derive(Debug)]
struct SolarSystem {
    name: String,
    
}

#[derive(Debug)]
pub struct GalacticMap {
    coords: Vec<(f64, f64)>,
    systems: HashMap<(f64, f64), SolarSystem>,
    current_pos_x: f64,
    current_pos_y: f64,
}

impl GalacticMap {
    pub fn new() -> Self {
        GalacticMap {
            coords: vec![(7.0, 8.0), (3.0, 2.0), (4.0, 5.0)],
            systems: HashMap::new(),
            current_pos_x: 0.0,
            current_pos_y: 0.0,
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('w') => { self.current_pos_y += MOVE_DISTANCE; },
            KeyCode::Char('s') => { self.current_pos_y -= MOVE_DISTANCE; },
            KeyCode::Char('a') => { self.current_pos_x -= MOVE_DISTANCE; },
            KeyCode::Char('d') => { self.current_pos_x += MOVE_DISTANCE; },
            _ => {},
        }
    }
}

impl Widget for &GalacticMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .paint(|ctx| {
                ctx.draw(&Points {coords: &self.coords, color: Color::White});
                ctx.draw(&Circle{
                    x: self.current_pos_x,
                    y: self.current_pos_y,
                    radius: MOVE_DISTANCE * 2.0,
                    color: Color::Gray,
                });
            })
            .x_bounds([-10.0, 10.0])
            .y_bounds([-10.0, 10.0])
            .render(area, buf);

    }
}


use std::collections::HashMap;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{
        canvas::{
            Canvas, Circle, Points, 
        }, Widget
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
    current_pos: (f64, f64),
    selected_pos: (f64, f64),
}

impl GalacticMap {
    pub fn new() -> Self {
        GalacticMap {
            coords: vec![(7.0, 8.0), (3.0, 2.0), (4.0, 5.0)],
            systems: HashMap::new(),
            current_pos: (0.0, 0.0),
            selected_pos: (0.0, 0.0),
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('a') => { self.selected_pos.0 -= MOVE_DISTANCE; },
            KeyCode::Char('d') => { self.selected_pos.0 += MOVE_DISTANCE; },
            KeyCode::Char('w') => { self.selected_pos.1 += MOVE_DISTANCE; },
            KeyCode::Char('s') => { self.selected_pos.1 -= MOVE_DISTANCE; },
            _ => {},
        }
    }
}

impl Widget for &GalacticMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .paint(|ctx| {
                ctx.draw(&Points {coords: &self.coords, color: Color::White});
                // Draw selected position
                ctx.draw(&Circle{
                    x: self.selected_pos.0,
                    y: self.selected_pos.1,
                    radius: MOVE_DISTANCE * 2.0,
                    color: Color::White,
                });
                // Draw possible warp radius
                ctx.draw(&Circle{
                    x: self.current_pos.0,
                    y: self.current_pos.1,
                    radius: MOVE_DISTANCE * 20.0,
                    color: Color::Gray,
                });
            })
            .x_bounds([-10.0, 10.0])
            .y_bounds([-10.0, 10.0])
            .render(area, buf);

    }
}


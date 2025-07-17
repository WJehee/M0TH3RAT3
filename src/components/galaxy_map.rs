use std::time::Duration;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{
        canvas::{
            Canvas, Circle, Points, 
        }, Block, Gauge, Widget
    }
};

use crate::util::WARP_HOLD_DURATION;

const MOVE_DISTANCE: f64 = 0.1;

#[derive(Debug)]
pub struct GalacticMap {
    coords: Vec<(f64, f64)>,
    // systems: HashMap<(f64, f64), SolarSystem>,
    current_pos: (f64, f64),
    selected_pos: (f64, f64),
    warp_progress: f64,
}

impl GalacticMap {
    pub fn new() -> Self {
        GalacticMap {
            coords: vec![(7.0, 8.0), (3.0, 2.0), (4.0, 5.0)],
            // systems: HashMap::new(),
            current_pos: (0.0, 0.0),
            selected_pos: (0.0, 0.0),
            warp_progress: 0.0,
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent, last_key_pressed: Option<KeyEvent>, last_press_time: std::time::Instant) {
        match key_event.code {
            KeyCode::Char('a') => { self.selected_pos.0 -= MOVE_DISTANCE; },
            KeyCode::Char('d') => { self.selected_pos.0 += MOVE_DISTANCE; },
            KeyCode::Char('w') => { self.selected_pos.1 += MOVE_DISTANCE; },
            KeyCode::Char('s') => { self.selected_pos.1 -= MOVE_DISTANCE; },
            KeyCode::Enter => {
                if let Some(key) = last_key_pressed {
                    if key == key_event {
                        self.warp_progress = last_press_time.elapsed().as_secs_f64() / Duration::from_secs(WARP_HOLD_DURATION).as_secs_f64();
                        if self.warp_progress > 1.0 {
                            self.warp_progress = 1.0;
                        }
                    }
                    if last_press_time.elapsed() > Duration::from_secs(WARP_HOLD_DURATION) {
                        self.current_pos = self.selected_pos;
                        self.warp_progress = 0.0;
                    }
                }
            },
            _ => {},
        }
    }
}

impl Widget for &GalacticMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main, bar] = Layout::vertical([
            Constraint::Percentage(95),
            Constraint::Percentage(5),
        ]).areas(area);

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


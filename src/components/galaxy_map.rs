use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{
        canvas::{
            Canvas, Points, Rectangle
        }, Block, Widget
    }
};

const MOVE_DISTANCE: f64 = 0.08;

#[derive(Debug)]
enum SolarSystem {
    None,
    Something
}

#[derive(Debug)]
pub struct GalaxyMap {
    coords: Vec<(f64, f64)>,
    current_pos_x: f64,
    current_pos_y: f64,
}

impl GalaxyMap {
    pub fn new() -> Self {
        GalaxyMap {
            coords: vec![(7.0, 8.0), (3.0, 2.0), (4.0, 5.0)],
            current_pos_x: 0.0,
            current_pos_y: 0.0,
        }
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => { self.current_pos_x -= MOVE_DISTANCE; },
            KeyCode::Right => { self.current_pos_x += MOVE_DISTANCE; },
            KeyCode::Up => { self.current_pos_y -= MOVE_DISTANCE; },
            KeyCode::Down => { self.current_pos_y += MOVE_DISTANCE; },
            _ => {},
        }
    }
}

impl Widget for &GalaxyMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ratio = (area.height as f32) / (area.width as f32);

        let square_size = 10;
        // let square = Rect::default()
        //     .block(Block::default());

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage((ratio * 100.0) as u16),
                Constraint::Min(0),
            ])
                .split(area);

        Canvas::default()
            .block(Block::bordered().title("Galaxy map"))
            .paint(|ctx| {
                ctx.draw(&Points {coords: &self.coords, color: Color::White});
                ctx.draw(&Rectangle {
                    x: self.current_pos_x,
                    y: self.current_pos_y,
                    width: MOVE_DISTANCE * 2.0,
                    height: MOVE_DISTANCE * 2.0,
                    color: Color::Gray,
                });
            })
            .x_bounds([-10.0, 10.0])
            .y_bounds([-10.0, 10.0])
            .render(layout[1], buf);
    }
}


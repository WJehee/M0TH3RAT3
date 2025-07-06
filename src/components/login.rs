use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::Widget
};

use bincode::{Decode, Encode};

#[derive(Decode, Encode, Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginScreen {
    pub username: String,
    pub password: String,
    on_password: bool,
}

impl LoginScreen {
    pub fn new() -> LoginScreen {
        LoginScreen { 
            username: String::new(),
            password: String::new(),
            on_password: false,
        }
    }
}

impl LoginScreen {
    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(char) => match self.on_password {
                false => self.username.push(char),
                true => self.password.push(char),
            },
            KeyCode::Backspace => match self.on_password {
                false => { self.username.pop(); },
                true => { self.password.pop(); },
            },
            KeyCode::Tab => self.on_password = !self.on_password,
            _ => {},
        }
    }
}

impl Widget for &LoginScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(vec![
            Line::from("username"),
            Line::from(self.username.clone()),
            Line::from("password"),
            Line::from("*".repeat(self.password.len())),
        ]);
        text.render(area, buf);
    }
}


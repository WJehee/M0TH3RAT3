use ratatui::{
    prelude::*,
    widgets::Widget,
    crossterm::event::{KeyEvent},
};

use bincode::{Decode, Encode};

#[derive(Decode, Encode, Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginScreen;

impl LoginScreen {
    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
    }
}

impl Widget for &LoginScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}


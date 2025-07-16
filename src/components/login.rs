use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, prelude::*, widgets::{Block, Paragraph, Widget}
};
use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password: String,
    pub pos_x: f64,
    pub pox_y: f64,
}

#[derive(Debug)]
pub struct LoginScreen {
    pub username: String,
    pub password: String,
    password_selected: bool,
    user_list: Vec<User>,
}

impl LoginScreen {
    pub fn new(user_list: Vec<User>) -> LoginScreen {
        LoginScreen { 
            username: String::new(),
            password: String::new(),
            password_selected: false,
            user_list,
        }
    }
}

impl LoginScreen {
    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(char) => match self.password_selected {
                false => self.username.push(char),
                true => self.password.push(char),
            },
            // For some reason this does not work in cool-retro-term, something weird with stty and
            // command codes, it uses ^H instead of ^?
            KeyCode::Backspace => match self.password_selected {
                false => { self.username.pop(); },
                true => { self.password.pop(); },
            },
            KeyCode::Tab => self.password_selected = !self.password_selected,
            _ => {},
        }
    }

    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.password_selected = false;
    }
}

impl Widget for &LoginScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = Text::from(util::TITLE_HEADER)
            .fg(Color::Green);

        let lines: Vec<_> = vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from("USERNAME".red()),
            Line::from(self.username.clone().white()),
            Line::from("PASSWORD".red()),
            Line::from("*".repeat(self.password.len()).white()),
        ];
        text.extend(Text::from(lines));

        let area = util::center(
            area,
            Constraint::Length((text.width()+10) as u16),
            Constraint::Length((text.height()+5) as u16),
        );

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Double);

        Paragraph::new(text)
            .block(block)
            .centered()
            .render(area, buf);
    }
}


use std::io;
use std::time::Instant;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind}, prelude::*, widgets::{Block, Paragraph, Widget}
};
use tachyonfx::{fx, EffectManager};

use crate::{tui, user::User, util};

pub struct LoginScreen {
    exit: bool,
    pub username: String,
    pub password: String,
    password_selected: bool,
    user_list: Vec<User>,
    user: Option<User>,
    effects: EffectManager<()>,
}

impl LoginScreen {
    pub fn new(user_list: Vec<User>) -> LoginScreen {
        let mut effects: EffectManager<()> = EffectManager::default();
        effects.add_effect(
            fx::prolong_start(0, fx::coalesce(3000))
        );
        LoginScreen { 
            exit: false,
            username: String::new(),
            password: String::new(),
            password_selected: false,
            user_list,
            user: None,
            effects,
        }
    }
}

impl LoginScreen {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<User> {
        let mut last_frame = Instant::now();

        while !self.exit {
            let elapsed = last_frame.elapsed();
            last_frame = Instant::now();

            terminal.draw(|frame| {
                let area = frame.area();
                frame.render_widget(&mut *self, area);
                self.effects.process_effects(elapsed.into(), frame.buffer_mut(), area);
            })?;
            self.handle_events()?;
        }
        Ok(self.user.clone().expect("user to be set"))
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_press_event(key);
                }
            }
        } 
        Ok(())
    }

    fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc        => { self.exit = true; },
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
            KeyCode::Enter => {
                self.user = self.try_login(self.username.clone(), self.password.clone());
                if self.user == None {
                    // TODO: show an error popup given login has failed
                    self.clear();
                } else {
                    // Successfull login
                    self.exit = true;
                }
            },
            _ => {},
        }
    }

    fn try_login(&mut self, username: String, password: String) -> Option<User> {
        for user in self.user_list.iter_mut() {

            if user.password == "" {
                let parts: Vec<&str> = password.split("-").collect();
                if parts[0] == user.password_start {
                    if user.password_attempts >= user.password_attempts_max {
                        user.password = password.clone();
                    } else {
                        user.password_attempts += 1;
                    }
                }
            }

            if user.username == username && user.password == password {
                return Some(user.clone());
            }
        }
        None
    }

    fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.password_selected = false;
    }
}

impl Widget for &mut LoginScreen {
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


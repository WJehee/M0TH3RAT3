use std::{io, time::Instant};

use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        Block, List, ListState, Paragraph, Widget
    },
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
};
use tachyonfx::{fx, EffectManager};

use crate::{components::{galaxy_map::GalaxyMap, login::{LoginScreen, User}, ship_status::ShipStatus, star_map::StarMap}, storage::Storage, tui, util};

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive, strum::AsRefStr)]
enum MenuItem {
    StarMap = 0,
    Crew,
    Info,
}

#[derive(Debug)]
struct MenuState {
    list_state: ListState,
    selected: MenuItem,
    active: MenuItem,
}

impl MenuState {
    fn select(&mut self, offset: i8) {
        let current = self.selected as i8;
        let next = current + offset;
        if next == -1 { 
            // Set to last item in the list
            self.selected = MenuItem::Info
        } else {
            self.selected = match FromPrimitive::from_i8(next) {
                Some(d2) => d2,
                None => FromPrimitive::from_u8(0).unwrap(),
            };
        }
        self.list_state.select(self.selected.to_usize());
    }

    fn activate(&mut self) {
        self.active = self.selected;
    }
}

pub struct App {
    exit: bool,
    storage: Storage,
    effects: EffectManager<()>,

    // Login requirements
    user: Option<User>,
    loginscreen: LoginScreen,

    // Post login
    menu: MenuState,
    starmap: StarMap,
    galaxy: GalaxyMap,
}

impl App {
    pub fn new(storage_path: Option<String>) -> Self {
        let storage = match storage_path {
            Some(path) => Storage::load(path).expect("storage path to be valid"),
            None => Storage::new(),
        };
        let mut effects: EffectManager<()> = EffectManager::default();

        effects.add_effect(fx::coalesce(3000));
        
        Self {
            exit: false,
            storage: storage,
            effects: effects,

            user: None,
            loginscreen: LoginScreen::new(),

            menu: MenuState {
                list_state: ListState::default().with_selected(Some(0)),
                selected: MenuItem::StarMap,
                active: MenuItem::StarMap,
            },
            starmap: StarMap::new(),
            galaxy: GalaxyMap::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let mut last_frame = Instant::now();

        while !self.exit {
            let elapsed = last_frame.elapsed();
            last_frame = Instant::now();

            terminal.draw(|frame| {
                self.render_frame(frame);
                let area = frame.area();
                self.effects.process_effects(elapsed.into(), frame.buffer_mut(), area);
            })?;

            self.handle_events()?;

        }
        let copy = self.storage.clone();
        let _ = copy.save();
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.galaxy.handle_press_event(key);
                    self.handle_press_event(key);
                    // match self.menu.selected {
                    //     MenuItem::StarMap => { self.starmap.handle_press_event(key); },
                    //     _ => {}
                    // }
                }
            }
        }
        Ok(())
    }

    fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Key's for all widgets
            KeyCode::Esc        => { self.exit = true; },
            // Login screen handler
            KeyCode::Enter if self.user == None => {
                self.user = self.storage.try_login(&self.loginscreen.username, &self.loginscreen.password);
                if self.user == None {
                    // TODO: show an error popup given login has failed
                    self.loginscreen.clear();
                } else {
                    // Successfull login, add new transition effect
                    self.effects.add_effect(fx::coalesce(1000));
                }
            },
            _ if self.user == None => self.loginscreen.handle_press_event(key_event),

            // Other
            KeyCode::Up         => { self.menu.select(-1); },
            KeyCode::Down       => { self.menu.select(1); },
            KeyCode::Enter      => { self.menu.activate(); },
            _ => {},
        }
    }

    fn render_title(&mut self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Select ".into(),
            "<Enter>".green().bold(),
            " Move up ".into(),
            "<Up>".green().bold(),
            " Move down ".into(),
            "<Down>".green().bold(),
            " Quit ".into(),
            "<Esc> ".green().bold(),
        ]);
        let block = Block::bordered()
            .title_bottom(instructions)
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);

        let mut text = Text::from(util::TITLE_HEADER)
            .fg(Color::Green);

        let user = self.user.clone().expect("we are past login at this point");
        text.extend(Line::from(
            format!("Logged in as: {}", user.username.clone())
        ));

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let [_padding_top, menu_pos, _padding_bottom] = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ]).areas(area);

        let menu = List::new([
            Line::from(MenuItem::StarMap.as_ref()).alignment(Alignment::Center),
            Line::from(MenuItem::Crew.as_ref()).alignment(Alignment::Center),
            Line::from(MenuItem::Info.as_ref()).alignment(Alignment::Center),
        ])
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default()
                .bold()
                .fg(Color::Green)
            )
            .repeat_highlight_symbol(true);

        ratatui::prelude::StatefulWidget::render(menu, menu_pos, buf, &mut self.menu.list_state);
    }
}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // self.galaxy.render(area, buf);
        // return;
       
        if self.user == None {
            self.loginscreen.render(area, buf);
            return;
        }

        let [left, right] = Layout::horizontal([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ]).areas(area);

        let [title, list, ship_status] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(25),
                Constraint::Percentage(35),
            ])
            .areas(left);

        self.render_title(title, buf);
        self.render_list(list, buf);
        ShipStatus.render(ship_status, buf);

        let block = Block::bordered()
            .title(self.menu.active.as_ref().bold())
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);
        let inner = block.inner(right);
        block.render(right, buf);

        match self.menu.active {
            MenuItem::StarMap   => { self.starmap.render(inner, buf); },
            MenuItem::Crew      => {},
            MenuItem::Info      => {},
        }
    }
}


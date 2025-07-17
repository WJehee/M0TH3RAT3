use std::{fmt, io, time::{Duration, Instant}};

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

use crate::{
    storage::Storage, 
    user::{User},
    components::{crew::CrewStatus, galaxy_map::GalacticMap, resources::Resources, star_map::StarMap},
    tui, util,
};

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
enum MenuItem {
    GalacticMap = 0,
    StarMap,
    Crew,
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            MenuItem::GalacticMap => "Sterren kaart",
            MenuItem::StarMap => "Zonnestelsels",
            MenuItem::Crew => "Crew",
        };
        write!(f, "{}", res)
    }
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
            self.selected = MenuItem::Crew
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
    last_key_pressed: Option<event::KeyEvent>,
    last_press_time: Instant,
    storage: Storage,
    effects: EffectManager<()>,
    pub user: User,

    menu: MenuState,
    starmap: StarMap,
    galaxy: GalacticMap,
    crew: CrewStatus,
}

impl App {
    pub fn new(storage: Storage, user: User)-> Self {
        // Init effect
        let mut effects: EffectManager<()> = EffectManager::default();
        effects.add_effect(
            fx::prolong_start(0, fx::coalesce(1000))
        );
       
        let pos = (user.pos_x, user.pos_y);
        Self {
            exit: false,
            last_key_pressed: None,
            last_press_time: Instant::now(),
            effects,
            storage,
            user,

            menu: MenuState {
                list_state: ListState::default().with_selected(Some(0)),
                selected: MenuItem::GalacticMap,
                active: MenuItem::GalacticMap,
            },
            starmap: StarMap::new(),
            galaxy: GalacticMap::new(pos),
            crew: CrewStatus{},
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let mut last_frame = Instant::now();

        while !self.exit {
            let elapsed = last_frame.elapsed();
            last_frame = Instant::now();

            terminal.draw(|frame| { self.render_frame(frame, elapsed); })?;
            self.handle_events()?;

        }
        self.user.pos_x = self.galaxy.current_pos.0;
        self.user.pos_y = self.galaxy.current_pos.1;

        let mut copy = self.storage.clone();
        copy.update_user(&self.user);
        let _ = copy.save();
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame, elapsed: Duration) {
        let area = frame.area();
        frame.render_widget(&mut *self, area);
        self.effects.process_effects(elapsed.into(), frame.buffer_mut(), area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Only set when the key is different
                    if self.last_key_pressed != Some(key) {
                        self.last_press_time = Instant::now();
                        self.last_key_pressed = Some(key);
                    }

                    self.handle_press_event(key);
                    match self.menu.active {
                        MenuItem::GalacticMap => { 
                            if let Some(diff) = self.galaxy.handle_press_event(key, self.last_key_pressed, self.last_press_time) {
                                self.user.crystals += diff.crystals;
                                self.user.fuel += diff.fuel;
                                self.storage.components += diff.components;
                            }
                        }
                        MenuItem::StarMap => { self.starmap.handle_press_event(key, self.last_key_pressed, self.last_press_time); },
                        _ => {}
                    }
                }
            }
        } else {
            // No key was pressed, reset 
            self.last_key_pressed = None;
        }
        Ok(())
    }

    fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Key's for all widgets
            KeyCode::Esc        => { self.exit = true; },
            // Other
            KeyCode::Up         => { self.menu.select(-1); },
            KeyCode::Down       => { self.menu.select(1); },
            KeyCode::Enter      => { 
                self.menu.activate();
                // TODO: apply the effect only to the submodule / widget in the screen
                // self.effects.add_effect(fx::coalesce(1000));
            },
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

        text.extend(Line::from(
            format!("Logged in as: {}", self.user.username.clone())
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
            Line::from(MenuItem::GalacticMap.to_string()).alignment(Alignment::Center),
            Line::from(MenuItem::StarMap.to_string()).alignment(Alignment::Center),
            Line::from(MenuItem::Crew.to_string()).alignment(Alignment::Center),
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
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ]).areas(area);

        let [title, list, status, resources] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(25),
                Constraint::Percentage(15),
            ])
            .areas(left);

        self.render_title(title, buf);
        self.render_list(list, buf);
        Resources {
            crystals: self.user.crystals,
            fuel: self.user.fuel,
            components: self.storage.components,
        }.render(resources, buf);

        let block = Block::bordered()
            .title(self.menu.active.to_string().bold())
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);
        let inner = block.inner(right);
        block.render(right, buf);

        match self.menu.active {
            MenuItem::GalacticMap => { self.galaxy.render(inner, buf); }
            MenuItem::StarMap   => { self.starmap.render(inner, buf); },
            MenuItem::Crew      => { self.crew.render(inner, buf); },
        }
    }
}


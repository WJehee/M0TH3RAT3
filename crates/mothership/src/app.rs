use std::{fmt, time::Duration};

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};
use tachyonfx::{fx, EffectManager};
use throbber_widgets_tui::ThrobberState;
use widgets::{Diagnostics, KeyHold, Labels, Level, Menu, Notifications, Program};

use crate::{
    components::{
        crew::CrewStatus, galaxy_map::GalacticMap, resources::Resources, star_map::StarMap,
        stock_market::StockMarket,
    },
    storage::Storage,
    user::User,
    util::{self, Event},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MenuItem {
    GalacticMap,
    StarMap,
    Crew,
    Diagnostics,
    StockMarket,
}

impl MenuItem {
    const ALL: [MenuItem; 5] = [
        MenuItem::GalacticMap,
        MenuItem::StarMap,
        MenuItem::Crew,
        MenuItem::Diagnostics,
        MenuItem::StockMarket,
    ];
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            MenuItem::GalacticMap => "Sterren kaart",
            MenuItem::StarMap => "Zonnestelsels",
            MenuItem::Crew => "Crew",
            MenuItem::Diagnostics => "Diagnostics",
            MenuItem::StockMarket => "Beurs",
        };
        write!(f, "{}", res)
    }
}

pub struct App {
    // UI
    exit: bool,
    effects: EffectManager<()>,
    throbber_state: ThrobberState,

    // Data
    storage: Storage,
    pub user: User,

    // Sub components
    menu: Menu<MenuItem>,
    starmap: Option<StarMap>,
    galaxy: GalacticMap,
    crew: CrewStatus,
    diagnostics: Diagnostics,
    stock_market: StockMarket,
    notifications: Notifications<MenuItem>,
}

impl App {
    pub fn new(storage: Storage, user: User) -> Self {
        // Init effect
        let mut effects: EffectManager<()> = EffectManager::default();
        effects.add_effect(fx::prolong_start(0, fx::coalesce(1000)));

        let pos = (user.pos_x, user.pos_y);
        let solar_systems = storage.map.clone();
        let mut result = Self {
            exit: false,
            effects,
            throbber_state: ThrobberState::default(),

            storage,
            user,

            menu: Menu::new(MenuItem::ALL.to_vec()),
            starmap: None,
            galaxy: GalacticMap::new(solar_systems.clone(), pos),
            crew: CrewStatus {},
            diagnostics: Diagnostics::new(),
            stock_market: StockMarket::new(),
            notifications: Notifications::new(Labels::DUTCH),
        };
        result.galaxy.update_system();
        if let Some(system) = result.galaxy.get_current_system() {
            result.starmap = Some(system.to_star_map());
        }
        result
    }

    /// Copies the player's position, resources and the explored map into
    /// `storage`. Called once when the session ends rather than on every
    /// change, so a crash mid-session loses that session's progress; that is
    /// acceptable for a game night. The map is copied whole, so with several
    /// players online the last one to leave overwrites planet state.
    /// TODO: merge per-planet changes instead of replacing the map, so
    /// concurrent SSH players do not undo each other's exploration.
    pub fn apply_to(&self, storage: &mut Storage) {
        let mut user = self.user.clone();
        user.pos_x = self.galaxy.current_pos.0;
        user.pos_y = self.galaxy.current_pos.1;
        storage.update_user(&user);
        storage.map = self.galaxy.solar_systems.clone();
        storage.components = self.storage.components;
    }

    fn apply(&mut self, event: Event) {
        match event {
            Event::Item(diff) => {
                let fuel_before = self.user.fuel;
                self.user.crystals += diff.crystals;
                self.user.fuel += diff.fuel;
                self.storage.components += diff.components;
                // Earn 1 reputation per component
                self.user.reputation += diff.components;
                if fuel_before > 0 && self.user.fuel <= 0 {
                    self.notifications.push(
                        Level::Critical,
                        "Brandstof is op!",
                        Some(MenuItem::GalacticMap),
                    );
                }
            }
            Event::NewSystem(Some(system)) => {
                self.starmap = Some(system.to_star_map());
            }
            Event::NewSystem(None) => {
                self.starmap = None;
            }
            Event::PlanetUpdate => {
                if let Some(system) = self.galaxy.get_current_system_mut() {
                    system.planets = self
                        .starmap
                        .as_ref()
                        .expect("starmap just handled input")
                        .planets
                        .clone();
                }
            }
            Event::RandomEvent => {
                self.notifications.push(
                    Level::Warning,
                    "Willekeurige gebeurtenis! Ga naar de leiding.",
                    Some(MenuItem::StarMap),
                );
            }
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
            " Sluit melding ".into(),
            "<n>".green().bold(),
            " Quit ".into(),
            "<Esc> ".green().bold(),
        ]);
        let block = Block::bordered()
            .title_bottom(instructions)
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);

        let mut text = Text::from(util::TITLE_HEADER).fg(Color::Green);

        text.extend(Line::from(format!(
            "Ingelogd als: {}",
            self.user.username.clone()
        )));

        Paragraph::new(text).centered().block(block).render(area, buf);
    }

    fn render_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let [_padding_top, menu_pos, _padding_bottom] = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .areas(area);

        // Each item is prefixed with the marker of its most severe pending
        // notification, so the player can see which screen needs attention
        // without reading the notification panel.
        let notifications = &self.notifications;
        let out_of_fuel = self.user.fuel <= 0;
        self.menu.render(menu_pos, buf, |item| {
            let mut spans: Vec<Span<'_>> = Vec::with_capacity(3);
            if let Some(level) = notifications.highest_level_for(item) {
                spans.push(level.marker_span());
                spans.push(" ".into());
            }
            let mut name = Span::from(item.to_string());
            if *item == MenuItem::GalacticMap && out_of_fuel {
                name = name.crossed_out();
            }
            spans.push(name);
            Line::from(spans).alignment(Alignment::Center)
        });
    }
}

impl Program for App {
    fn tick(&mut self) {
        self.throbber_state.calc_next();
        self.diagnostics.tick();
        if let Some(headline) = self.stock_market.tick() {
            self.notifications
                .push(Level::Info, headline, Some(MenuItem::StockMarket));
        }
    }

    fn handle_key(&mut self, key: KeyEvent, hold: &KeyHold) {
        match key.code {
            // Key's for all widgets
            KeyCode::Esc => self.exit = true,
            KeyCode::Up => self.menu.select(-1),
            KeyCode::Down => self.menu.select(1),
            KeyCode::Enter => {
                self.menu.activate();
                // TODO: apply the effect only to the submodule / widget in the screen
                // self.effects.add_effect(fx::coalesce(1000));
            }
            KeyCode::Char('n') => self.notifications.dismiss(),
            _ => {}
        }

        let events = match self.menu.active() {
            MenuItem::GalacticMap => {
                self.galaxy.handle_press_event(key, hold, self.user.fuel > 0)
            }
            MenuItem::StarMap => match &mut self.starmap {
                Some(map) => map.handle_press_event(key, hold, self.user.username.clone()),
                None => Vec::new(),
            },
            MenuItem::StockMarket => {
                self.stock_market.handle_press_event(key);
                Vec::new()
            }
            _ => Vec::new(),
        };
        for event in events {
            self.apply(event);
        }
    }

    fn render(&mut self, frame: &mut Frame, elapsed: Duration) {
        let area = frame.area();
        frame.render_widget(&mut *self, area);
        self.effects
            .process_effects(elapsed.into(), frame.buffer_mut(), area);
    }

    fn should_exit(&self) -> bool {
        self.exit
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).areas(area);

        let [title, list, status, resources] = Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .areas(left);

        self.render_title(title, buf);
        self.render_menu(list, buf);

        StatefulWidget::render(&self.notifications, status, buf, &mut self.throbber_state);

        // TODO: render current planet stats

        Resources {
            crystals: self.user.crystals,
            fuel: self.user.fuel,
            reputation: self.user.reputation,
            components: self.storage.components,
        }
        .render(resources, buf);

        // Main widget
        let block = Block::bordered()
            .title(self.menu.active().to_string().bold())
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);
        let inner = block.inner(right);
        block.render(right, buf);

        match self.menu.active() {
            MenuItem::GalacticMap => self.galaxy.render(inner, buf),
            MenuItem::StarMap => {
                if let Some(map) = &self.starmap {
                    map.render(inner, buf);
                }
            }
            MenuItem::Crew => self.crew.render(inner, buf),
            MenuItem::Diagnostics => self.diagnostics.render(inner, buf),
            MenuItem::StockMarket => self.stock_market.render(inner, buf),
        }
    }
}

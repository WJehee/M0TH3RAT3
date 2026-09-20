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

use crate::screens::{music::Music, navigation::Navigation};

const TITLE_HEADER: &str = r#"
  ____    _    ____  
 / ___|  / \  |  _ \ 
| |     / _ \ | |_) |
| |___ / ___ \|  _ < 
 \____/_/   \_\_| \_\

CAR-OS
"#;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Screen {
    Navigation,
    Music,
    Vehicle,
}

impl Screen {
    const ALL: [Screen; 3] = [Screen::Navigation, Screen::Music, Screen::Vehicle];
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            Screen::Navigation => "Navigation",
            Screen::Music => "Music",
            Screen::Vehicle => "Vehicle",
        };
        write!(f, "{res}")
    }
}

pub struct App {
    exit: bool,
    effects: EffectManager<()>,
    throbber_state: ThrobberState,

    menu: Menu<Screen>,
    notifications: Notifications<Screen>,
    navigation: Navigation,
    music: Music,
    // The Diagnostics widget stands in for real vehicle telemetry until the
    // display is connected to the car's CAN bus or OBD-II port.
    vehicle: Diagnostics,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut effects: EffectManager<()> = EffectManager::default();
        effects.add_effect(fx::prolong_start(0, fx::coalesce(1000)));

        let ticks_per_second = (1000 / crate::TICK_RATE.as_millis()) as u32;
        let mut notifications = Notifications::new(Labels::ENGLISH);
        notifications.push(Level::Info, "No GPS fix, showing demo route.", Some(Screen::Navigation));

        Self {
            exit: false,
            effects,
            throbber_state: ThrobberState::default(),
            menu: Menu::new(Screen::ALL.to_vec()),
            notifications,
            navigation: Navigation::default(),
            music: Music::new(ticks_per_second),
            vehicle: Diagnostics::new(),
        }
    }

    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Select ".into(),
            "<Enter>".green().bold(),
            " Menu ".into(),
            "<Up/Down>".green().bold(),
            " Dismiss ".into(),
            "<n>".green().bold(),
            " Quit ".into(),
            "<Esc> ".green().bold(),
        ]);
        let block = Block::bordered()
            .title_bottom(instructions)
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);

        let mut text = Text::from(TITLE_HEADER).fg(Color::Green);
        let track = self.music.now_playing();
        let state = if self.music.is_paused() { "paused" } else { "playing" };
        text.extend(Line::from(format!("{state}: {} - {}", track.artist, track.title)));

        Paragraph::new(text).centered().block(block).render(area, buf);
    }

    fn render_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let [_top, menu_area, _bottom] = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .areas(area);

        let notifications = &self.notifications;
        self.menu.render(menu_area, buf, |item| {
            let mut spans: Vec<Span<'_>> = Vec::with_capacity(3);
            if let Some(level) = notifications.highest_level_for(item) {
                spans.push(level.marker_span());
                spans.push(" ".into());
            }
            spans.push(Span::from(item.to_string()));
            Line::from(spans).alignment(Alignment::Center)
        });
    }
}

impl Program for App {
    fn tick(&mut self) {
        self.throbber_state.calc_next();
        self.vehicle.tick();
        self.navigation.tick();
        if let Some(track) = self.music.tick() {
            self.notifications.push(
                Level::Info,
                format!("Now playing: {} - {}", track.artist, track.title),
                Some(Screen::Music),
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent, _hold: &KeyHold) {
        match key.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Up => self.menu.select(-1),
            KeyCode::Down => self.menu.select(1),
            KeyCode::Char('n') => self.notifications.dismiss(),
            _ => {}
        }
        // Enter both activates the highlighted menu entry and confirms inside
        // the active screen; the screens only see Enter once they are active.
        if key.code == KeyCode::Enter && self.menu.selected() != self.menu.active() {
            self.menu.activate();
            return;
        }
        match self.menu.active() {
            Screen::Navigation => self.navigation.handle_key(key),
            Screen::Music => self.music.handle_key(key),
            Screen::Vehicle => {}
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
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).areas(area);
        let [title, menu, status] = Layout::vertical([
            Constraint::Length(11),
            Constraint::Min(0),
            Constraint::Length(6),
        ])
        .areas(left);

        self.render_title(title, buf);
        self.render_menu(menu, buf);
        StatefulWidget::render(&self.notifications, status, buf, &mut self.throbber_state);

        let block = Block::bordered()
            .title(self.menu.active().to_string().bold())
            .title_alignment(Alignment::Center)
            .border_set(border::THICK);
        let inner = block.inner(right);
        block.render(right, buf);

        match self.menu.active() {
            Screen::Navigation => self.navigation.render(inner, buf),
            Screen::Music => self.music.render(inner, buf),
            Screen::Vehicle => self.vehicle.render(inner, buf),
        }
    }
}

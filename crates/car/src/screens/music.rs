//! Music screen: a playlist with a playing track and a progress gauge.
//!
//! TODO: back this with mpd or an MPRIS client so the list and progress come
//! from the actual player instead of the built-in demo playlist.

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};
use widgets::LabeledGauge;

pub struct Track {
    pub title: &'static str,
    pub artist: &'static str,
    pub length_secs: u32,
}

const DEMO_PLAYLIST: [Track; 4] = [
    Track { title: "Ghost Ship", artist: "Astral Drift", length_secs: 214 },
    Track { title: "Cold Fusion", artist: "Neon Hull", length_secs: 187 },
    Track { title: "Airlock", artist: "Void Runner", length_secs: 256 },
    Track { title: "Last Signal", artist: "Deep Static", length_secs: 301 },
];

pub struct Music {
    list_state: ListState,
    selected: usize,
    playing: usize,
    paused: bool,
    /// Elapsed time of the playing track, in ticks of the app tick rate.
    elapsed_ticks: u32,
    ticks_per_second: u32,
}

impl Music {
    pub fn new(ticks_per_second: u32) -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            selected: 0,
            playing: 0,
            paused: true,
            elapsed_ticks: 0,
            ticks_per_second,
        }
    }

    pub fn now_playing(&self) -> &'static Track {
        &DEMO_PLAYLIST[self.playing]
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    fn elapsed_secs(&self) -> u32 {
        self.elapsed_ticks / self.ticks_per_second
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => self.move_selection(-1),
            KeyCode::Right => self.move_selection(1),
            KeyCode::Enter => {
                self.playing = self.selected;
                self.elapsed_ticks = 0;
                self.paused = false;
            }
            KeyCode::Char(' ') => self.paused = !self.paused,
            _ => {}
        }
    }

    fn move_selection(&mut self, offset: isize) {
        let len = DEMO_PLAYLIST.len() as isize;
        self.selected = (self.selected as isize + offset).rem_euclid(len) as usize;
        self.list_state.select(Some(self.selected));
    }

    /// Advances playback. Returns the track that just started when the
    /// previous one finished, so the app can announce it.
    pub fn tick(&mut self) -> Option<&'static Track> {
        if self.paused {
            return None;
        }
        self.elapsed_ticks += 1;
        if self.elapsed_secs() >= self.now_playing().length_secs {
            self.playing = (self.playing + 1) % DEMO_PLAYLIST.len();
            self.elapsed_ticks = 0;
            return Some(self.now_playing());
        }
        None
    }
}

impl Widget for &mut Music {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [list_area, progress] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);

        let items: Vec<ListItem> = DEMO_PLAYLIST
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let marker = if i == self.playing {
                    if self.paused { "‖ " } else { "▶ " }
                } else {
                    "  "
                };
                ListItem::new(Line::from(vec![
                    marker.green().bold(),
                    track.title.bold(),
                    "  ".into(),
                    track.artist.dark_gray(),
                ]))
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("PLAYLIST")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(Style::default().fg(Color::Green).bold())
            .highlight_symbol("> ");
        StatefulWidget::render(list, list_area, buf, &mut self.list_state);

        let track = self.now_playing();
        LabeledGauge::new(
            &format!("{} - {}", track.artist, track.title),
            self.elapsed_secs() as f64,
            track.length_secs as f64,
            Color::Magenta,
        )
        .render(progress, buf);
    }
}

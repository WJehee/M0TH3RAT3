//! The frame loop shared by every app: draw, poll input, tick on a fixed
//! interval, and restore the terminal on panic.
//!
//! The loop is transport-agnostic. Input arrives through an [`EventSource`]
//! and output goes to whatever backend the [`Terminal`] was built with, so
//! the same [`Program`] runs on the local terminal (see [`run_local`]) or on
//! a remote client whose bytes are shuttled over SSH.

use std::{
    io, panic,
    sync::mpsc,
    time::{Duration, Instant},
};

use color_eyre::{config::HookBuilder, eyre};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, KeyEvent, KeyEventKind},
    layout::Rect,
    Frame, Terminal,
};

use crate::tui;

/// How long the loop waits for input before treating the held key as
/// released. Key repeat in every terminal tested fires faster than this, so
/// a held key keeps arriving within the window while a released key does not.
pub const POLL_TIMEOUT: Duration = Duration::from_millis(50);

/// Input the frame loop understands, independent of where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(KeyEvent),
    /// New terminal size in `(columns, rows)`.
    Resize(u16, u16),
    /// The input side is gone (client disconnected, channel closed). The
    /// loop ends after this.
    Quit,
}

/// Where the frame loop gets its input from.
pub trait EventSource {
    /// Returns the next input, or `None` if nothing arrived within `timeout`.
    fn next(&mut self, timeout: Duration) -> io::Result<Option<Input>>;
}

/// Input from the terminal this process is attached to.
#[derive(Debug, Default)]
pub struct CrosstermEvents;

impl EventSource for CrosstermEvents {
    fn next(&mut self, timeout: Duration) -> io::Result<Option<Input>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }
        Ok(match event::read()? {
            event::Event::Key(key) if key.kind == KeyEventKind::Press => Some(Input::Key(key)),
            event::Event::Resize(cols, rows) => Some(Input::Resize(cols, rows)),
            _ => None,
        })
    }
}

/// Input delivered by another thread over a channel, for sessions whose
/// bytes arrive over the network. A closed channel reads as [`Input::Quit`].
#[derive(Debug)]
pub struct ChannelEvents {
    rx: mpsc::Receiver<Input>,
}

impl ChannelEvents {
    pub fn new() -> (mpsc::Sender<Input>, Self) {
        let (tx, rx) = mpsc::channel();
        (tx, Self { rx })
    }
}

impl EventSource for ChannelEvents {
    fn next(&mut self, timeout: Duration) -> io::Result<Option<Input>> {
        match self.rx.recv_timeout(timeout) {
            Ok(input) => Ok(Some(input)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(Some(Input::Quit)),
        }
    }
}

/// Tracks a key being held down. Terminals only report presses (via key
/// repeat), never releases, so a key counts as held while identical press
/// events keep arriving within `POLL_TIMEOUT`.
#[derive(Debug)]
pub struct KeyHold {
    last_key: Option<KeyEvent>,
    since: Instant,
}

impl Default for KeyHold {
    fn default() -> Self {
        Self {
            last_key: None,
            since: Instant::now(),
        }
    }
}

impl KeyHold {
    fn press(&mut self, key: KeyEvent) {
        if self.last_key != Some(key) {
            self.since = Instant::now();
            self.last_key = Some(key);
        }
    }

    fn release(&mut self) {
        self.last_key = None;
    }

    /// How long `key` has been held, or `None` if it is not the key being
    /// held right now.
    pub fn held_for(&self, key: KeyEvent) -> Option<Duration> {
        (self.last_key == Some(key)).then(|| self.since.elapsed())
    }

    /// Progress in [0, 1] of holding `key` towards `duration`, for warp bars
    /// and long-press confirmations.
    pub fn progress(&self, key: KeyEvent, duration: Duration) -> f64 {
        self.held_for(key)
            .map(|held| (held.as_secs_f64() / duration.as_secs_f64()).min(1.0))
            .unwrap_or(0.0)
    }

    pub fn completed(&self, key: KeyEvent, duration: Duration) -> bool {
        self.held_for(key).is_some_and(|held| held > duration)
    }
}

/// An application driven by [`run`].
pub trait Program {
    /// Called every `tick_rate`, independent of input and frame rate.
    fn tick(&mut self);

    /// Called for every key press. `hold` already includes this press.
    fn handle_key(&mut self, key: KeyEvent, hold: &KeyHold);

    /// Called when no key arrived within the poll window.
    fn key_released(&mut self) {}

    /// `elapsed` is the time since the previous frame, for effects.
    fn render(&mut self, frame: &mut Frame, elapsed: Duration);

    fn should_exit(&self) -> bool;
}

/// Drives `program` until it asks to exit or the event source quits.
pub fn run<B: Backend>(
    program: &mut impl Program,
    terminal: &mut Terminal<B>,
    events: &mut impl EventSource,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut hold = KeyHold::default();
    let mut last_frame = Instant::now();
    let mut last_tick = Instant::now();

    while !program.should_exit() {
        let elapsed = last_frame.elapsed();
        if last_tick.elapsed() >= tick_rate {
            program.tick();
            last_tick = Instant::now();
        }
        last_frame = Instant::now();

        terminal.draw(|frame| program.render(frame, elapsed))?;

        match events.next(POLL_TIMEOUT)? {
            Some(Input::Key(key)) => {
                hold.press(key);
                program.handle_key(key, &hold);
            }
            // A fullscreen viewport resizes itself on the next draw, but a
            // fixed one (used for remote clients) only learns the size from
            // us, so always forward it.
            Some(Input::Resize(cols, rows)) => terminal.resize(Rect::new(0, 0, cols, rows))?,
            Some(Input::Quit) => break,
            None => {
                hold.release();
                program.key_released();
            }
        }
    }
    Ok(())
}

/// [`run`] on the terminal this process is attached to.
pub fn run_local(program: &mut impl Program, terminal: &mut tui::Tui, tick_rate: Duration) -> io::Result<()> {
    run(program, terminal, &mut CrosstermEvents, tick_rate)
}

/// Installs panic and eyre hooks that leave the local terminal in a usable
/// state before printing the report. Harmless when no terminal is attached.
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = tui::restore();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            let _ = tui::restore();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

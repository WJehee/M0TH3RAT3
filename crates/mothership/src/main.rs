use std::{path::PathBuf, sync::Mutex, time::Duration};

use clap::Parser;
use color_eyre::Result;
use widgets::{install_hooks, run_local, tui};

use crate::{storage::Storage, user::User};

mod app;
mod components;
mod login;
mod objects;
mod storage;
mod user;
mod util;

/// The ship simulation advances five times per second, which keeps the
/// throbber and diagnostics lively without burning CPU on the Pi.
const TICK_RATE: Duration = Duration::from_millis(200);

/// M0TH3R@3: ship interface for Mothership players.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Save file. Falls back to default.json, then to an empty fallback.json.
    storage: Option<PathBuf>,

    /// Serve the interface over SSH instead of running it in this terminal.
    /// Players log in with the username and password from the save file.
    #[arg(long)]
    serve: bool,

    /// Address to listen on in serve mode.
    #[arg(long, default_value = "0.0.0.0:2222")]
    listen: String,

    /// SSH host key in serve mode. Generated on first start if missing.
    #[arg(long, default_value = "mothership_host_key")]
    host_key: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // The widgets hooks restore the local terminal before printing an error,
    // which would spray escape codes into the server log, so serve mode gets
    // plain color-eyre instead.
    if cli.serve {
        color_eyre::install()?;
    } else {
        install_hooks()?;
    }

    let storage = match &cli.storage {
        Some(path) => Storage::load(path.to_string_lossy().into_owned()).expect("storage path to be valid"),
        None => match Storage::load(String::from("default.json")) {
            Ok(storage) => storage,
            Err(err) => {
                println!("{err:?}");
                Storage::new(String::from("fallback.json"))
            }
        },
    };

    if cli.serve {
        serve_ssh(storage, &cli)
    } else {
        run_in_terminal(storage)
    }
}

fn run_in_terminal(mut storage: Storage) -> Result<()> {
    let mut terminal = tui::init()?;

    // let user = login::LoginScreen::new(storage.clone()).run(&mut terminal)?;
    let user = storage.users.first().cloned().unwrap_or_else(|| User {
        username: String::from("captain"),
        password_start: String::new(),
        password_attempts: 0,
        password_attempts_max: 0,
        password: String::new(),
        pos_x: 0.0,
        pos_y: 0.0,
        fuel: 2,
        crystals: 0,
        reputation: 0,
    });
    let mut app = app::App::new(storage.clone(), user);
    let run_result = run_local(&mut app, &mut terminal, TICK_RATE);
    app.apply_to(&mut storage);
    let _ = storage.save();

    tui::restore()?;
    run_result?;
    Ok(())
}

fn serve_ssh(storage: Storage, cli: &Cli) -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!(
        "Serving {} with {} users on {}",
        storage.path,
        storage.users.len(),
        cli.listen
    );
    let service = std::sync::Arc::new(Ship {
        storage: Mutex::new(storage),
    });
    serve::run_server_blocking(
        service,
        cli.listen.clone(),
        serve::ServerConfig {
            host_key: cli.host_key.clone(),
            ..Default::default()
        },
    )
}

/// The shared game state behind the SSH server. Every session takes a copy
/// of the storage to play on and writes its changes back when it ends.
struct Ship {
    storage: Mutex<Storage>,
}

impl Ship {
    fn lock(&self) -> std::sync::MutexGuard<'_, Storage> {
        // A panic while holding the lock leaves plain data behind, nothing
        // half-updated, so keep serving instead of poisoning every session.
        self.storage.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl serve::Service for Ship {
    fn authenticate(&self, username: &str, password: &str) -> bool {
        let mut storage = self.lock();
        let accepted = storage.try_login(username, password).is_some();
        // Attempt counts are part of the login puzzle, so persist them even
        // for rejected logins.
        if let Err(err) = storage.save() {
            log::error!("Saving after login attempt failed: {err}");
        }
        accepted
    }

    fn run_session(&self, mut session: serve::Session) {
        let (storage, user) = {
            let storage = self.lock();
            let Some(user) = storage.find_user(&session.username).cloned() else {
                log::warn!("User {:?} vanished after login", session.username);
                return;
            };
            (storage.clone(), user)
        };

        let mut app = app::App::new(storage, user);
        if let Err(err) = widgets::run(&mut app, &mut session.terminal, &mut session.events, TICK_RATE) {
            log::warn!("Session for {:?} ended with error: {err}", session.username);
        }

        let mut storage = self.lock();
        app.apply_to(&mut storage);
        if let Err(err) = storage.save() {
            log::error!("Saving after session failed: {err}");
        }
    }
}

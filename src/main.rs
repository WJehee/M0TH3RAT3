use std::panic;
use std::sync::{Mutex, LazyLock};
use std::path::Path;

use color_eyre::{
    config::HookBuilder,
    eyre,
    Result,
};
use rusqlite::{Connection};
    
mod tui;
mod app;
mod components;

static DB: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
    let conn = Connection::open("sqlite.db").unwrap();
    Mutex::new(conn)
});

fn main() -> Result<()> {
    let conn = DB.lock().unwrap();
    let path = Path::new(conn.path().unwrap());

    // Init database if not already done
    if !path.exists() {
        conn.execute("
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
            )", ()).unwrap();
    }

    install_hooks()?;
    let mut terminal = tui::init()?;
    let _app_result = app::App::new().run(&mut terminal);
    tui::restore()?;
    Ok(())
}

pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            tui::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}


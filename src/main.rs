use std::{panic};

use color_eyre::{
    Result,
    config::HookBuilder,
    eyre,
};

mod tui;
mod app;
mod components;
mod storage;

fn main() -> Result<()> {
    let path = std::env::args().nth(1);

    install_hooks()?;
    let mut terminal = tui::init()?;
    let _app_result = app::App::new(path).run(&mut terminal);
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


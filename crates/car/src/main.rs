use std::time::Duration;

use color_eyre::Result;
use widgets::{install_hooks, run_local, tui};

mod app;
mod screens;

/// Track progress and vehicle readouts only need to move a few times per
/// second; the frame loop itself still redraws as fast as input arrives.
const TICK_RATE: Duration = Duration::from_millis(250);

fn main() -> Result<()> {
    install_hooks()?;
    let mut terminal = tui::init()?;

    let mut app = app::App::new();
    let run_result = run_local(&mut app, &mut terminal, TICK_RATE);

    tui::restore()?;
    run_result?;
    Ok(())
}

//! Widgets and runtime plumbing shared by every display built in this
//! workspace. Nothing in here knows about spaceships or cars: anything
//! domain specific lives in the application crates.

pub mod diagnostics;
pub mod gauge;
pub mod menu;
pub mod notifications;
pub mod runtime;
pub mod tui;
pub mod util;

pub use diagnostics::Diagnostics;
pub use gauge::LabeledGauge;
pub use menu::Menu;
pub use notifications::{Labels, Level, Notifications};
pub use runtime::{install_hooks, run, run_local, ChannelEvents, CrosstermEvents, EventSource, Input, KeyHold, Program};
pub use tui::Tui;

//! Serves a [`widgets::Program`] to SSH clients.
//!
//! Each accepted shell gets its own OS thread running the ordinary
//! synchronous frame loop from `widgets::runtime`. Keystrokes decoded from the
//! SSH channel are pushed into a [`widgets::ChannelEvents`] queue, and the
//! ratatui backend writes into a [`ChannelWriter`] whose bytes a tokio task
//! forwards back over the channel. The application never touches tokio.
//!
//! The structure follows frittura-tui-serve by ricott1
//! (https://github.com/ricott1/frittura-ssh), which is GPL-3.0 and therefore
//! not used as a dependency. This is a smaller reimplementation on russh:
//! password auth only, no mouse support, no WebSocket transport, and the
//! session runs on a blocking thread instead of a tokio task.

mod handler;
mod input;
mod keys;
mod writer;

use std::{path::Path, sync::Arc, time::Duration};

use tokio::net::ToSocketAddrs;

use color_eyre::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use widgets::ChannelEvents;

pub use writer::ChannelWriter;

/// A ratatui terminal whose output goes to an SSH client.
pub type SshTerminal = Terminal<CrosstermBackend<ChannelWriter>>;

/// Everything a [`Service`] gets for one interactive shell.
pub struct Session {
    /// The username the client authenticated with.
    pub username: String,
    /// `TERM` advertised by the client, for example `xterm-256color`.
    pub term: String,
    /// Already switched to the alternate screen and sized to the client's
    /// window. Resizes arrive through `events` and are applied by
    /// `widgets::run`.
    pub terminal: SshTerminal,
    pub events: ChannelEvents,
}

/// What an application must provide to be served over SSH. One instance is
/// shared by every connection, so anything mutable needs its own locking.
pub trait Service: Send + Sync + 'static {
    /// Decides whether `username` may log in with `password`. Called from
    /// the async accept path, so it must not block for long.
    fn authenticate(&self, username: &str, password: &str) -> bool;

    /// Runs a whole session and returns when it is over. This is called on a
    /// dedicated thread per session, so it may block, and the program it
    /// builds never has to cross a thread boundary.
    fn run_session(&self, session: Session);
}

/// Tuning for the listening server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path of the Ed25519 host key. Generated on first start if missing.
    pub host_key: std::path::PathBuf,
    /// Connections idle this long are dropped by russh itself.
    pub inactivity_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host_key: "host_key".into(),
            inactivity_timeout: Duration::from_secs(3600),
        }
    }
}

/// Listens on `addr` and serves `service` until the process is killed.
pub async fn run_server<S: Service>(
    service: Arc<S>,
    addr: impl ToSocketAddrs + Send,
    config: ServerConfig,
) -> Result<()> {
    use russh::server::Server;

    let host_key = keys::load_or_generate(Path::new(&config.host_key))?;
    let russh_config = russh::server::Config {
        inactivity_timeout: Some(config.inactivity_timeout),
        // Failed logins are delayed so password guessing stays slow, but the
        // first failure answers immediately so a mistyped password does not
        // feel like a hung connection.
        auth_rejection_time: Duration::from_secs(3),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        // Clients behind NAT vanish silently. Keepalives detect that well
        // within the inactivity timeout so threads and channels are freed.
        keepalive_interval: Some(Duration::from_secs(30)),
        keepalive_max: 3,
        keys: vec![host_key],
        // Only password auth is offered. Clients try keyboard-interactive
        // before password, and every rejected method costs them the
        // rejection delay, so advertising unused methods makes logging in
        // feel slow.
        methods: russh::MethodSet::from(&[russh::MethodKind::Password][..]),
        nodelay: true,
        ..Default::default()
    };

    let mut server = handler::SshServer::new(service);
    server
        .run_on_address(Arc::new(russh_config), addr)
        .await?;
    Ok(())
}

/// [`run_server`] for callers without a tokio runtime of their own.
pub fn run_server_blocking<S: Service>(
    service: Arc<S>,
    addr: impl ToSocketAddrs + Send,
    config: ServerConfig,
) -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_server(service, addr, config))
}

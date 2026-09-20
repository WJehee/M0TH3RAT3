use std::{collections::HashMap, io::Write, sync::Arc};

use color_eyre::{eyre::eyre, Report};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{cursor, execute, terminal},
    layout::Rect,
    Terminal, TerminalOptions, Viewport,
};
use russh::{
    server::{Auth, ChannelOpenHandle, Handle, Handler, Msg, Server, Session as SshSession},
    Channel, ChannelId, Pty,
};
use tokio::sync::mpsc;
use widgets::{ChannelEvents, Input};

use crate::{input, writer::ChannelWriter, Service, Session};

pub(crate) struct SshServer<S> {
    service: Arc<S>,
}

impl<S: Service> SshServer<S> {
    pub(crate) fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

impl<S: Service> Server for SshServer<S> {
    type Handler = Connection<S>;

    fn new_client(&mut self, peer: Option<std::net::SocketAddr>) -> Self::Handler {
        log::info!("Connection from {peer:?}");
        Connection {
            service: self.service.clone(),
            username: None,
            channels: HashMap::new(),
        }
    }

    fn handle_session_error(&mut self, error: Report) {
        log::warn!("Session error: {error}");
    }
}

/// One SSH connection. A connection can open several channels, each of
/// which becomes its own session once the client asks for a shell.
pub(crate) struct Connection<S> {
    service: Arc<S>,
    /// Set once authentication succeeds; russh opens no channels before that.
    username: Option<String>,
    channels: HashMap<ChannelId, ChannelState>,
}

enum ChannelState {
    AwaitingPty,
    /// The client asked for a pty but not yet for a shell.
    PtyReady { term: String, cols: u16, rows: u16 },
    /// The session thread is running and consumes this queue.
    Running { input_tx: std::sync::mpsc::Sender<Input> },
}

impl<S: Service> Connection<S> {
    fn channel(&mut self, id: ChannelId) -> Result<&mut ChannelState, Report> {
        self.channels
            .get_mut(&id)
            .ok_or_else(|| eyre!("unknown channel {id}"))
    }

    fn start_session(&mut self, id: ChannelId, handle: Handle) -> Result<(), Report> {
        let state = std::mem::replace(self.channel(id)?, ChannelState::AwaitingPty);
        let ChannelState::PtyReady { term, cols, rows } = state else {
            self.channels.insert(id, state);
            return Err(eyre!("shell requested before pty on channel {id}"));
        };
        let username = self
            .username
            .clone()
            .ok_or_else(|| eyre!("shell requested before authentication"))?;

        // Output path: the session thread writes frames into the channel,
        // this task pushes them over SSH. When the writer is dropped at the
        // end of the session the loop ends and the channel is closed, which
        // is how the client learns the session is over.
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let forwarder_handle = handle.clone();
        tokio::spawn(async move {
            while let Some(frame) = out_rx.recv().await {
                if forwarder_handle.data(id, frame).await.is_err() {
                    break;
                }
            }
            // A clean exit status lets the client's ssh exit 0 instead of
            // reporting the connection as dropped.
            let _ = forwarder_handle.exit_status_request(id, 0).await;
            let _ = forwarder_handle.eof(id).await;
            let _ = forwarder_handle.close(id).await;
        });

        let (input_tx, events) = ChannelEvents::new();
        let backend = CrosstermBackend::new(ChannelWriter::new(out_tx));
        // A client without a real terminal (scripted ssh, a pty that was
        // never sized) reports 0x0, which would render nothing at all.
        let (cols, rows) = if cols == 0 || rows == 0 { (80, 24) } else { (cols, rows) };
        let options = TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, cols, rows)),
        };
        let mut terminal = Terminal::with_options(backend, options)?;

        let service = self.service.clone();
        tokio::task::spawn_blocking(move || {
            // The client's ssh already put its terminal in raw mode for the
            // pty, so only the screen state is ours to manage.
            let _ = execute!(
                terminal.backend_mut(),
                terminal::EnterAlternateScreen,
                cursor::Hide
            );
            let _ = terminal.clear();

            log::info!("Session started for {username:?} ({term})");
            service.run_session(Session {
                username: username.clone(),
                term,
                terminal,
                events,
            });
            log::info!("Session ended for {username:?}");
        });

        self.channels
            .insert(id, ChannelState::Running { input_tx });
        Ok(())
    }
}

/// Leaves the alternate screen and shows the cursor when the session ends,
/// including when the service panics, otherwise the client's shell stays
/// garbled after disconnect.
impl Drop for Session {
    fn drop(&mut self) {
        let backend = self.terminal.backend_mut();
        let _ = execute!(
            backend,
            terminal::LeaveAlternateScreen,
            cursor::Show,
            terminal::Clear(terminal::ClearType::All)
        );
        let _ = backend.flush();
    }
}

impl<S: Service> Handler for Connection<S> {
    type Error = Report;

    async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Report> {
        if self.service.authenticate(user, password) {
            self.username = Some(user.to_string());
            Ok(Auth::Accept)
        } else {
            // Debug formatting escapes control characters so a hostile
            // username cannot forge log lines.
            log::info!("Rejected login for {user:?}");
            Ok(Auth::reject())
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        _session: &mut SshSession,
    ) -> Result<(), Report> {
        self.channels
            .insert(channel.id(), ChannelState::AwaitingPty);
        reply.accept().await;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        id: ChannelId,
        term: &str,
        cols: u32,
        rows: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(Pty, u32)],
        session: &mut SshSession,
    ) -> Result<(), Report> {
        *self.channel(id)? = ChannelState::PtyReady {
            term: term.to_string(),
            cols: cols.min(u16::MAX as u32) as u16,
            rows: rows.min(u16::MAX as u32) as u16,
        };
        session.channel_success(id)?;
        Ok(())
    }

    async fn shell_request(&mut self, id: ChannelId, session: &mut SshSession) -> Result<(), Report> {
        self.start_session(id, session.handle())?;
        session.channel_success(id)?;
        Ok(())
    }

    async fn exec_request(
        &mut self,
        id: ChannelId,
        _command: &[u8],
        session: &mut SshSession,
    ) -> Result<(), Report> {
        // `ssh host somecommand` is treated the same as a plain shell; the
        // app has no commands to run.
        self.start_session(id, session.handle())?;
        session.channel_success(id)?;
        Ok(())
    }

    async fn data(&mut self, id: ChannelId, data: &[u8], session: &mut SshSession) -> Result<(), Report> {
        let ChannelState::Running { input_tx } = self.channel(id)? else {
            return Ok(());
        };
        for input in input::decode(data) {
            if input_tx.send(input).is_err() {
                // The session thread has finished; the forwarder closes the
                // channel, so just stop feeding it.
                self.channels.remove(&id);
                let _ = session.close(id);
                break;
            }
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        id: ChannelId,
        cols: u32,
        rows: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut SshSession,
    ) -> Result<(), Report> {
        let cols = cols.min(u16::MAX as u32) as u16;
        let rows = rows.min(u16::MAX as u32) as u16;
        match self.channel(id)? {
            ChannelState::Running { input_tx } => {
                let _ = input_tx.send(Input::Resize(cols, rows));
            }
            ChannelState::PtyReady { cols: c, rows: r, .. } => {
                *c = cols;
                *r = rows;
            }
            ChannelState::AwaitingPty => {}
        }
        Ok(())
    }

    async fn channel_eof(&mut self, id: ChannelId, _session: &mut SshSession) -> Result<(), Report> {
        // Dropping the sender makes the session's event source report Quit.
        self.channels.remove(&id);
        Ok(())
    }

    async fn channel_close(&mut self, id: ChannelId, _session: &mut SshSession) -> Result<(), Report> {
        self.channels.remove(&id);
        Ok(())
    }
}

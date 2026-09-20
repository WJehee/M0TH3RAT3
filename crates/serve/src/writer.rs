use std::io;

use tokio::sync::mpsc::UnboundedSender;

/// `io::Write` target for a ratatui `CrosstermBackend` whose bytes belong to
/// an SSH client. Writes accumulate in a buffer; ratatui calls `flush` once
/// per frame, which hands the whole frame to the forwarding task in one
/// message so a frame is never split across SSH packets mid-escape-sequence.
#[derive(Debug)]
pub struct ChannelWriter {
    tx: UnboundedSender<Vec<u8>>,
    sink: Vec<u8>,
}

impl ChannelWriter {
    pub(crate) fn new(tx: UnboundedSender<Vec<u8>>) -> Self {
        Self {
            tx,
            sink: Vec::new(),
        }
    }
}

impl io::Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.sink.is_empty() {
            return Ok(());
        }
        let frame = std::mem::take(&mut self.sink);
        self.tx
            .send(frame)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "ssh channel closed"))
    }
}

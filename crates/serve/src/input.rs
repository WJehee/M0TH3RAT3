//! Turns raw bytes from the SSH channel into key events.
//!
//! The client's terminal is in raw mode, so keys arrive as the bytes a VT
//! terminal would send. Only the sequences the apps use are recognised; a
//! held key or fast typing can batch several keys into one packet, so the
//! decoder loops until the buffer is empty rather than matching it whole.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use widgets::Input;

/// Escape sequences with their key, longest first so a prefix never wins
/// over a longer match.
const SEQUENCES: &[(&[u8], KeyCode)] = &[
    (b"\x1b[3~", KeyCode::Delete),
    (b"\x1b[A", KeyCode::Up),
    (b"\x1b[B", KeyCode::Down),
    (b"\x1b[C", KeyCode::Right),
    (b"\x1b[D", KeyCode::Left),
    (b"\x1bOA", KeyCode::Up),
    (b"\x1bOB", KeyCode::Down),
    (b"\x1bOC", KeyCode::Right),
    (b"\x1bOD", KeyCode::Left),
];

pub fn decode(mut data: &[u8]) -> Vec<Input> {
    let mut out = Vec::new();
    while !data.is_empty() {
        if let Some((seq, code)) = SEQUENCES.iter().find(|(seq, _)| data.starts_with(seq)) {
            out.push(key(*code));
            data = &data[seq.len()..];
            continue;
        }
        let (code, len) = match data[0] {
            // Ctrl-C and Ctrl-D are the only ways out of a hung remote
            // session, so both act like Esc, which every app treats as quit.
            0x03 | 0x04 | 0x1b => (KeyCode::Esc, 1),
            b'\r' | b'\n' => (KeyCode::Enter, 1),
            0x7f | 0x08 => (KeyCode::Backspace, 1),
            b'\t' => (KeyCode::Tab, 1),
            _ => match std::str::from_utf8(data).ok().and_then(|s| s.chars().next()) {
                Some(c) => (KeyCode::Char(c), c.len_utf8()),
                // Not valid UTF-8 at this position: skip the byte.
                None => {
                    data = &data[1..];
                    continue;
                }
            },
        };
        out.push(key(code));
        data = &data[len..];
    }
    out
}

fn key(code: KeyCode) -> Input {
    Input::Key(KeyEvent::new(code, KeyModifiers::empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(data: &[u8]) -> Vec<KeyCode> {
        decode(data)
            .into_iter()
            .map(|i| match i {
                Input::Key(k) => k.code,
                other => panic!("unexpected {other:?}"),
            })
            .collect()
    }

    #[test]
    fn decodes_batched_keys() {
        assert_eq!(
            codes(b"\x1b[Aw\r\r"),
            vec![KeyCode::Up, KeyCode::Char('w'), KeyCode::Enter, KeyCode::Enter]
        );
    }

    #[test]
    fn lone_escape_is_esc() {
        assert_eq!(codes(b"\x1b"), vec![KeyCode::Esc]);
    }

    #[test]
    fn multibyte_chars() {
        assert_eq!(codes("é".as_bytes()), vec![KeyCode::Char('é')]);
    }
}

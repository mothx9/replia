use crate::{EditError, core::valid_text};

#[derive(Debug, PartialEq)]
pub(crate) enum Key {
    Text(String),
    Enter,
    Interrupt,
    Eof,
    Home,
    End,
    Clear,
    Backspace,
    Delete,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Rejected(EditError),
    IncompletePaste,
}

#[derive(Default)]
enum State {
    #[default]
    Ready,
    Utf8(Vec<u8>),
    Escape(Vec<u8>),
    Discard {
        osc: bool,
        esc: bool,
    },
    Paste {
        bytes: Vec<u8>,
        matched: usize,
        overflow: bool,
    },
}

pub(crate) struct Decoder {
    state: State,
    limit: usize,
}
impl Decoder {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            state: State::Ready,
            limit,
        }
    }
    pub(crate) fn pending(&self) -> bool {
        !matches!(self.state, State::Ready)
    }
    pub(crate) fn expire(&mut self) -> Option<Key> {
        match std::mem::take(&mut self.state) {
            State::Ready => None,
            State::Paste { .. } => Some(Key::IncompletePaste),
            State::Utf8(_) => Some(Key::Rejected(EditError::InvalidUtf8)),
            _ => Some(Key::Rejected(EditError::InvalidSequence)),
        }
    }
    pub(crate) fn feed(&mut self, byte: u8) -> Option<Key> {
        match std::mem::take(&mut self.state) {
            State::Ready => self.ordinary(byte),
            State::Utf8(mut bytes) => {
                bytes.push(byte);
                match std::str::from_utf8(&bytes) {
                    Ok(s) if valid_text(s) => Some(Key::Text(s.to_owned())),
                    Ok(_) => Some(Key::Rejected(EditError::InvalidText)),
                    Err(e) if e.error_len().is_none() && bytes.len() < 4 => {
                        self.state = State::Utf8(bytes);
                        None
                    }
                    Err(_) => Some(Key::Rejected(EditError::InvalidUtf8)),
                }
            }
            State::Escape(mut bytes) => {
                bytes.push(byte);
                let osc = bytes.get(1) == Some(&b']');
                let finished = if osc {
                    byte == 7 || bytes.ends_with(b"\x1b\\")
                } else if bytes.get(1) == Some(&b'[') {
                    bytes.len() >= 3 && (0x40..=0x7e).contains(&byte)
                } else {
                    bytes.get(1) != Some(&b'O') || bytes.len() >= 3
                };
                if finished {
                    if bytes == b"\x1b[200~" {
                        self.state = State::Paste {
                            bytes: Vec::new(),
                            matched: 0,
                            overflow: false,
                        };
                        None
                    } else {
                        Some(sequence(&bytes))
                    }
                } else if bytes.len() >= 64 {
                    self.state = State::Discard {
                        osc,
                        esc: byte == 27,
                    };
                    Some(Key::Rejected(EditError::InvalidSequence))
                } else {
                    self.state = State::Escape(bytes);
                    None
                }
            }
            State::Discard { osc, esc } => {
                if !(if osc {
                    byte == 7 || (esc && byte == b'\\')
                } else {
                    (0x40..=0x7e).contains(&byte)
                }) {
                    self.state = State::Discard {
                        osc,
                        esc: byte == 27,
                    };
                }
                None
            }
            State::Paste {
                mut bytes,
                mut matched,
                mut overflow,
            } => {
                const END: &[u8] = b"\x1b[201~";
                if byte == END[matched] {
                    matched += 1;
                    if matched == END.len() {
                        if overflow {
                            return Some(Key::Rejected(EditError::Capacity));
                        }
                        return Some(match String::from_utf8(bytes) {
                            Ok(s) => {
                                let s = s.replace("\r\n", "\n").replace('\r', "\n");
                                if valid_text(&s) {
                                    Key::Text(s)
                                } else {
                                    Key::Rejected(EditError::InvalidText)
                                }
                            }
                            Err(_) => Key::Rejected(EditError::InvalidUtf8),
                        });
                    }
                } else {
                    for &b in &END[..matched] {
                        append(&mut bytes, b, self.limit, &mut overflow);
                    }
                    matched = 0;
                    if byte == END[0] {
                        matched = 1;
                    } else {
                        append(&mut bytes, byte, self.limit, &mut overflow);
                    }
                }
                self.state = State::Paste {
                    bytes,
                    matched,
                    overflow,
                };
                None
            }
        }
    }
    fn ordinary(&mut self, byte: u8) -> Option<Key> {
        Some(match byte {
            27 => {
                self.state = State::Escape(vec![27]);
                return None;
            }
            1 => Key::Home,
            3 => Key::Interrupt,
            4 => Key::Eof,
            5 => Key::End,
            8 | 127 => Key::Backspace,
            9 => Key::Tab,
            10 | 13 => Key::Enter,
            12 => Key::Clear,
            32..=126 => Key::Text(char::from(byte).to_string()),
            0xc2..=0xf4 => {
                self.state = State::Utf8(vec![byte]);
                return None;
            }
            0..=31 => Key::Rejected(EditError::InvalidText),
            _ => Key::Rejected(EditError::InvalidUtf8),
        })
    }
}

fn append(bytes: &mut Vec<u8>, byte: u8, limit: usize, overflow: &mut bool) {
    if bytes.len() < limit && !*overflow {
        bytes.push(byte);
    } else {
        *overflow = true;
    }
}
fn sequence(bytes: &[u8]) -> Key {
    match bytes {
        b"\x1b[A" => Key::Up,
        b"\x1b[B" => Key::Down,
        b"\x1b[C" => Key::Right,
        b"\x1b[D" => Key::Left,
        b"\x1b[H" | b"\x1bOH" | b"\x1b[1~" | b"\x1b[7~" => Key::Home,
        b"\x1b[F" | b"\x1bOF" | b"\x1b[4~" | b"\x1b[8~" => Key::End,
        b"\x1b[3~" => Key::Delete,
        _ => Key::Rejected(EditError::InvalidSequence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decode(bytes: &[u8], limit: usize) -> Vec<Key> {
        let mut d = Decoder::new(limit);
        bytes.iter().filter_map(|&b| d.feed(b)).collect()
    }
    #[test]
    fn keys_and_every_escape_alias_are_incremental() {
        for (bytes, expected) in [
            (b"\x1b[A".as_slice(), Key::Up),
            (b"\x1b[B", Key::Down),
            (b"\x1b[C", Key::Right),
            (b"\x1b[D", Key::Left),
            (b"\x1b[H", Key::Home),
            (b"\x1bOH", Key::Home),
            (b"\x1b[1~", Key::Home),
            (b"\x1b[7~", Key::Home),
            (b"\x1b[F", Key::End),
            (b"\x1bOF", Key::End),
            (b"\x1b[4~", Key::End),
            (b"\x1b[8~", Key::End),
            (b"\x1b[3~", Key::Delete),
        ] {
            assert_eq!(decode(bytes, 100), [expected]);
        }
        assert_eq!(
            decode(b"\x01\x03\x04\x05\x0c\x08\x7f\t\r\n", 100),
            [
                Key::Home,
                Key::Interrupt,
                Key::Eof,
                Key::End,
                Key::Clear,
                Key::Backspace,
                Key::Backspace,
                Key::Tab,
                Key::Enter,
                Key::Enter
            ]
        );
    }
    #[test]
    fn paste_is_atomic_normalized_and_not_shortcuts() {
        assert_eq!(
            decode("\x1b[200~é\r\n界\r🌍\n\x1b[201~\r".as_bytes(), 100),
            [Key::Text("é\n界\n🌍\n".into()), Key::Enter]
        );
        for control in [3, 4, 8, 12, 27, 127, 0] {
            let mut input = b"\x1b[200~safe".to_vec();
            input.push(control);
            input.extend_from_slice(b"\x1b[201~");
            assert_eq!(decode(&input, 100), [Key::Rejected(EditError::InvalidText)]);
        }
    }
    #[test]
    fn oversized_paste_drains_through_the_end_marker() {
        assert_eq!(
            decode(b"\x1b[200~abcdef\r\x03\x1b[201~x", 3),
            [Key::Rejected(EditError::Capacity), Key::Text("x".into())]
        );
    }
    #[test]
    fn invalid_utf8_and_partial_sequences_are_observable() {
        assert_eq!(
            decode(&[0xff, 0xc0, 0xed, 0xa0, 0x80], 10),
            [
                Key::Rejected(EditError::InvalidUtf8),
                Key::Rejected(EditError::InvalidUtf8),
                Key::Rejected(EditError::InvalidUtf8),
                Key::Rejected(EditError::InvalidUtf8)
            ]
        );
        for prefix in [b"\x1b".as_slice(), b"\x1b[20", b"\xf0\x9f"] {
            let mut d = Decoder::new(8);
            for b in prefix {
                assert!(d.feed(*b).is_none());
            }
            assert!(matches!(d.expire(), Some(Key::Rejected(_))));
            assert_eq!(d.feed(b'a'), Some(Key::Text("a".into())));
        }
        let mut d = Decoder::new(8);
        for b in b"\x1b[200~a\x1b[20" {
            d.feed(*b);
        }
        assert_eq!(d.expire(), Some(Key::IncompletePaste));
    }
    #[test]
    fn unknown_sequences_make_bounded_progress() {
        assert_eq!(
            decode(b"\x1b[999~a", 5),
            [
                Key::Rejected(EditError::InvalidSequence),
                Key::Text("a".into())
            ]
        );
        let mut d = Decoder::new(5);
        for b in b"\x1b[" {
            d.feed(*b);
        }
        for _ in 0..10000 {
            d.feed(b'1');
        }
        assert!(d.feed(b'~').is_none());
        assert_eq!(d.feed(b'a'), Some(Key::Text("a".into())));
    }
}

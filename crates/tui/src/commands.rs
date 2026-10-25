#![allow(dead_code)]

use tv::command::Command;
use tv::event::{Key, KeyEvent, KeyModifiers};

pub const CMD_ABOUT: Command = Command::custom("about");
pub const CMD_LOGS: Command = Command::custom("logs");

pub fn ctrl_key_event(key: Key) -> KeyEvent {
    KeyEvent::new(
        key,
        KeyModifiers {
            ctrl: true,
            ..Default::default()
        },
    )
}

pub fn alt_key_event(key: Key) -> KeyEvent {
    KeyEvent::new(
        key,
        KeyModifiers {
            alt: true,
            ..Default::default()
        },
    )
}

pub fn ctrl_char_event(key: char) -> KeyEvent {
    KeyEvent::new(
        Key::Char(key),
        KeyModifiers {
            ctrl: true,
            ..Default::default()
        },
    )
}

pub fn alt_char_event(key: char) -> KeyEvent {
    KeyEvent::new(
        Key::Char(key),
        KeyModifiers {
            alt: true,
            ..Default::default()
        },
    )
}

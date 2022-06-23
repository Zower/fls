use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Action;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
}

impl Mode {
    pub fn parse_key(&self, key: KeyEvent) -> Option<Action> {
        match self {
            Mode::Normal => Mode::parse_normal(key),
            Mode::Search => Mode::parse_search(key),
        }
    }

    pub fn parse_normal(key: KeyEvent) -> Option<Action> {
        let KeyEvent { code, modifiers: _ } = key;

        match code {
            KeyCode::Char('n') => Some(Action::Back),
            KeyCode::Char('o') => Some(Action::Open),
            KeyCode::Char('d') => Some(Action::Delete),
            KeyCode::Char('t') => Some(Action::ToggleCurrent),
            KeyCode::Char('s') | KeyCode::Char('/') => Some(Action::SearchMode),
            KeyCode::Char('i') => Some(Action::Up),
            KeyCode::Char('e') => Some(Action::Down),
            KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        }
    }

    fn parse_search(key: KeyEvent) -> Option<Action> {
        let KeyEvent { code, modifiers } = key;

        if modifiers.contains(KeyModifiers::SHIFT) {
            match code {
                KeyCode::Char(c) => {
                    return Mode::parse_normal(KeyEvent::new(
                        KeyCode::Char(c.to_ascii_lowercase()),
                        KeyModifiers::NONE,
                    ));
                }
                _ => (),
            }
        }

        match code {
            KeyCode::Enter => Some(Action::FreezeSearch),
            KeyCode::Backspace => Some(Action::PopFromSearch),
            KeyCode::Char(c) => Some(Action::AddToSearch(c)),
            _ => None,
        }
    }
}

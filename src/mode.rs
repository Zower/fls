use iced_native::keyboard::{Event, KeyCode};

use crate::app::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search(SearchMode),
}

impl Mode {
    pub fn parse_event(self, key: Event) -> Action {
        match self {
            Mode::Normal => Mode::parse_normal(key),
            Mode::Search(_) => Mode::parse_search(key),
        }
    }

    pub fn parse_normal(key: Event) -> Action {
        // let Event { code } = key;

        match key {
            Event::CharacterReceived('n') => Action::UpDir,
            Event::CharacterReceived('e') => Action::Down,
            Event::CharacterReceived('i') => Action::Up,
            Event::CharacterReceived('o') => Action::Open,
            Event::CharacterReceived('d') => Action::Delete,
            Event::CharacterReceived('t') => Action::ToggleCurrent,
            Event::CharacterReceived('s') | Event::CharacterReceived('/') => {
                Action::NewMode(Mode::Search(SearchMode::Regular))
            }
            // Event::CharacterReceived('g') => Action::SearchMode(SearchMode::Global(10)),
            Event::CharacterReceived('q') => Action::Quit,
            _ => Action::None,
        }
    }

    fn parse_search(key: Event) -> Action {
        // let KeyEvent { code, modifiers } = key;

        //TOOD move around in search with shift
        // if modifiers.contains(KeyModifiers::SHIFT) {
        //     match code {
        //         Event::CharacterReceived(c) => {
        //             return Mode::parse_normal(KeyEvent::new(
        //                 Event::CharacterReceived(c.to_ascii_lowercase()),
        //                 KeyModifiers::NONE,
        //             ));
        //         }
        //         _ => (),
        //     }
        // }

        // let enter = pressed(KeyCode::Enter);
        if let Event::KeyPressed { key_code, .. } = key {
            match key_code {
                KeyCode::Enter => Action::FreezeSearch,
                KeyCode::Backspace => Action::PopFromSearch,
                KeyCode::Escape => Action::NewMode(Mode::Normal),
                _ => Action::None,
            }
        } else {
            match key {
                Event::CharacterReceived(c) if c.is_alphabetic() => Action::AddToSearch(c),
                _ => Action::None,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Regular,
    // Files in subdirectories as well
    // .0 is depth
    // Global(usize),
}

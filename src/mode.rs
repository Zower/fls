use std::fmt::Debug;

use iced::keyboard::Modifiers;
use iced_native::keyboard::{Event, KeyCode};

use crate::app::{Action, SettingsView, View};

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
        if let Event::KeyPressed {
            key_code,
            modifiers,
        } = key
        {
            match key_code {
                KeyCode::Escape => Action::NewMode(Mode::Normal),
                KeyCode::N => Action::UpDir,
                KeyCode::E => Action::Down,
                KeyCode::I => Action::Up,
                KeyCode::O => Action::Open,
                KeyCode::D => Action::Delete,
                KeyCode::T => Action::ToggleCurrent,
                KeyCode::S if modifiers.contains(Modifiers::CTRL) => {
                    Action::NewView(View::Settings(SettingsView::default()))
                }
                KeyCode::S | KeyCode::Slash => Action::NewMode(Mode::Search(SearchMode::Regular)),
                KeyCode::Q => Action::Quit,
                _ => Action::None,
            }
        } else {
            Action::None
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

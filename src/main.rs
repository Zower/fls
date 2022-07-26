#![deny(
    missing_debug_implementations,
    unsafe_code,
    unused_results,
    warnings,
    clippy::all,
    rust_2018_idioms
)]

mod app;
mod mode;
mod tasks;
mod theme;
mod ui;

use std::io;

use app::Fls;
use iced::{pure::Application, Settings};

fn main() -> Result<(), io::Error> {
    Fls::run(Settings {
        flags: std::env::current_dir().unwrap(),
        id: None,
        window: iced::window::Settings {
            size: (800, 600),
            position: iced::window::Position::Centered,
            min_size: Some((400, 300)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
        },
        default_font: None,
        default_text_size: 20,
        text_multithreading: false,
        antialiasing: true,
        exit_on_close_request: true,
        try_opengles_first: false,
    })
    .unwrap();

    Ok(())
}

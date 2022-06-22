#![feature(drain_filter)]

mod app;
mod events;
mod task;
mod ui;

use std::{io, time::Duration};

use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::Events;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(std::env::current_dir().unwrap());

    let mut rcv = Events::start();

    app.init();

    loop {
        if app.should_quit {
            break;
        }

        terminal.draw(|f| {
            ui::draw(f, &app);
        })?;

        while let Ok(key) = rcv.try_recv() {
            app.parse_key(key.code);
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
        app.tick();
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}

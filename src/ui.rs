use std::io;

use tui::layout::Rect;
use tui::widgets::{BorderType, Clear, Paragraph};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::{App, Mode};

type Frame<'a> = tui::Frame<'a, CrosstermBackend<io::Stdout>>;

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(Clear, f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(93), Constraint::Percentage(7)].as_ref())
        .split(f.size());

    render_files(f, app, chunks[0]);

    let s;

    let text = if app.mode == Mode::Search {
        s = format!("{} {}", "/", app.search_term.as_str());
        &s
    } else {
        app.search_term.as_str()
    };

    let text = Paragraph::new(text).block(
        Block::default()
            .title("Search")
            .style(Style::default())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(text, chunks[1]);
}

pub fn render_files(f: &mut Frame, app: &App, rect: Rect) {
    let list = List::new(
        app.files
            .iter()
            .map(|f| {
                let mut item = ListItem::new(format!(
                    "{}{}",
                    f.borrow().name,
                    if f.borrow().metadata.is_dir() {
                        "/"
                    } else {
                        ""
                    },
                ));

                if f.borrow().selected {
                    item = item.style(Style::default().fg(Color::LightBlue));
                } else if f.borrow().metadata.is_dir() {
                    item = item.style(Style::default().fg(Color::Rgb(242, 89, 18)));
                }

                item
            })
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("Files").borders(Borders::ALL))
    .style(Style::default().fg(Color::White))
    .highlight_symbol("> ");

    let mut state = ListState::default();

    state.select(Some(app.find_hover().map(|(idx, _)| idx).unwrap_or(0)));

    f.render_stateful_widget(list, rect, &mut state);
}

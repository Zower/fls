use iced::{
    pure::{
        text,
        widget::{Button, Column, Container, Row, Scrollable},
        Element,
    },
    Background, Length, Padding, Space,
};

use crate::{
    app::{Fls, Message},
    mode::Mode,
    theme::{colors, Theme, ThemedButton, ThemedContainer, ThemedText},
};

pub fn draw(app: &Fls) -> Element<'_, Message, iced::Renderer<Theme>> {
    // let pane_grid = PaneGrid::new(&app.pane, |pane, state| {
    //     pane_grid::Content::new(match state {
    //         PaneState::SomePane => text("This is some pane"),
    //         PaneState::AnotherKindOfPane => text("This is another kind of pane"),
    //     })
    // })
    // .on_drag(Message::Dragged)
    // .on_click(Message::Clicked)
    // .on_resize(10, Message::Resized);

    let mut col = Column::new()
        // .width(Length::Fill)
        // .height(Length::Fill)
        .push(draw_status(app))
        .push(draw_files(app));

    col = col
        // .push(Space::new(Length::Fill, Length::FillPortion(15)))
        .push(draw_search(app));

    // col.push(Scrollable::new(&mut State::new()).into());
    // .push(draw_search(app));

    // TODO sort
    // if !app.search_term.is_empty() {

    col.into()
}

pub fn draw_status(app: &Fls) -> Element<'_, Message, iced::Renderer<Theme>> {
    Container::new(
        Row::new()
            .width(Length::Fill)
            .padding(Padding::left(10))
            .push(text(app.current_dir.to_str().unwrap_or("Unknown"))),
    )
    .width(Length::Fill)
    .height(Length::Units(50))
    .style(ThemedContainer::Color(colors::DARK_GRAY))
    .center_x()
    .center_y()
    .into()
}

pub fn draw_files(app: &Fls) -> Element<'_, Message, iced::Renderer<Theme>> {
    let mut col = Column::new();

    for (idx, file) in app.files().enumerate() {
        let style = if idx == app.hovered {
            ThemedText::Hovered
        } else if file.selected {
            ThemedText::Selected
        } else {
            Default::default()
        };

        let after = if file.data.metadata.is_dir() { "/" } else { "" };

        col = col
            // .push(
            //     Container::new(text(format!("{}{after}", &file.data.name)).style(style))
            //         .style(ThemedContainer::Color(colors::SEMI_DARK_GRAY))
            //         .padding(Padding::new(7))
            //         .width(Length::Fill),
            // )
            .push(text(format!("{}{after}", &file.data.name)).style(style))
            .push(Space::new(Length::Fill, Length::Units(3)));
    }

    Container::new(
        Container::new(Scrollable::new(col))
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(Padding::new(10))
            .style(ThemedContainer::Custom(iced::container::Appearance {
                border_width: 0.6,
                border_radius: 5.,
                border_color: colors::LIGHT_GRAY,
                background: Some(Background::Color(colors::SEAFOAM_GREEN)),
                ..Theme::container_default()
            })),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(Padding::new(10))
    .into()
}

pub fn draw_search(app: &Fls) -> Element<'_, Message, iced::Renderer<Theme>> {
    let is_search = match app.mode {
        Mode::Search(_) => true,
        _ => false,
    };

    let pre = if is_search { ">" } else { "" };

    let button = Button::new(
        text(format!("{pre} {}", &app.search_term))
            .vertical_alignment(iced::alignment::Vertical::Center),
    )
    .width(Length::Units(u16::MAX))
    .height(Length::Units(37))
    .style(ThemedButton::Search(is_search))
    .on_press(Message::Button);

    Container::new(button)
        .padding(Padding::custom(0, 8, 8, 8))
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

trait PaddingExt {
    fn left(padding: u16) -> Padding {
        Padding::from([0, 0, 0, padding])
    }

    fn custom(top: u16, right: u16, bottom: u16, left: u16) -> Padding {
        Padding {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl PaddingExt for Padding {}

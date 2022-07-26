use iced::{
    container::StyleSheet,
    pure::{
        text,
        widget::{Button, Column, Container, Row, Scrollable},
        Element,
    },
    Length, Padding, Space,
};

use crate::{
    app::{Fls, Message, View},
    mode::Mode,
    theme::{colors, ContainerKind, Theme, ThemedButton, ThemedText},
};

use self::settings::draw_settings;

pub fn draw(app: &Fls) -> Element<'_, Message, iced::Renderer<Theme>> {
    match &app.curr_view {
        View::MainView => draw_main(app),
        View::Settings(s) => draw_settings(s, app),
    }
}

pub fn draw_main(app: &Fls) -> Element<'_, Message, iced::Renderer<Theme>> {
    Column::new()
        .push(draw_status(app))
        .push(draw_files(app))
        .push(draw_search(app))
        .into()
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
            .push(text(format!("{}{after}", &file.data.name)).style(style))
            .push(Space::new(Length::Fill, Length::Units(3)));
    }

    Container::new(
        Container::new(Scrollable::new(col))
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(Padding::new(10))
            .style(ContainerKind::Custom(iced::container::Appearance {
                border_width: 0.6,
                border_radius: 5.,
                border_color: colors::LIGHT_GRAY,
                // background: Some(Background::Color(colors::SEAFOAM_GREEN)),
                ..app.theme.appearance(ContainerKind::Secondary)
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
    .style(ThemedButton::Search(is_search));

    Container::new(button)
        .padding(Padding::custom(0, 8, 8, 8))
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

pub mod components {
    use iced::{
        pure::{
            text,
            widget::{Container, Row},
            Element,
        },
        Alignment, Length, Padding, Rule, Space,
    };

    use crate::{
        app::Message,
        theme::{colors, ContainerKind, Theme},
    };

    use super::PaddingExt;

    pub fn text_input<'a>(
        value_name: &str,
        placeholder: &str,
        value: &str,
        on_change: impl Fn(String) -> Message + 'a,
        on_submit: Message,
    ) -> Element<'a, Message, iced::Renderer<Theme>> {
        let row = Row::new()
            .align_items(Alignment::Center)
            .push(text(value_name).width(Length::Units(80)))
            .push(Space::new(Length::Units(8), Length::Units(0)))
            .push(Rule::vertical(2))
            .push(Space::new(Length::Units(8), Length::Units(0)))
            .push(
                iced::pure::text_input(placeholder, value, on_change)
                    .on_submit(on_submit)
                    .width(Length::Units(200))
                    .padding(Padding::new(10)),
            );

        Container::new(row)
            .center_x()
            .center_y()
            .height(Length::Units(50))
            .width(Length::Units(280))
            .padding(Padding::custom(0, 10, 0, 10))
            .style(ContainerKind::Color(colors::DARK_GRAY))
            .into()
    }
}

pub mod settings {
    use iced::{
        pure::{
            text,
            widget::{Column, Container, Row},
            Element,
        },
        Alignment, Length, Padding, Rule, Space,
    };

    use crate::{
        app::{Fls, Message, SettingsInputKind, SettingsView},
        theme::{ColorExt, Theme},
    };

    use super::{components::text_input, draw_files};

    pub fn draw_settings<'a>(
        s: &'a SettingsView,
        fls: &'a Fls,
    ) -> Element<'a, Message, iced::Renderer<Theme>> {
        let curr_primary = fls.theme.primary.to_rgb();
        let primary = text_input(
            "Primary",
            &curr_primary.to_hex_string(),
            &s.primary_input,
            |s| Message::ColorInput(SettingsInputKind::PrimaryColor, s),
            Message::SubmitColor(SettingsInputKind::PrimaryColor),
        );

        let curr_secondary = fls.theme.secondary.to_rgb();
        let secondary = text_input(
            "Secondary",
            &curr_secondary.to_hex_string(),
            &s.secondary_input,
            |s| Message::ColorInput(SettingsInputKind::SecondaryColor, s),
            Message::SubmitColor(SettingsInputKind::SecondaryColor),
        );

        let content = Column::new()
            .push(text("Settings"))
            .push(Space::new(Length::Units(0), Length::Units(50)))
            .push(primary)
            .push(Space::new(Length::Units(0), Length::Units(50)))
            .push(secondary)
            .align_items(Alignment::Center);

        let cont = Container::new(content)
            .width(Length::Shrink)
            .height(Length::Fill);

        let row = Row::new()
            .push(cont)
            .push(Rule::vertical(50))
            .push(Space::new(Length::Units(70), Length::Units(0)))
            .push(draw_files(fls));

        Container::new(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(10))
            .into()
    }
}

pub trait PaddingExt {
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

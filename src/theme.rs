use iced::{
    pure::{application, widget::button, widget::container},
    scrollable::Scroller,
    Background, Color, Vector,
};

#[allow(dead_code)]
pub mod colors {
    use iced::Color;

    pub const DARK_GRAY: Color = Color::from_rgb(0.1, 0.1, 0.1);
    pub const SEMI_DARK_GRAY: Color = Color::from_rgb(0.14, 0.14, 0.14);
    pub const LIGHT_GRAY: Color = Color::from_rgb(0.65, 0.65, 0.65);
    pub const RED: Color = Color::from_rgb(0.21, 0.11, 0.11);
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.12, 0.21, 0.19);

    pub const DARK_BLUE: Color = Color::from_rgb(0.13, 0.13, 0.23);
    pub const BLUE: Color = Color::from_rgb(0.29, 0.31, 0.41);
    pub const SKY_BLUE: Color = Color::from_rgb(0.56, 0.79, 0.9);
    pub const LIGHT_PURPLE: Color = Color::from_rgb(0.6, 0.55, 0.6);
    pub const BEIGE: Color = Color::from_rgb(0.79, 0.68, 0.65);
    pub const LIGHT: Color = Color::from_rgb(0.95, 0.91, 0.89);

    pub const LIGHT_GREEN: Color = Color::from_rgb(0., 0.88, 0.57);
    pub const DARK_GREEN: Color = Color::from_rgb(0., 0.45, 0.36);
    pub const SEAFOAM_GREEN: Color = Color::from_rgb(0.76, 0.85, 0.72);
    pub const SCALLOP_SEASHELL: Color = Color::from_rgb(0.9, 0.64, 0.6);
}

#[derive(Debug, Default)]
pub enum Theme {
    #[default]
    Default,
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _: Self::Style) -> iced::application::Appearance {
        iced::application::Appearance {
            background_color: colors::RED,
            text_color: Color::WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ThemedContainer {
    #[default]
    Default,
    Color(Color),
    Custom(iced::container::Appearance),
}

impl Theme {
    pub fn container_default() -> iced::container::Appearance {
        <Self as container::StyleSheet>::appearance(&Theme::Default, ThemedContainer::Default)
    }
}

impl container::StyleSheet for Theme {
    type Style = ThemedContainer;

    fn appearance(&self, style: Self::Style) -> iced::container::Appearance {
        let color = match style {
            ThemedContainer::Default => colors::DARK_GREEN,
            ThemedContainer::Color(color) => color,
            ThemedContainer::Custom(c) => return c,
        };

        iced::container::Appearance {
            text_color: Some(colors::LIGHT),
            background: Some(Background::Color(color)),
            border_radius: 0.,
            border_width: 0.,
            border_color: colors::DARK_BLUE,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThemedButton {
    #[default]
    Default,
    // Search is a button, deal with it
    Search(bool),
}
impl button::StyleSheet for Theme {
    type Style = ThemedButton;

    fn active(&self, style: Self::Style) -> iced::button::Appearance {
        let color = match style {
            ThemedButton::Default => colors::LIGHT_PURPLE,
            ThemedButton::Search(is_active) => {
                if is_active {
                    colors::RED
                } else {
                    colors::LIGHT_GREEN
                }
            }
        };

        iced::button::Appearance {
            shadow_offset: Vector::new(0., 0.),
            background: Some(Background::Color(color)),
            border_radius: 0.,
            border_width: 1.,
            border_color: colors::DARK_BLUE,
            text_color: colors::LIGHT,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ThemedText {
    #[default]
    Default,
    Hovered,
    Selected,
}

impl iced_native::widget::text::StyleSheet for Theme {
    type Style = ThemedText;

    fn appearance(&self, style: Self::Style) -> iced_native::widget::text::Appearance {
        let color = match style {
            ThemedText::Default => colors::LIGHT,
            ThemedText::Hovered => colors::LIGHT_GREEN,
            ThemedText::Selected => colors::DARK_BLUE,
        };

        iced_native::widget::text::Appearance { color: Some(color) }
    }
}

impl iced::scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _: Self::Style) -> iced::scrollable::Scrollbar {
        iced::scrollable::Scrollbar {
            background: None,
            border_radius: 0.,
            border_width: 0.,
            border_color: colors::DARK_BLUE,
            scroller: Scroller {
                color: colors::BEIGE,
                border_radius: 0.,
                border_width: 0.,
                border_color: colors::DARK_BLUE,
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> iced::scrollable::Scrollbar {
        self.active(style)
    }
}

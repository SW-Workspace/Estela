pub mod tokens;

use iced::{
    widget::{button, container},
    Background, Border, Theme,
};

pub fn theme() -> Theme {
    Theme::custom(
        "Estela",
        iced::theme::Palette {
            background: tokens::BASE,
            text: tokens::TEXT,
            primary: tokens::ACCENT,
            success: iced::Color::from_rgb8(50, 215, 75),
            warning: iced::Color::from_rgb8(255, 214, 10),
            danger: tokens::RECORD,
        },
    )
}

pub fn panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tokens::ELEVATED)),
        text_color: Some(tokens::TEXT),
        border: Border {
            radius: 14.0.into(),
            width: 1.0,
            color: tokens::BORDER,
        },
        ..Default::default()
    }
}

pub fn record(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.background = Some(tokens::RECORD.into());
    style.text_color = iced::Color::WHITE;
    style.border.radius = 16.0.into();
    style.shadow = iced::Shadow {
        color: tokens::RECORD.scale_alpha(0.35),
        offset: iced::Vector::ZERO,
        blur_radius: 14.0,
    };
    if status == button::Status::Disabled {
        style.background = Some(tokens::BORDER.into());
        style.text_color = tokens::SECONDARY;
    }
    style
}

pub fn pill(theme: &Theme) -> container::Style {
    let mut style = panel(theme);
    style.border.radius = 28.0.into();
    style.shadow = iced::Shadow {
        color: iced::Color::BLACK.scale_alpha(0.4),
        offset: iced::Vector::new(0.0, 6.0),
        blur_radius: 18.0,
    };
    style
}

pub fn secondary(_theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(
            if status == button::Status::Hovered {
                tokens::BORDER
            } else {
                tokens::SURFACE
            }
            .into(),
        ),
        text_color: tokens::TEXT,
        border: Border {
            color: tokens::BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn quiet(_theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: (status == button::Status::Hovered).then_some(tokens::BORDER.into()),
        text_color: tokens::SECONDARY,
        border: Border {
            radius: 16.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn chip(_theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(
            if status == button::Status::Hovered {
                tokens::BORDER
            } else {
                tokens::SURFACE
            }
            .into(),
        ),
        text_color: tokens::TEXT,
        border: Border {
            radius: 16.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn muted(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = chip(theme, status);
    style.background = Some(tokens::WARNING.scale_alpha(0.13).into());
    style.text_color = tokens::WARNING;
    style
}

pub fn stop(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = chip(theme, status);
    style.background = Some(tokens::TEXT.into());
    style.text_color = tokens::BASE;
    style
}

pub fn resume(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = chip(theme, status);
    style.background = Some(tokens::ACCENT.into());
    style.text_color = tokens::TEXT;
    style
}

pub fn source_item(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: Some(
            if selected {
                tokens::SELECTED
            } else if status == button::Status::Hovered {
                tokens::BORDER
            } else {
                tokens::ELEVATED
            }
            .into(),
        ),
        text_color: tokens::TEXT,
        border: Border {
            color: if selected {
                tokens::ACCENT
            } else {
                iced::Color::TRANSPARENT
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

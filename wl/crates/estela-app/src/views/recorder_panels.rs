use iced::{
    widget::{button, checkbox, column, container, pick_list, row, scrollable, space, text},
    Alignment, Element, Length,
};

use estela_engine::api::{Source, SourceKind, SourceSelection};

use crate::{
    app::{App, Popover},
    message::Message,
    theme::{self, tokens},
    widgets::icon::{self, Icon},
    windows::recording_bar::control,
};

pub(crate) fn view(app: &App, popover: Popover) -> Element<'_, Message> {
    let content = match popover {
        Popover::Source => sources(app),
        Popover::Microphone => microphone(app),
        Popover::Menu => menu(app),
        Popover::Settings => settings(app),
        Popover::Shortcuts => shortcuts(app),
    };
    container(scrollable(content))
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel)
        .into()
}

fn heading<'a>(app: &'a App, title: &'a str) -> Element<'a, Message> {
    row![
        space().width(30),
        container(
            text(app.strings.get(title))
                .size(13)
                .color(tokens::SECONDARY)
        )
        .center_x(Length::Fill),
        control(app, Icon::Close, "back", Message::DismissPopover)
    ]
    .align_y(Alignment::Center)
    .into()
}

fn sources(app: &App) -> Element<'_, Message> {
    let kind_button = |kind, name, glyph| {
        button(
            row![
                icon::view(glyph, tokens::SECONDARY, 16.0),
                text(app.strings.get(name)).size(13)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::Kind(kind))
        .padding([9, 12])
        .width(Length::Fill)
        .style(theme::source_item(app.kind == kind))
    };
    let mut items = column![row![
        kind_button(SourceKind::Display, "display", Icon::Screen),
        kind_button(SourceKind::Window, "window", Icon::Window),
        control(app, Icon::Refresh, "refresh", Message::Refresh),
        control(app, Icon::Close, "back", Message::DismissPopover)
    ]
    .spacing(6)]
    .spacing(8);
    let sources: Vec<_> = app
        .sources
        .iter()
        .filter(|source| source.kind == app.kind)
        .collect();
    if sources.is_empty() {
        items = items.push(
            text(app.strings.get("no-source"))
                .size(13)
                .color(tokens::SECONDARY),
        );
    }
    for source in sources {
        items = items.push(source_item(app, source));
    }
    if app
        .selected
        .as_ref()
        .is_some_and(|source| source.selection == SourceSelection::SystemPicker)
    {
        items = items.push(
            button(app.strings.get(if app.kind == SourceKind::Window {
                "choose-window"
            } else {
                "choose-display"
            }))
            .on_press(Message::ChooseSource)
            .padding([9, 12])
            .width(Length::Fill)
            .style(theme::secondary),
        );
    }
    if let Some(error) = &app.error {
        items = items.push(super::recorder::error_view(app, error));
    }
    items.into()
}

fn source_item<'a>(app: &'a App, source: &'a Source) -> Element<'a, Message> {
    let selected = app
        .selected
        .as_ref()
        .is_some_and(|current| current.id == source.id && current.kind == source.kind);
    let kind = if source.kind == SourceKind::Display {
        Icon::Screen
    } else {
        Icon::Window
    };
    let name = if source.name.is_empty() {
        app.strings.get(if source.kind == SourceKind::Display {
            "portal-display"
        } else {
            "portal-window"
        })
    } else {
        &source.name
    };
    let tile = container(icon::view(kind, tokens::TEXT, 22.0))
        .center(44)
        .style(|_| container::Style {
            background: Some(
                iced::gradient::Linear::new(iced::Radians(0.8))
                    .add_stop(0.0, tokens::ACCENT)
                    .add_stop(1.0, iced::Color::from_rgb8(68, 100, 160))
                    .into(),
            ),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });
    let label = column![
        text(name).size(13),
        text(app.strings.get(if selected {
            "selected"
        } else if source.kind == SourceKind::Display {
            "display"
        } else {
            "window"
        }))
        .size(12)
        .color(if selected {
            tokens::ACCENT
        } else {
            tokens::SECONDARY
        })
    ]
    .spacing(3)
    .width(Length::Fill);
    let mut content = row![tile, label].spacing(12).align_y(Alignment::Center);
    if selected {
        content = content.push(icon::view(Icon::Check, tokens::ACCENT, 16.0));
    }
    button(content)
        .padding(8)
        .width(Length::Fill)
        .on_press(Message::Source(source.clone()))
        .style(theme::source_item(selected))
        .into()
}

fn microphone(app: &App) -> Element<'_, Message> {
    column![
        heading(app, "microphone"),
        row![
            icon::view(Icon::MicOff, tokens::WARNING, 20.0),
            text(app.strings.get("no-audio"))
                .size(14)
                .color(tokens::WARNING)
        ]
        .spacing(10),
        checkbox(true).label(app.strings.get("mute-microphone")),
        text(app.strings.get("silent"))
            .size(13)
            .color(tokens::SECONDARY)
    ]
    .spacing(12)
    .into()
}

fn menu(app: &App) -> Element<'_, Message> {
    column![
        menu_item(app, Icon::Folder, "open", Message::Open),
        menu_item(
            app,
            Icon::Settings,
            "settings",
            Message::TogglePopover(Popover::Settings)
        ),
        menu_item(
            app,
            Icon::Keyboard,
            "shortcut-settings",
            Message::TogglePopover(Popover::Shortcuts)
        ),
        menu_item(app, Icon::Close, "quit", Message::Close),
    ]
    .spacing(4)
    .into()
}

fn menu_item<'a>(
    app: &'a App,
    glyph: Icon,
    name: &'a str,
    message: Message,
) -> Element<'a, Message> {
    let enabled = app.state == crate::app::State::Ready
        || !matches!(
            message,
            Message::Open | Message::TogglePopover(Popover::Settings)
        );
    button(
        row![
            icon::view(glyph, tokens::SECONDARY, 16.0),
            text(app.strings.get(name)).size(13)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .on_press_maybe(enabled.then_some(message))
    .style(theme::quiet)
    .into()
}

fn settings(app: &App) -> Element<'_, Message> {
    column![
        heading(app, "settings"),
        row![
            text(app.strings.get("fps")).size(13),
            space::horizontal(),
            pick_list([30, 60], Some(app.preferences.fps), Message::Fps)
        ]
        .align_y(Alignment::Center),
        checkbox(app.preferences.countdown)
            .label(app.strings.get("countdown-setting"))
            .on_toggle(Message::Countdown),
        text(app.strings.get("silent"))
            .size(12)
            .color(tokens::SECONDARY)
    ]
    .spacing(16)
    .into()
}

fn shortcuts(app: &App) -> Element<'_, Message> {
    let status = if app.shortcuts_ready {
        "shortcuts-ready"
    } else if app.shortcuts_failed {
        "shortcuts-failed"
    } else {
        "shortcuts-pending"
    };
    let color = if app.shortcuts_ready {
        tokens::SECONDARY
    } else {
        tokens::WARNING
    };
    let mut content = column![
        heading(app, "shortcut-settings"),
        text(app.strings.get("shortcuts")).size(13),
        text(app.strings.get(status)).size(12).color(color)
    ]
    .spacing(14);
    if !app.shortcuts_ready {
        content = content.push(
            button(app.strings.get("retry-shortcuts"))
                .on_press(Message::RetryShortcuts)
                .padding([9, 12])
                .style(theme::secondary),
        );
        content = content.push(
            button(app.strings.get("choose-window"))
                .on_press(Message::Kind(SourceKind::Window))
                .padding([9, 12])
                .style(theme::secondary),
        );
    }
    if let Some(error) = &app.error {
        content = content.push(super::recorder::error_view(app, error));
    }
    content.into()
}

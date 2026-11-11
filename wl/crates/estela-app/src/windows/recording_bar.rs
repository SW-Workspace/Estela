use iced::{
    widget::{button, container, mouse_area, row, space, text, tooltip},
    Alignment, Element, Length,
};

use estela_engine::api::SourceKind;

use crate::{
    app::{App, Popover, State},
    message::Message,
    theme::{self, tokens},
    widgets::icon::{self, Icon},
};

pub const HEIGHT: f32 = 56.0;
const SOURCE_NAME_LIMIT: usize = 17;

pub(crate) fn view(app: &App) -> Element<'_, Message> {
    let controls = if matches!(app.state, State::Recording | State::Paused) {
        active(app)
    } else {
        ready(app)
    };
    mouse_area(
        container(container(controls).center_y(Length::Fill))
            .padding([8, 12])
            .width(Length::Fill)
            .height(HEIGHT)
            .style(theme::pill),
    )
    .on_press(Message::Drag)
    .into()
}

fn ready(app: &App) -> Element<'_, Message> {
    let strings = &app.strings;
    let kind = if app.kind == SourceKind::Display {
        Icon::Screen
    } else {
        Icon::Window
    };
    let fallback = strings.get(if app.kind == SourceKind::Display {
        "display"
    } else {
        "window"
    });
    let name = app
        .selected
        .as_ref()
        .filter(|source| !source.name.is_empty())
        .map_or(fallback, |source| source.name.as_str());
    let label = if name.chars().count() > SOURCE_NAME_LIMIT {
        format!(
            "{}…",
            name.chars().take(SOURCE_NAME_LIMIT).collect::<String>()
        )
    } else {
        name.to_owned()
    };
    let source = button(
        row![
            icon::view(kind, tokens::SECONDARY, 16.0),
            text(label).size(13).width(Length::Fill),
            icon::view(Icon::Chevron, tokens::SECONDARY, 10.0)
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .on_press(Message::TogglePopover(Popover::Source))
    .padding([9, 12])
    .width(168)
    .style(theme::chip);
    let record = button(
        row![
            icon::view(Icon::Record, tokens::TEXT, 14.0),
            text(strings.get("record")).size(13)
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press_maybe(app.selected.is_some().then_some(Message::Record))
    .padding([9, 14])
    .style(theme::record);
    row![
        grip(app),
        source,
        microphone(app),
        space::horizontal(),
        record,
        control(
            app,
            Icon::More,
            "more",
            Message::TogglePopover(Popover::Menu)
        )
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn active(app: &App) -> Element<'_, Message> {
    let paused = app.state == State::Paused;
    let color = if paused {
        tokens::WARNING
    } else {
        tokens::RECORD
    };
    let mut status = row![
        icon::view(Icon::Record, color, 14.0),
        text(elapsed(app.elapsed)).size(14)
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    if paused {
        status = status.push(text(app.strings.get("paused")).size(11).color(color));
    }
    let timer = container(status)
        .padding([9, 10])
        .style(move |_| container::Style {
            background: Some(color.scale_alpha(0.13).into()),
            border: iced::Border {
                radius: 18.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });
    let pause = button(icon::view(
        if paused { Icon::Play } else { Icon::Pause },
        tokens::TEXT,
        16.0,
    ))
    .padding(10)
    .on_press(Message::Pause)
    .style(if paused { theme::resume } else { theme::chip });
    let pause = tooltip(
        pause,
        text(app.strings.get(if paused { "resume" } else { "pause" })).size(12),
        tooltip::Position::Bottom,
    );
    let stop = button(
        row![
            icon::view(Icon::Stop, tokens::RECORD, 13.0),
            text(app.strings.get("stop")).size(13)
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Stop)
    .padding([9, 12])
    .style(theme::stop);
    row![
        grip(app),
        timer,
        control(
            app,
            Icon::MicOff,
            "no-audio",
            Message::TogglePopover(Popover::Microphone)
        ),
        space::horizontal(),
        pause,
        stop,
        control(app, Icon::Restart, "restart", Message::Restart),
        control(
            app,
            Icon::More,
            "more",
            Message::TogglePopover(Popover::Menu)
        )
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn grip(app: &App) -> Element<'_, Message> {
    tooltip(
        mouse_area(container(icon::view(Icon::Grip, tokens::SECONDARY, 16.0)).padding(7))
            .on_press(Message::Drag)
            .interaction(iced::mouse::Interaction::Grab),
        text(app.strings.get("drag")).size(12),
        tooltip::Position::Bottom,
    )
    .into()
}

fn microphone(app: &App) -> Element<'_, Message> {
    let content = row![
        icon::view(Icon::MicOff, tokens::WARNING, 15.0),
        text(app.strings.get("no-audio")).size(12)
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    button(content)
        .on_press(Message::TogglePopover(Popover::Microphone))
        .padding([9, 9])
        .style(theme::muted)
        .into()
}

pub(crate) fn control<'a>(
    app: &'a App,
    kind: Icon,
    label: &'a str,
    message: Message,
) -> Element<'a, Message> {
    tooltip(
        button(icon::view(kind, tokens::SECONDARY, 16.0))
            .padding(7)
            .on_press(message)
            .style(theme::quiet),
        text(app.strings.get(label)).size(12),
        tooltip::Position::Bottom,
    )
    .into()
}

fn elapsed(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

use iced::{
    widget::{button, column, container, row, scrollable, space, text},
    Alignment, Element, Length,
};

use crate::{
    app::{App, State},
    message::Message,
    theme::{self, tokens},
    windows::{countdown, recording_bar},
};

pub fn view(app: &App) -> Element<'_, Message> {
    let content = match app.state {
        State::Ready | State::Recording | State::Paused => controls(app),
        State::Preparing | State::Loading | State::Saving | State::Starting => waiting(app),
        State::Countdown => countdown::view(app),
        State::Playback => crate::widgets::preview::view(app),
    };
    let content = if matches!(app.state, State::Ready | State::Recording | State::Paused) {
        container(content)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
    } else {
        container(container(content).width(320).height(380)).center(Length::Fill)
    };
    content
        .style(|_| container::Style {
            text_color: Some(tokens::TEXT),
            ..Default::default()
        })
        .into()
}

fn controls(app: &App) -> Element<'_, Message> {
    let mut content = column![recording_bar::view(app)].spacing(12);
    if let Some(popover) = app.popover {
        let (width, height) = popover.panel_size();
        let panel = container(super::recorder_panels::view(app, popover))
            .width(width as f32)
            .height(height as f32);
        content = content.push(container(panel).center_x(Length::Fill));
    }
    content.into()
}

fn waiting(app: &App) -> Element<'_, Message> {
    let key = match app.state {
        State::Preparing => "preparing",
        State::Loading => "loading",
        State::Starting => "countdown",
        _ => "saving",
    };
    let mut content = column![
        space::vertical(),
        text(app.strings.get(key)).size(16),
        space::vertical()
    ]
    .spacing(16)
    .align_x(Alignment::Center);
    if app.state == State::Preparing {
        content = content.push(
            button(app.strings.get("cancel"))
                .on_press(Message::Cancel)
                .style(theme::secondary),
        );
    }
    container(content)
        .padding(20)
        .height(Length::Fill)
        .width(Length::Fill)
        .style(theme::panel)
        .into()
}

pub(crate) fn error_view<'a>(app: &'a App, error: &'a str) -> Element<'a, Message> {
    row![
        scrollable(column![
            text(app.strings.get("error"))
                .size(13)
                .color(tokens::RECORD),
            text(error).size(12)
        ])
        .height(64),
        button(app.strings.get("dismiss"))
            .on_press(Message::Dismiss)
            .style(theme::quiet)
    ]
    .spacing(8)
    .into()
}

pub(crate) fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u64;
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

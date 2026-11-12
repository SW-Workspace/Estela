use iced::{
    widget::{button, column, container, image, row, slider, space, text, toggler},
    Alignment, Element, Length,
};

use crate::{
    app::App,
    message::Message,
    theme::tokens,
    views::recorder::{clock, error_view},
};

pub(crate) fn view(app: &App) -> Element<'_, Message> {
    let strings = &app.strings;
    let title = app
        .recording
        .as_ref()
        .map(|recording| recording.title.as_str())
        .unwrap_or_default();
    let canvas: Element<'_, Message> = if let Some(handle) = &app.image {
        image(handle.clone())
            .content_fit(iced::ContentFit::Contain)
            .border_radius(12)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        container(text(strings.get("loading")))
            .center(Length::Fill)
            .into()
    };
    let stage = container(container(canvas).padding(2).style(|_| container::Style {
        background: Some(tokens::SURFACE.into()),
        border: iced::Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        shadow: iced::Shadow {
            color: iced::Color::BLACK.scale_alpha(0.4),
            offset: iced::Vector::new(0.0, 12.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }))
    .padding(40)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(
            iced::gradient::Linear::new(iced::Radians(0.75))
                .add_stop(0.0, iced::Color::from_rgb8(57, 39, 100))
                .add_stop(1.0, iced::Color::from_rgb8(20, 37, 61))
                .into(),
        ),
        border: iced::Border {
            radius: 14.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });
    let count = app
        .recording
        .as_ref()
        .map_or(0, |recording| recording.zooms.len());
    let mut content = column![
        row![
            button(strings.get("new"))
                .on_press(Message::New)
                .style(button::text),
            text(title).size(16),
            space::horizontal(),
            button(strings.get("open"))
                .on_press(Message::Open)
                .style(button::text),
            button(strings.get("close"))
                .on_press(Message::Close)
                .style(button::text)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        row![
            text(strings.get("preview")).size(24),
            space::horizontal(),
            text(format!("{count} {}", strings.get("zooms")))
                .size(12)
                .color(tokens::SECONDARY),
            toggler(app.autozoom)
                .label(strings.get("autozoom"))
                .on_toggle(Message::AutoZoom)
        ]
        .spacing(16)
        .align_y(Alignment::Center),
        stage,
        slider(0.0..=app.duration().max(0.001), app.position, Message::Seek)
            .step(0.001)
            .on_release(Message::SeekCommit),
        row![
            button(strings.get("frame-back"))
                .on_press(Message::Step(-1))
                .style(button::text),
            button(strings.get(if app.playing { "play-pause" } else { "play" }))
                .on_press(Message::Play)
                .padding([10, 24]),
            button(strings.get("frame-next"))
                .on_press(Message::Step(1))
                .style(button::text),
            space::horizontal(),
            text(format!(
                "{} / {}",
                clock(app.position),
                clock(app.duration())
            ))
            .size(14),
            button(strings.get("fullscreen"))
                .on_press(Message::Fullscreen)
                .style(button::text)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        text(strings.get("stored"))
            .size(12)
            .color(tokens::SECONDARY),
    ]
    .spacing(16);
    if let Some(recording) = &app.recording {
        if recording.zoom_trigger == estela_engine::api::ZoomTrigger::CursorDwell {
            content = content.push(
                text(strings.get("wayland-dwell"))
                    .size(12)
                    .color(tokens::SECONDARY),
            );
        }
        if recording.zoom_trigger == estela_engine::api::ZoomTrigger::Unavailable {
            content = content.push(
                text(strings.get("wayland-zoom"))
                    .size(12)
                    .color(tokens::SECONDARY),
            );
        }
    }
    if let Some(error) = &app.error {
        content = content.push(error_view(app, error));
    }
    container(content)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(tokens::BASE.into()),
            text_color: Some(tokens::TEXT),
            ..Default::default()
        })
        .into()
}

use iced::{
    widget::{column, container, image, row, slider, text, toggler, tooltip},
    Alignment, Element, Length,
};

use estela_engine::api::ZoomTrigger;

use crate::{
    app::App,
    message::Message,
    theme::tokens,
    views::recorder::{clock, error_view},
    widgets::icon::{self, Icon},
    windows::recording_bar::control,
};

const PAGE_PADDING: u16 = 16;
const SECTION_SPACING: f32 = 12.0;
const CONTROL_SPACING: f32 = 8.0;
const STAGE_PADDING: u16 = 16;
const STAGE_RADIUS: f32 = 12.0;
const TITLE_SIZE: f32 = 16.0;
const CLOCK_SIZE: f32 = 14.0;
const NOTE_SIZE: f32 = 12.0;

pub(crate) fn view(app: &App) -> Element<'_, Message> {
    let mut content = column![
        header(app),
        stage(app),
        slider(0.0..=app.duration().max(0.001), app.position, Message::Seek)
            .step(0.001)
            .on_release(Message::SeekCommit),
        transport(app),
    ]
    .spacing(SECTION_SPACING);
    if let Some(error) = &app.error {
        content = content.push(error_view(app, error));
    }
    container(content)
        .padding(PAGE_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(tokens::BASE.into()),
            text_color: Some(tokens::TEXT),
            ..Default::default()
        })
        .into()
}

fn header(app: &App) -> Element<'_, Message> {
    let title = app
        .recording
        .as_ref()
        .map(|recording| recording.title.as_str())
        .unwrap_or_default();
    // A long title is clipped so it never pushes the actions out of view.
    let title = container(text(title).size(TITLE_SIZE).wrapping(text::Wrapping::None))
        .width(Length::Fill)
        .clip(true);
    row![
        title,
        control(app, Icon::Record, "new", Message::New),
        control(app, Icon::Folder, "open", Message::Open),
        control(app, Icon::Close, "close", Message::Close),
    ]
    .spacing(CONTROL_SPACING)
    .align_y(Alignment::Center)
    .into()
}

fn stage(app: &App) -> Element<'_, Message> {
    let canvas: Element<'_, Message> = match &app.image {
        Some(handle) => image(handle.clone())
            .content_fit(iced::ContentFit::Contain)
            .border_radius(STAGE_RADIUS)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        None => text(app.strings.get("loading")).into(),
    };
    container(canvas)
        .padding(STAGE_PADDING)
        .center(Length::Fill)
        .style(|_| container::Style {
            background: Some(
                iced::gradient::Linear::new(iced::Radians(0.75))
                    .add_stop(0.0, iced::Color::from_rgb8(57, 39, 100))
                    .add_stop(1.0, iced::Color::from_rgb8(20, 37, 61))
                    .into(),
            ),
            border: iced::Border {
                radius: STAGE_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn transport(app: &App) -> Element<'_, Message> {
    let (play_icon, play_label) = if app.playing {
        (Icon::Pause, "play-pause")
    } else {
        (Icon::Play, "play")
    };
    let position = format!("{} / {}", clock(app.position), clock(app.duration()));
    // Wrapping keeps every control reachable when the window is narrow.
    row![
        control(app, Icon::Previous, "frame-back", Message::Step(-1)),
        control(app, play_icon, play_label, Message::Play),
        control(app, Icon::Next, "frame-next", Message::Step(1)),
        text(position).size(CLOCK_SIZE),
        control(app, Icon::Maximize, "fullscreen", Message::Fullscreen),
        zoom_control(app),
    ]
    .spacing(CONTROL_SPACING)
    .align_y(Alignment::Center)
    .wrap()
    .vertical_spacing(CONTROL_SPACING)
    .into()
}

fn zoom_control(app: &App) -> Element<'_, Message> {
    let strings = &app.strings;
    let count = app
        .recording
        .as_ref()
        .map_or(0, |recording| recording.zooms.len());
    let mut hint = column![text(strings.get("autozoom")).size(NOTE_SIZE)];
    let note = match app
        .recording
        .as_ref()
        .map(|recording| recording.zoom_trigger)
    {
        Some(ZoomTrigger::CursorDwell) => Some("wayland-dwell"),
        Some(ZoomTrigger::Unavailable) => Some("wayland-zoom"),
        _ => None,
    };
    if let Some(key) = note {
        hint = hint.push(text(strings.get(key)).size(NOTE_SIZE));
    }
    tooltip(
        row![
            icon::view(Icon::Zoom, tokens::SECONDARY, 16.0),
            text(count.to_string())
                .size(NOTE_SIZE)
                .color(tokens::SECONDARY),
            toggler(app.autozoom).on_toggle(Message::AutoZoom),
        ]
        .spacing(CONTROL_SPACING)
        .align_y(Alignment::Center),
        hint.max_width(280),
        tooltip::Position::Top,
    )
    .into()
}

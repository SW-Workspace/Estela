use iced::{
    widget::{button, column, container, image, row, slider, text, toggler},
    Alignment, Element, Length,
};

use estela_engine::api::ZoomTrigger;

use crate::{
    app::App,
    message::Message,
    theme::tokens,
    views::recorder::{clock, error_view},
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
    for note in notes(app) {
        content = content.push(note);
    }
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
    let strings = &app.strings;
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
        text_button(strings.get("new"), Message::New),
        text_button(strings.get("open"), Message::Open),
        text_button(strings.get("close"), Message::Close),
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
    let strings = &app.strings;
    let count = app
        .recording
        .as_ref()
        .map_or(0, |recording| recording.zooms.len());
    let play = button(strings.get(if app.playing { "play-pause" } else { "play" }))
        .on_press(Message::Play)
        .padding([8, 20]);
    let position = format!("{} / {}", clock(app.position), clock(app.duration()));
    // Wrapping keeps every control reachable when the window is narrow.
    row![
        text_button(strings.get("frame-back"), Message::Step(-1)),
        play,
        text_button(strings.get("frame-next"), Message::Step(1)),
        text(position).size(CLOCK_SIZE),
        text_button(strings.get("fullscreen"), Message::Fullscreen),
        toggler(app.autozoom)
            .label(strings.get("autozoom"))
            .on_toggle(Message::AutoZoom),
        text(format!("{count} {}", strings.get("zooms")))
            .size(NOTE_SIZE)
            .color(tokens::SECONDARY),
    ]
    .spacing(CONTROL_SPACING)
    .align_y(Alignment::Center)
    .wrap()
    .vertical_spacing(CONTROL_SPACING)
    .into()
}

fn notes(app: &App) -> Vec<Element<'_, Message>> {
    let strings = &app.strings;
    let mut keys = vec!["stored"];
    match app
        .recording
        .as_ref()
        .map(|recording| recording.zoom_trigger)
    {
        Some(ZoomTrigger::CursorDwell) => keys.push("wayland-dwell"),
        Some(ZoomTrigger::Unavailable) => keys.push("wayland-zoom"),
        _ => {}
    }
    keys.into_iter()
        .map(|key| {
            text(strings.get(key))
                .size(NOTE_SIZE)
                .color(tokens::SECONDARY)
                .into()
        })
        .collect()
}

fn text_button(label: &str, message: Message) -> Element<'_, Message> {
    button(text(label))
        .on_press(message)
        .style(button::text)
        .into()
}

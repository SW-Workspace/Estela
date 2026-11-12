use iced::{
    mouse,
    widget::{
        button,
        canvas::{self, Frame, Path, Stroke},
        column, container, space, stack, text,
    },
    Alignment, Element, Length, Point, Rectangle, Renderer, Theme,
};

use crate::{app::App, message::Message, theme::tokens};

const COUNTER_RADIUS: f32 = 82.0;
const NUMBER_SIZE: f32 = 96.0;

pub(crate) fn view(app: &App) -> Element<'_, Message> {
    let counter = canvas::Canvas::new(Counter {
        fraction: app.countdown_fraction(),
        number: app.countdown,
    })
    .width(Length::Fill)
    .height(Length::Fill);
    container(
        stack![
            counter,
            column![
                text(app.strings.get("countdown"))
                    .size(14)
                    .color(tokens::TEXT.scale_alpha(0.8)),
                space::vertical(),
                button(text(app.strings.get("escape")).size(12))
                    .on_press(Message::Cancel)
                    .padding([10, 16])
                    .style(crate::theme::chip),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(
            iced::gradient::Linear::new(iced::Radians(0.7))
                .add_stop(0.0, iced::Color::from_rgb8(255, 108, 193))
                .add_stop(1.0, tokens::ACCENT)
                .into(),
        ),
        border: iced::Border {
            radius: 16.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

struct Counter {
    fraction: f32,
    number: u64,
}

impl<Message> canvas::Program<Message> for Counter {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = COUNTER_RADIUS;
        let circle = Path::circle(center, radius);
        frame.fill(&circle, tokens::BASE.scale_alpha(0.68));
        frame.stroke(
            &circle,
            Stroke::default()
                .with_color(tokens::TEXT.scale_alpha(0.22))
                .with_width(5.0),
        );
        let start = -std::f32::consts::FRAC_PI_2;
        let end = start + std::f32::consts::TAU * self.fraction;
        let arc = Path::new(|path| {
            path.arc(canvas::path::Arc {
                center,
                radius,
                start_angle: iced::Radians(start),
                end_angle: iced::Radians(end),
            })
        });
        frame.stroke(
            &arc,
            Stroke::default()
                .with_color(tokens::TEXT.scale_alpha(0.8))
                .with_width(5.0),
        );
        frame.fill(
            &Path::circle(
                Point::new(center.x + radius * end.cos(), center.y + radius * end.sin()),
                5.0,
            ),
            tokens::TEXT,
        );
        frame.fill_text(canvas::Text {
            content: self.number.to_string(),
            position: center,
            color: tokens::TEXT,
            size: NUMBER_SIZE.into(),
            font: iced::Font::with_name("Inter Variable"),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Default::default()
        });
        vec![frame.into_geometry()]
    }
}

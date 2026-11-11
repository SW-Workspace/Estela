#![forbid(unsafe_code)]

use estela_app::{app::App, theme};

fn main() -> iced::Result {
    // Wayland needs alpha buffers; X11 shapes also support software rendering.
    std::env::set_var("ICED_BACKEND", estela_engine::api::presentation_backend());
    iced::daemon(App::boot, App::update, App::view)
        .title("Estela")
        .settings(iced::Settings {
            id: Some("app.estela.Estela".into()),
            ..Default::default()
        })
        .font(include_bytes!("../assets/InterVariable.ttf").as_slice())
        .default_font(iced::Font::with_name("Inter Variable"))
        .theme(theme::theme())
        .style(|_, _| iced::theme::Style {
            background_color: iced::Color::TRANSPARENT,
            text_color: theme::tokens::TEXT,
        })
        .subscription(App::subscription)
        .run()
}

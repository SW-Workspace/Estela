use iced::{widget::svg, Color, Element};

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Grip,
    Screen,
    Window,
    MicOff,
    Chevron,
    More,
    Record,
    Pause,
    Play,
    Stop,
    Restart,
    Folder,
    Settings,
    Keyboard,
    Close,
    Check,
    Refresh,
}

pub fn view<'a, Message: 'a>(kind: Icon, color: Color, size: f32) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(kind.bytes()))
        .width(size)
        .height(size)
        .content_fit(iced::ContentFit::Contain)
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

impl Icon {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Grip | Self::More => include_bytes!("../../../../../assets/icons/more.svg"),
            Self::Screen => include_bytes!("../../../../../assets/icons/monitor.svg"),
            Self::Window => include_bytes!("../../../../../assets/icons/window.svg"),
            Self::MicOff => include_bytes!("../../../../../assets/icons/mic-off.svg"),
            Self::Chevron => include_bytes!("../../../../../assets/icons/chevron-down.svg"),
            Self::Record => include_bytes!("../../../../../assets/icons/film.svg"),
            Self::Pause => include_bytes!("../../../../../assets/icons/pause.svg"),
            Self::Play => include_bytes!("../../../../../assets/icons/play.svg"),
            Self::Stop => include_bytes!("../../../../../assets/icons/stop.svg"),
            Self::Restart | Self::Refresh => {
                include_bytes!("../../../../../assets/icons/restart.svg")
            }
            Self::Folder => include_bytes!("../../../../../assets/icons/folder.svg"),
            Self::Settings => include_bytes!("../../../../../assets/icons/settings.svg"),
            Self::Keyboard => include_bytes!("../../../../../assets/icons/keyboard.svg"),
            Self::Close => include_bytes!("../../../../../assets/icons/x.svg"),
            Self::Check => include_bytes!("../../../../../assets/icons/check.svg"),
        }
    }
}

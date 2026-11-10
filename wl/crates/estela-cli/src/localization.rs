pub(super) fn message(name: &str) -> &'static str {
    let language = std::env::var("LC_ALL")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("LC_MESSAGES")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .or_else(|| std::env::var("LANG").ok().filter(|value| !value.is_empty()))
        .unwrap_or_default();
    let spanish = language.starts_with("es");
    match (name, spanish) {
        ("help", false) => include_str!("../../../locales/en/help.txt").trim_end(),
        ("help", true) => include_str!("../../../locales/es/help.txt").trim_end(),
        ("not-implemented", false) => {
            include_str!("../../../locales/en/not-implemented.txt").trim_end()
        }
        ("not-implemented", true) => {
            include_str!("../../../locales/es/not-implemented.txt").trim_end()
        }
        ("unknown-command", false) => {
            include_str!("../../../locales/en/unknown-command.txt").trim_end()
        }
        ("unknown-command", true) => {
            include_str!("../../../locales/es/unknown-command.txt").trim_end()
        }
        _ => "",
    }
}

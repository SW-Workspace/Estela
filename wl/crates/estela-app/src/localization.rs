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
        ("app-pending", false) => include_str!("../../../locales/en/app-pending.txt").trim_end(),
        ("app-pending", true) => include_str!("../../../locales/es/app-pending.txt").trim_end(),
        _ => "",
    }
}

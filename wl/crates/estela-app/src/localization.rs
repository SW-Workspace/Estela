use std::collections::BTreeMap;

pub struct Strings {
    english: BTreeMap<String, String>,
    translated: BTreeMap<String, String>,
}

impl Default for Strings {
    fn default() -> Self {
        let language = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
            .unwrap_or_default();
        Self {
            english: serde_json::from_str(include_str!("../../../locales/en/recorder.json"))
                .unwrap_or_default(),
            translated: if language.starts_with("es") {
                serde_json::from_str(include_str!("../../../locales/es/recorder.json"))
                    .unwrap_or_default()
            } else {
                BTreeMap::new()
            },
        }
    }
}

impl Strings {
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.translated
            .get(key)
            .or_else(|| self.english.get(key))
            .map(String::as_str)
            .unwrap_or(key)
    }
}

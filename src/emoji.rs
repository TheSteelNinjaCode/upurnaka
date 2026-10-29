//! Bundled, licensed Fluent Emoji data shared by the picker and RPC validation.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize, Serialize)]
pub struct Emoji {
    pub glyph: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub category: String,
    pub image: String,
}
pub fn catalog() -> &'static [Emoji] {
    static CATALOG: OnceLock<Vec<Emoji>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        rahti::serde_json::from_str(include_str!("../public/images/fluent-emoji/catalog.json"))
            .expect("bundled Fluent Emoji catalog")
    })
}
fn normalize(value: &str) -> String {
    value.replace('\u{fe0f}', "")
}
pub fn image_map() -> BTreeMap<String, String> {
    catalog()
        .iter()
        .map(|emoji| (normalize(&emoji.glyph), emoji.image.clone()))
        .collect()
}
pub fn supported(value: &str) -> bool {
    let value = normalize(value);
    !value.is_empty()
        && catalog()
            .iter()
            .any(|emoji| normalize(&emoji.glyph) == value)
}

use serde::Deserialize;
use std::path::PathBuf;

use super::places::expand_custom_place_path;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoToConfig {
    pub entries: Vec<GoToEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoToEntry {
    pub title: String,
    pub path: PathBuf,
}

#[derive(Deserialize, Default)]
pub(super) struct GoToConfigOverride {
    entries: Option<Vec<toml::Value>>,
}

impl GoToConfig {
    pub(super) fn from_override(overrides: GoToConfigOverride) -> Self {
        let entries = overrides
            .entries
            .unwrap_or_default()
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                GoToEntry::from_toml_value(value, &format!("go_to.entries[{index}]"))
            })
            .collect();
        Self { entries }
    }
}

impl GoToEntry {
    fn from_toml_value(value: &toml::Value, field_name: &str) -> Option<Self> {
        let table = match value {
            toml::Value::Table(t) => t,
            _ => {
                eprintln!(
                    "elio: {field_name}: expected a {{ title, path }} object; skipping entry"
                );
                return None;
            }
        };

        let title = table
            .get("title")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(title) = title else {
            eprintln!(
                "elio: {field_name}: go_to entries require a non-empty string title; skipping entry"
            );
            return None;
        };

        let path = table
            .get("path")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(path) = path else {
            eprintln!(
                "elio: {field_name}: go_to entries require a non-empty string path; skipping entry"
            );
            return None;
        };

        match expand_custom_place_path(path) {
            Ok(path) => Some(Self {
                title: title.to_string(),
                path,
            }),
            Err(error) => {
                eprintln!("elio: {field_name}: {error}; skipping entry");
                None
            }
        }
    }
}

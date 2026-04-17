use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub keybindings: BTreeMap<String, Vec<String>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut keybindings = BTreeMap::new();
        keybindings.insert("quit".into(), vec!["q".into()]);
        keybindings.insert("start".into(), vec!["enter".into()]);
        keybindings.insert("focus_next".into(), vec!["tab".into()]);
        keybindings.insert("focus_previous".into(), vec!["shift+tab".into()]);
        keybindings.insert("cycle_next".into(), vec!["right".into(), "l".into()]);
        keybindings.insert("cycle_previous".into(), vec!["left".into(), "h".into()]);
        keybindings.insert("restart".into(), vec!["tab enter".into()]);
        keybindings.insert("menu".into(), vec!["tab m".into()]);
        keybindings.insert("backspace".into(), vec!["backspace".into()]);

        Self {
            defaults: Defaults::default(),
            options: Options::default(),
            keybindings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_duration")]
    pub duration: u16,
    #[serde(default = "default_word_count")]
    pub word_count: u16,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            duration: default_duration(),
            word_count: default_word_count(),
            language: default_language(),
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default = "default_durations")]
    pub durations: Vec<u16>,
    #[serde(default = "default_word_counts")]
    pub word_counts: Vec<u16>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            durations: default_durations(),
            word_counts: default_word_counts(),
        }
    }
}

fn default_mode() -> String {
    "time".into()
}

fn default_duration() -> u16 {
    30
}

fn default_word_count() -> u16 {
    25
}

fn default_language() -> String {
    "english".into()
}

fn default_theme() -> String {
    "default".into()
}

fn default_durations() -> Vec<u16> {
    vec![15, 30, 60, 120]
}

fn default_word_counts() -> Vec<u16> {
    vec![10, 25, 50, 100]
}

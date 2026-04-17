use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tttui_core::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyStroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyStroke {
    pub fn from_token(token: &str) -> AppResult<Self> {
        let mut modifiers = KeyModifiers::empty();
        let mut key_name = None;

        for part in token.to_ascii_lowercase().split('+') {
            match part {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                value => key_name = Some(value),
            }
        }

        let code = match key_name {
            Some("enter") => KeyCode::Enter,
            Some("tab") => KeyCode::Tab,
            Some("backtab") | Some("shift-tab") => KeyCode::BackTab,
            Some("backspace") => KeyCode::Backspace,
            Some("esc") | Some("escape") => KeyCode::Esc,
            Some("left") => KeyCode::Left,
            Some("right") => KeyCode::Right,
            Some("up") => KeyCode::Up,
            Some("down") => KeyCode::Down,
            Some(value) if value.chars().count() == 1 => {
                KeyCode::Char(value.chars().next().unwrap())
            }
            Some(value) => {
                return Err(AppError::InvalidConfig(format!(
                    "unsupported key token `{value}`"
                )))
            }
            None => {
                return Err(AppError::InvalidConfig(format!(
                    "missing key name in `{token}`"
                )))
            }
        };

        Ok(Self { code, modifiers })
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        let event_code = match event.code {
            KeyCode::Char(value) => KeyCode::Char(value.to_ascii_lowercase()),
            other => other,
        };

        let expected_code = match self.code {
            KeyCode::Char(value) => KeyCode::Char(value.to_ascii_lowercase()),
            other => other,
        };

        expected_code == event_code && self.modifiers == event.modifiers
    }
}

#[derive(Debug, Clone)]
pub struct KeySequence(pub Vec<KeyStroke>);

impl KeySequence {
    pub fn parse(value: &str) -> AppResult<Self> {
        let sequence = value
            .split_whitespace()
            .map(KeyStroke::from_token)
            .collect::<AppResult<Vec<_>>>()?;

        if sequence.is_empty() {
            return Err(AppError::InvalidConfig("empty key sequence".into()));
        }

        Ok(Self(sequence))
    }
}

#[derive(Debug, Clone)]
pub struct KeyMap {
    bindings: BTreeMap<String, Vec<KeySequence>>,
}

impl KeyMap {
    pub fn from_config(config: &BTreeMap<String, Vec<String>>) -> AppResult<Self> {
        let bindings = config
            .iter()
            .map(|(action, values)| {
                let sequences = values
                    .iter()
                    .map(|value| KeySequence::parse(value))
                    .collect::<AppResult<Vec<_>>>()?;
                Ok((action.clone(), sequences))
            })
            .collect::<AppResult<BTreeMap<_, _>>>()?;

        Ok(Self { bindings })
    }

    pub fn sequences_for(&self, action: &str) -> &[KeySequence] {
        self.bindings
            .get(action)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct KeySequenceMatcher {
    pending: Vec<KeyStroke>,
}

impl KeySequenceMatcher {
    pub fn push(&mut self, event: &KeyEvent, keymap: &KeyMap) -> Option<String> {
        self.pending.push(KeyStroke {
            code: match event.code {
                KeyCode::Char(value) => KeyCode::Char(value.to_ascii_lowercase()),
                other => other,
            },
            modifiers: event.modifiers,
        });

        let mut has_prefix = false;

        for (action, sequences) in &keymap.bindings {
            for sequence in sequences {
                if sequence.0.starts_with(&self.pending) {
                    has_prefix = true;
                    if sequence.0.len() == self.pending.len() {
                        self.pending.clear();
                        return Some(action.clone());
                    }
                }
            }
        }

        if !has_prefix {
            self.pending.clear();
        }

        None
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multi_key_sequences() {
        let sequence = KeySequence::parse("tab enter").unwrap();
        assert_eq!(sequence.0.len(), 2);
    }

    #[test]
    fn matches_configured_sequence() {
        let mut config = BTreeMap::new();
        config.insert("restart".into(), vec!["tab enter".into()]);
        let keymap = KeyMap::from_config(&config).unwrap();
        let mut matcher = KeySequenceMatcher::default();

        assert_eq!(
            matcher.push(&KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()), &keymap),
            None
        );
        assert_eq!(
            matcher.push(
                &KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
                &keymap
            ),
            Some("restart".into())
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestMode {
    Time(u16),
    Words(u16),
    Punctuation(u16),
    Numbers(u16),
    Quote,
}

impl TestMode {
    pub fn key(&self) -> String {
        match self {
            Self::Time(duration) => format!("time_{duration}"),
            Self::Words(count) => format!("words_{count}"),
            Self::Punctuation(count) => format!("punctuation_{count}"),
            Self::Numbers(count) => format!("numbers_{count}"),
            Self::Quote => "quote".into(),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Time(duration) => format!("time {duration}"),
            Self::Words(count) => format!("words {count}"),
            Self::Punctuation(count) => format!("punctuation {count}"),
            Self::Numbers(count) => format!("numbers {count}"),
            Self::Quote => "quote".into(),
        }
    }
}

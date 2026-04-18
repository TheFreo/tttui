#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestMode {
    Time(u16),
    Words(u16),
    Quote,
}

impl TestMode {
    pub fn key(&self) -> String {
        match self {
            Self::Time(duration) => format!("time_{duration}"),
            Self::Words(count) => format!("words_{count}"),
            Self::Quote => "quote".into(),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Time(duration) => format!("time {duration}"),
            Self::Words(count) => format!("words {count}"),
            Self::Quote => "quote".into(),
        }
    }
}

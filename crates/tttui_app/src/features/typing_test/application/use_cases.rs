use rand::prelude::{IndexedRandom, Rng};
use rand::seq::SliceRandom;
use tttui_core::{AppError, AppResult};

use super::ports::ContentRepository;
use crate::features::typing_test::domain::session::TestSession;
use crate::features::typing_test::domain::test_mode::TestMode;

pub struct StartTypingTest<'a, R>
where
    R: ContentRepository,
{
    repository: &'a R,
}

impl<'a, R> StartTypingTest<'a, R>
where
    R: ContentRepository,
{
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, mode: TestMode, language: &str) -> AppResult<TestSession> {
        let target = match mode {
            TestMode::Time(_) => self.words_target(language, 240)?,
            TestMode::Words(count) => self.words_target(language, count as usize)?,
            TestMode::Punctuation(count) => self.punctuation_target(language, count as usize)?,
            TestMode::Numbers(count) => self.numbers_target(count as usize),
            TestMode::Quote => self.quote_target(language)?,
        };

        Ok(TestSession::new(mode, language.into(), target))
    }

    fn words_target(&self, language: &str, count: usize) -> AppResult<String> {
        let mut words = self.repository.words(language)?;
        if words.is_empty() {
            return Err(AppError::InvalidConfig(format!(
                "no words available for `{language}`"
            )));
        }

        let mut rng = rand::rng();
        words.shuffle(&mut rng);

        if words.len() < count {
            let source = words.clone();
            while words.len() < count {
                let mut next = source.clone();
                next.shuffle(&mut rng);
                words.extend(next);
            }
        }

        Ok(words.into_iter().take(count).collect::<Vec<_>>().join(" "))
    }

    fn quote_target(&self, language: &str) -> AppResult<String> {
        let quotes = self.repository.quotes(language)?;
        let mut rng = rand::rng();
        quotes
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| AppError::InvalidConfig(format!("no quotes available for `{language}`")))
    }

    fn punctuation_target(&self, language: &str, count: usize) -> AppResult<String> {
        let words = self
            .words_target(language, count)?
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        Ok(apply_punctuation(words))
    }

    fn numbers_target(&self, count: usize) -> String {
        let mut rng = rand::rng();
        (0..count)
            .map(|_| {
                let digits = rng.random_range(1..=4);
                let upper = 10_u32.pow(digits);
                rng.random_range(0..upper).to_string()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn apply_punctuation(mut words: Vec<String>) -> String {
    if words.is_empty() {
        return String::new();
    }

    let mut rng = rand::rng();
    let punctuation = [",", ".", "!", "?", ";", ":"];
    let len = words.len();

    for index in 0..len {
        if index == 0 {
            capitalize(&mut words[index]);
        }

        let should_add_mark = index == len - 1 || rng.random_ratio(1, 4);
        if should_add_mark {
            let mark = punctuation.choose(&mut rng).copied().unwrap_or(".");
            words[index].push_str(mark);

            if matches!(mark, "." | "!" | "?") && index + 1 < len {
                capitalize(&mut words[index + 1]);
            }
        }
    }

    words.join(" ")
}

fn capitalize(word: &mut String) {
    if let Some(first) = word.chars().next() {
        let replacement = first.to_uppercase().to_string();
        word.replace_range(0..first.len_utf8(), &replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punctuation_targets_capitalize_and_end_with_mark() {
        let target = apply_punctuation(vec!["hello".into(), "world".into()]);

        assert!(target.starts_with('H'));
        assert!(matches!(
            target.chars().last(),
            Some(',' | '.' | '!' | '?' | ';' | ':')
        ));
    }

    #[test]
    fn number_targets_preserve_requested_token_count() {
        struct NoContent;

        impl ContentRepository for NoContent {
            fn available_languages(&self) -> AppResult<Vec<String>> {
                Ok(Vec::new())
            }

            fn words(&self, _language: &str) -> AppResult<Vec<String>> {
                Ok(Vec::new())
            }

            fn quotes(&self, _language: &str) -> AppResult<Vec<String>> {
                Ok(Vec::new())
            }
        }

        let use_case = StartTypingTest::new(&NoContent);
        let session = use_case.execute(TestMode::Numbers(8), "english").unwrap();
        let target = session.target.iter().collect::<String>();

        assert_eq!(target.split_whitespace().count(), 8);
        assert!(target
            .split_whitespace()
            .all(|token| token.chars().all(|char| char.is_ascii_digit())));
    }
}

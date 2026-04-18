use rand::prelude::IndexedRandom;
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
}

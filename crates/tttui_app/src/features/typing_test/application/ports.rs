use tttui_core::AppResult;

pub trait ContentRepository {
    fn available_languages(&self) -> AppResult<Vec<String>>;
    fn words(&self, language: &str) -> AppResult<Vec<String>>;
    fn quotes(&self, language: &str) -> AppResult<Vec<String>>;
}

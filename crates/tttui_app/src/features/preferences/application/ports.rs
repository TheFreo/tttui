use std::collections::BTreeMap;

use crate::config::app_config::AppConfig;
use crate::features::preferences::domain::theme::ThemeDefinition;
use tttui_core::AppResult;

pub trait PreferencesRepository {
    fn load_config(&self) -> AppResult<AppConfig>;
    fn save_config(&self, config: &AppConfig) -> AppResult<()>;
    fn load_themes(&self) -> AppResult<BTreeMap<String, ThemeDefinition>>;
}

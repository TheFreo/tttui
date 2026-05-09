use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tttui_core::{AppError, AppResult};

use crate::config::app_config::AppConfig;
use crate::features::preferences::application::ports::PreferencesRepository;
use crate::features::preferences::domain::theme::ThemeDefinition;

pub struct FilePreferencesRepository {
    config_dir: PathBuf,
}

impl FilePreferencesRepository {
    pub fn new() -> AppResult<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::InvalidConfig("could not determine config directory".into()))?
            .join("tttui");

        fs::create_dir_all(config_dir.join("themes"))?;

        Ok(Self { config_dir })
    }

    fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    fn themes_dir(&self) -> PathBuf {
        self.config_dir.join("themes")
    }

    fn load_theme_file(path: &Path) -> AppResult<ThemeDefinition> {
        let raw = fs::read_to_string(path)?;
        toml::from_str(&raw).map_err(|error| AppError::ConfigParse(error.to_string()))
    }
}

impl PreferencesRepository for FilePreferencesRepository {
    fn load_config(&self) -> AppResult<AppConfig> {
        let path = self.config_path();
        if !path.exists() {
            let config = AppConfig::default();
            self.save_config(&config)?;
            return Ok(config);
        }

        let raw = fs::read_to_string(path)?;
        let mut config: AppConfig =
            toml::from_str(&raw).map_err(|error| AppError::ConfigParse(error.to_string()))?;
        config.merge_missing_keybindings();
        config.upgrade_legacy_default_keybindings();
        Ok(config)
    }

    fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        let serialized = toml::to_string_pretty(config)
            .map_err(|error| AppError::ConfigParse(error.to_string()))?;
        fs::write(self.config_path(), serialized)?;
        Ok(())
    }

    fn load_themes(&self) -> AppResult<BTreeMap<String, ThemeDefinition>> {
        let mut themes = built_in_themes()?;

        for entry in fs::read_dir(self.themes_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("toml") {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| AppError::InvalidConfig("invalid theme file name".into()))?
                .to_string();
            themes.insert(name, Self::load_theme_file(&path)?);
        }

        Ok(themes)
    }
}

fn built_in_themes() -> AppResult<BTreeMap<String, ThemeDefinition>> {
    [
        (
            "default",
            include_str!("../../../../assets/themes/default.toml"),
        ),
        ("nord", include_str!("../../../../assets/themes/nord.toml")),
        (
            "catppuccin-mocha",
            include_str!("../../../../assets/themes/catppuccin-mocha.toml"),
        ),
    ]
    .into_iter()
    .map(|(name, raw): (&str, &str)| {
        let theme: ThemeDefinition =
            toml::from_str(raw).map_err(|error| AppError::ConfigParse(error.to_string()))?;
        Ok((name.to_string(), theme))
    })
    .collect()
}

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use tttui_core::AppResult;

use crate::features::typing_test::application::ports::ContentRepository;

pub struct FileContentRepository {
    config_dir: PathBuf,
}

impl FileContentRepository {
    pub fn new(config_dir: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(config_dir.join("languages"))?;
        fs::create_dir_all(config_dir.join("quotes"))?;
        Ok(Self { config_dir })
    }

    fn user_dir(&self, item_type: &str) -> PathBuf {
        self.config_dir.join(item_type)
    }

    fn bundled_dir(item_type: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(item_type)
    }

    fn available_from_dir(path: &Path) -> AppResult<Vec<String>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut values = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("txt") {
                if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
                    values.push(stem.to_string());
                }
            }
        }
        Ok(values)
    }

    fn load_items(&self, item_type: &str, language: &str) -> AppResult<Vec<String>> {
        let user_path = self.user_dir(item_type).join(format!("{language}.txt"));
        let bundled_path = Self::bundled_dir(item_type).join(format!("{language}.txt"));
        let path = if user_path.exists() {
            user_path
        } else {
            bundled_path
        };
        let raw = fs::read_to_string(path)?;
        Ok(raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }
}

impl ContentRepository for FileContentRepository {
    fn available_languages(&self) -> AppResult<Vec<String>> {
        let mut values = BTreeSet::new();
        values.extend(Self::available_from_dir(&Self::bundled_dir("languages"))?);
        values.extend(Self::available_from_dir(&self.user_dir("languages"))?);
        Ok(values.into_iter().collect())
    }

    fn words(&self, language: &str) -> AppResult<Vec<String>> {
        self.load_items("languages", language)
    }

    fn quotes(&self, language: &str) -> AppResult<Vec<String>> {
        self.load_items("quotes", language)
    }
}

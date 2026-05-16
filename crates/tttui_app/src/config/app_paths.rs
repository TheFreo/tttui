use std::env;
use std::path::{Path, PathBuf};

use tttui_core::{AppError, AppResult};

pub fn config_dir() -> AppResult<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::InvalidConfig("could not determine home directory".into()))?;
    Ok(config_dir_from(
        &home,
        env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
    ))
}

fn config_dir_from(home: &Path, xdg_config_home: Option<PathBuf>) -> PathBuf {
    xdg_config_home
        .unwrap_or_else(|| home.join(".config"))
        .join("tttui")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_dot_config_under_home() {
        assert_eq!(
            config_dir_from(Path::new("/home/user"), None),
            PathBuf::from("/home/user/.config/tttui")
        );
    }

    #[test]
    fn honors_xdg_config_home() {
        assert_eq!(
            config_dir_from(
                Path::new("/home/user"),
                Some(PathBuf::from("/tmp/custom-config"))
            ),
            PathBuf::from("/tmp/custom-config/tttui")
        );
    }
}

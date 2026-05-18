use std::env;
use std::path::PathBuf;

use tttui_core::{AppError, AppResult};

pub fn config_dir() -> AppResult<PathBuf> {
    config_dir_from(
        dirs::home_dir(),
        dirs::config_dir(),
        env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        cfg!(windows),
    )
}

fn config_dir_from(
    home: Option<PathBuf>,
    platform_config_dir: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    windows: bool,
) -> AppResult<PathBuf> {
    if let Some(xdg_config_home) = xdg_config_home {
        return Ok(xdg_config_home.join("tttui"));
    }

    if windows {
        return platform_config_dir
            .map(|path| path.join("tttui"))
            .ok_or_else(|| AppError::InvalidConfig("could not determine config directory".into()));
    }

    home.map(|path| path.join(".config").join("tttui"))
        .ok_or_else(|| AppError::InvalidConfig("could not determine home directory".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_dot_config_under_home() {
        assert_eq!(
            config_dir_from(Some(PathBuf::from("/home/user")), None, None, false,).unwrap(),
            PathBuf::from("/home/user/.config/tttui"),
        );
    }

    #[test]
    fn honors_xdg_config_home() {
        assert_eq!(
            config_dir_from(
                Some(PathBuf::from("/home/user")),
                Some(PathBuf::from("/ignored/platform-config")),
                Some(PathBuf::from("/tmp/custom-config")),
                true,
            )
            .unwrap(),
            PathBuf::from("/tmp/custom-config/tttui"),
        );
    }

    #[test]
    fn windows_uses_platform_config_directory_by_default() {
        assert_eq!(
            config_dir_from(
                Some(PathBuf::from("C:/Users/user")),
                Some(PathBuf::from("C:/Users/user/AppData/Roaming")),
                None,
                true,
            )
            .unwrap(),
            PathBuf::from("C:/Users/user/AppData/Roaming/tttui"),
        );
    }
}

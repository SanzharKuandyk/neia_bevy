use std::fs;
use std::path::{Path, PathBuf};

pub mod paths;

use neia_shared::app::AppConfig;
use neia_shared::game::config::GameConfig;
pub use paths::{default_app_config_path, default_game_config_path, mods_dir};

pub struct LoadedConfigs {
    pub root: PathBuf,
    pub app: AppConfig,
    pub game: GameConfig,
}

impl LoadedConfigs {
    pub fn load_default() -> Result<Self, String> {
        let app_path = default_app_config_path();
        let game_path = default_game_config_path();
        let root = app_path
            .parent()
            .and_then(|dir| dir.parent())
            .ok_or_else(|| {
                format!(
                    "failed to resolve workspace root from {}",
                    app_path.display()
                )
            })?
            .to_path_buf();
        let app = load_toml::<AppConfig>(&app_path)?;
        let game = load_toml::<GameConfig>(&game_path)?;
        Ok(Self { root, app, game })
    }

    pub fn load_from_workspace_root(root: impl Into<PathBuf>) -> Result<Self, String> {
        let root = root.into();
        let app = load_toml::<AppConfig>(&root.join("configs/app.toml"))?;
        let game = load_toml::<GameConfig>(&root.join("configs/game.toml"))?;
        Ok(Self { root, app, game })
    }
}

fn load_toml<T>(path: &Path) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    toml::from_str(&source).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

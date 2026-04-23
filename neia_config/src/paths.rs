use std::env;
use std::path::PathBuf;

pub fn default_app_config_path() -> PathBuf {
    find_upwards("configs/app.toml").unwrap_or_else(|| PathBuf::from("configs/app.toml"))
}

pub fn default_game_config_path() -> PathBuf {
    find_upwards("configs/game.toml").unwrap_or_else(|| PathBuf::from("configs/game.toml"))
}

pub fn mods_dir() -> PathBuf {
    find_upwards("mods").unwrap_or_else(|| PathBuf::from("mods"))
}

fn find_upwards(relative: &str) -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;

    for dir in current_dir.ancestors() {
        let candidate = dir.join(relative);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);
    if let Some(manifest_dir) = manifest_dir {
        for dir in manifest_dir.ancestors() {
            let candidate = dir.join(relative);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    find_relative_to_exe(relative)
}

fn find_relative_to_exe(relative: &str) -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    for dir in exe_dir.ancestors() {
        let candidate = dir.join(relative);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedMod {
    pub root: PathBuf,
    pub manifest: ModManifest,
}

#[derive(Debug, Default, Clone)]
pub struct ModCollection {
    pub loaded: Vec<LoadedMod>,
}

impl ModCollection {
    pub fn scan(root: impl Into<PathBuf>) -> Result<Self, String> {
        let root = root.into();
        if !root.exists() {
            return Ok(Self::default());
        }

        let mut loaded = Vec::new();
        for entry in fs::read_dir(&root)
            .map_err(|err| format!("failed to read {}: {err}", root.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read mod entry: {err}"))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("mod.toml");
            if !manifest_path.exists() {
                continue;
            }

            let manifest = load_manifest(&manifest_path)?;
            loaded.push(LoadedMod {
                root: path,
                manifest,
            });
        }

        Ok(Self { loaded })
    }
}

fn load_manifest(path: &Path) -> Result<ModManifest, String> {
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    toml::from_str(&source).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

//! Non-secret settings kept in %APPDATA%\Keyplate\config.json.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub auto_lock_minutes: u32,
    pub clipboard_seconds: u32,
    /// 0 turns the practice reminder off.
    pub practice_days: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { auto_lock_minutes: 5, clipboard_seconds: 30, practice_days: 30 }
    }
}

impl Settings {
    pub fn clamped(self) -> Self {
        Self {
            auto_lock_minutes: self.auto_lock_minutes.clamp(1, 60),
            clipboard_seconds: self.clipboard_seconds.clamp(10, 120),
            practice_days: self.practice_days.min(365),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub vault_folder: Option<PathBuf>,
    pub device_id: String,
    pub settings: Settings,
    pub last_practice_ms: Option<i64>,
}

pub fn app_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("Keyplate")
}

pub fn cache_path() -> PathBuf {
    app_dir().join("cache.keyplate")
}

fn config_path() -> PathBuf {
    app_dir().join("config.json")
}

pub fn load() -> Config {
    let mut config: Config =
        fs::read(config_path()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    if config.device_id.is_empty() {
        config.device_id = vault_core::model::new_id().unwrap_or_else(|_| "device".into());
        save(&config);
    }
    config.settings = config.settings.clamped();
    config
}

pub fn save(config: &Config) {
    let _ = fs::create_dir_all(app_dir());
    if let Ok(json) = serde_json::to_vec_pretty(config) {
        let tmp = config_path().with_extension("tmp");
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(tmp, config_path());
        }
    }
}

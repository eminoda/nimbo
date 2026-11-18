use crate::models::NimboSettings;
use std::{fs, io::Write, path::PathBuf};
use tauri::{AppHandle, Manager};

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Result<NimboSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(NimboSettings::default());
    }

    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&contents).map_err(|error| format!("配置文件无法读取：{error}"))
}

pub fn save(app: &AppHandle, settings: &NimboSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let contents = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    let parent = path.parent().ok_or("配置目录无效")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    temp.write_all(&contents)
        .map_err(|error| error.to_string())?;
    temp.as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    temp.persist(&path)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

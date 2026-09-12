use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::sources::Wallpaper;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub source: String,
    pub url: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub file_type: String,
}


/*
 * ─────────────────────────────────────
 * HISTORY FILE
 * ─────────────────────────────────────
 */

pub fn history_path() -> Option<PathBuf> {
    let data_dir = dirs::data_dir()?;

    Some(
        data_dir
            .join("wally")
            .join("history.json"),
    )
}


/*
 * ─────────────────────────────────────
 * SOURCE
 * ─────────────────────────────────────
 */

fn source_from_wallpaper(
    wallpaper: &Wallpaper,
) -> &'static str {
    if wallpaper
        .url
        .contains("wallhaven.cc")
    {
        "wallhaven"
    } else if wallpaper
        .url
        .contains("konachan.com")
        || wallpaper
            .url
            .contains("konachan.net")
    {
        "konachan"
    } else {
        "unknown"
    }
}


/*
 * ─────────────────────────────────────
 * LOAD
 * ─────────────────────────────────────
 */

pub fn load() -> Vec<HistoryEntry> {
    let Some(path) = history_path() else {
        return Vec::new();
    };

    let Ok(content) =
        fs::read_to_string(&path)
    else {
        return Vec::new();
    };

    let mut entries: Vec<HistoryEntry> =
        serde_json::from_str(&content)
            .unwrap_or_default();


    /*
     * Поддержка старого history.json.
     *
     * Старые записи не имели source.
     * Для Wallhaven определяем его
     * по сохранённому URL.
     */

    for entry in &mut entries {
        if entry.source.is_empty() {
            if entry.url.contains("wallhaven.cc") {
                entry.source =
                    "wallhaven".to_string();
            } else if entry.url.contains("konachan.com")
                || entry.url.contains("konachan.net")
            {
                entry.source =
                    "konachan".to_string();
            } else {
                entry.source =
                    "unknown".to_string();
            }
        }
    }


    entries
}


/*
 * ─────────────────────────────────────
 * WRITE
 * ─────────────────────────────────────
 */

fn write_history(
    entries: &[HistoryEntry],
) -> Result<()> {
    let path =
        history_path()
            .context(
                "Failed to determine Wally data directory",
            )?;

    let parent =
        path.parent()
            .context(
                "Failed to determine Wally history directory",
            )?;

    fs::create_dir_all(parent)
        .context(
            "Failed to create Wally data directory",
        )?;

    let json =
        serde_json::to_string_pretty(entries)
            .context(
                "Failed to serialize Wally history",
            )?;

    fs::write(
        &path,
        format!("{json}\n"),
    )
    .context(
        "Failed to write Wally history",
    )?;

    Ok(())
}


/*
 * ─────────────────────────────────────
 * ADD
 * ─────────────────────────────────────
 */

pub fn add(
    wallpaper: &Wallpaper,
    path: &Path,
) -> Result<()> {
    let source =
        source_from_wallpaper(wallpaper);


    let mut entries =
        load();


    /*
     * Удаляем только запись этого
     * конкретного источника.
     *
     * Поэтому:
     *
     * wallhaven:123
     *
     * и
     *
     * konachan:123
     *
     * могут существовать одновременно.
     */

    entries.retain(|entry| {
        !(entry.id == wallpaper.id
            && entry.source == source)
    });


    let entry =
        HistoryEntry {
            id:
                wallpaper.id.clone(),

            source:
                source.to_string(),

            url:
                wallpaper.url.clone(),

            path:
                path.to_string_lossy()
                    .into_owned(),

            width:
                wallpaper.dimension_x,

            height:
                wallpaper.dimension_y,

            file_type:
                wallpaper.file_type.clone(),
        };


    entries.insert(
        0,
        entry,
    );


    write_history(&entries)
}


/*
 * ─────────────────────────────────────
 * CONTAINS
 * ─────────────────────────────────────
 */

pub fn contains(
    wallpaper: &Wallpaper,
) -> bool {
    let source =
        source_from_wallpaper(wallpaper);


    load()
        .iter()
        .any(|entry| {
            entry.id == wallpaper.id
                && entry.source == source
        })
}


/*
 * ─────────────────────────────────────
 * REMOVE
 * ─────────────────────────────────────
 */

pub fn remove(
    id: &str,
) -> Result<()> {
    let mut entries =
        load();


    entries.retain(|entry| {
        entry.id != id
    });


    write_history(&entries)
}


/*
 * ─────────────────────────────────────
 * CLEAR
 * ─────────────────────────────────────
 */

pub fn clear() -> Result<()> {
    write_history(&[])
}
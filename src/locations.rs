use std::fs;
use std::path::PathBuf;

const MAX_LOCATIONS: usize = 10;

fn locations_file() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;

    Some(
        config_dir
            .join("wally")
            .join("locations"),
    )
}

pub fn load() -> Vec<String> {
    let Some(path) = locations_file() else {
        return Vec::new();
    };

    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

pub fn save(location: &str) {
    let Some(path) = locations_file() else {
        return;
    };

    let Some(parent) = path.parent() else {
        return;
    };

    if fs::create_dir_all(parent).is_err() {
        return;
    }

    let mut locations = load();

    /*
     * Если такая папка уже есть —
     * убираем старую запись.
     */

    locations.retain(|item| item != location);

    /*
     * Новая папка становится первой.
     */

    locations.insert(0, location.to_string());

    /*
     * Ограничиваем количество сохранённых
     * местоположений.
     */

    locations.truncate(MAX_LOCATIONS);

    let content =
        locations.join("\n");

    let _ = fs::write(
        path,
        format!("{content}\n"),
    );
}

pub fn remove(location: &str) {
    let Some(path) = locations_file() else {
        return;
    };

    let mut locations = load();

    locations.retain(|item| item != location);

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let content =
        locations.join("\n");

    let _ = fs::write(
        path,
        if content.is_empty() {
            String::new()
        } else {
            format!("{content}\n")
        },
    );
}
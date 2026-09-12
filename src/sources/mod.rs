pub mod konachan;
pub mod wallhaven;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Wallpaper {
    pub id: String,
    pub url: String,
    pub purity: String,
    pub category: String,
    pub dimension_x: u32,
    pub dimension_y: u32,
    pub resolution: String,
    pub path: String,
    pub file_type: String,
    pub thumbs: Thumbnails,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Thumbnails {
    pub large: String,
    pub original: String,
    pub small: String,
}
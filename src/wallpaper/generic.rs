use std::path::Path;

use anyhow::Result;

pub async fn set_wallpaper(path: &Path) -> Result<()> {
    println!();
    println!("⚠ No automatic wallpaper backend was detected.");
    println!();
    println!("Wallpaper downloaded successfully:");
    println!("{}", path.display());
    println!();
    println!("You can set it manually using your desktop environment.");

    Ok(())
}
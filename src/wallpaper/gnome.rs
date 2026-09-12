use std::path::Path;

use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn set_wallpaper(path: &Path) -> Result<()> {
    println!("🟣 Using GNOME...");

    let path = path
        .canonicalize()
        .context("Failed to resolve wallpaper path")?;

    let uri = format!("file://{}", path.display());

    let status = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri",
            &uri,
        ])
        .status()
        .await
        .context("Failed to execute gsettings")?;

    if !status.success() {
        anyhow::bail!("GNOME failed to set wallpaper");
    }

    let status = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri-dark",
            &uri,
        ])
        .status()
        .await
        .context("Failed to set GNOME dark wallpaper")?;

    if !status.success() {
        anyhow::bail!("GNOME failed to set dark wallpaper");
    }

    println!("✓ Wallpaper set with GNOME");

    Ok(())
}
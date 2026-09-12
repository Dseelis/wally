use std::path::Path;

use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn set_wallpaper(path: &Path) -> Result<()> {
    if command_exists("swww").await {
        return set_with_swww(path).await;
    }

    if command_exists("swaybg").await {
        return set_with_swaybg(path).await;
    }

    anyhow::bail!(
        "Sway detected, but neither swww nor swaybg is installed."
    );
}

async fn set_with_swww(path: &Path) -> Result<()> {
    println!("🖼 Using swww...");

    let status = Command::new("swww")
        .arg("img")
        .arg(path)
        .status()
        .await
        .context("Failed to start swww")?;

    if !status.success() {
        anyhow::bail!("swww failed to set wallpaper");
    }

    println!("✓ Wallpaper set with swww");

    Ok(())
}

async fn set_with_swaybg(path: &Path) -> Result<()> {
    println!("🖼 Using swaybg...");

    let status = Command::new("swaybg")
        .arg("-i")
        .arg(path)
        .arg("-m")
        .arg("fill")
        .status()
        .await
        .context("Failed to start swaybg")?;

    if !status.success() {
        anyhow::bail!("swaybg failed to set wallpaper");
    }

    println!("✓ Wallpaper set with swaybg");

    Ok(())
}

async fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v '{command}'"))
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
}
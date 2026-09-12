use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn set_wallpaper(path: &Path) -> Result<()> {
    if command_exists("noctalia").await {
        return set_with_noctalia(path).await;
    }

    if command_exists("swww").await {
        return set_with_swww(path).await;
    }

    if command_exists("hyprctl").await
        && command_exists("hyprpaper").await
    {
        return set_with_hyprpaper(path).await;
    }

    anyhow::bail!(
        "Hyprland detected, but no supported wallpaper backend was found.\n\
         Install Noctalia, swww, or hyprpaper."
    );
}

async fn set_with_noctalia(path: &Path) -> Result<()> {
    println!("🌙 Using Noctalia...");

    let status = Command::new("noctalia")
        .arg("msg")
        .arg("wallpaper-set")
        .arg(path)
        .status()
        .await
        .context("Failed to start Noctalia")?;

    if !status.success() {
        anyhow::bail!(
            "Noctalia failed to set wallpaper (exit code: {:?})",
            status.code()
        );
    }

    println!("✓ Wallpaper set with Noctalia");

    Ok(())
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
        anyhow::bail!(
            "swww failed to set wallpaper (exit code: {:?})",
            status.code()
        );
    }

    println!("✓ Wallpaper set with swww");

    Ok(())
}

async fn set_with_hyprpaper(path: &Path) -> Result<()> {
    println!("🖼 Using hyprpaper...");

    let path = path
        .to_str()
        .context("Wallpaper path is not valid UTF-8")?;

    let preload = Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("preload")
        .arg(path)
        .status()
        .await
        .context("Failed to preload wallpaper with hyprctl")?;

    if !preload.success() {
        anyhow::bail!("hyprpaper failed to preload wallpaper");
    }

    let status = Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("wallpaper")
        .arg(format!(",{path}"))
        .status()
        .await
        .context("Failed to apply wallpaper with hyprctl")?;

    if !status.success() {
        anyhow::bail!("hyprpaper failed to apply wallpaper");
    }

    println!("✓ Wallpaper set with hyprpaper");

    Ok(())
}

async fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v '{command}'"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
}
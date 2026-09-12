use std::path::Path;

use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn set_wallpaper(path: &Path) -> Result<()> {
    println!("🔵 Using KDE Plasma...");

    let path = path
        .canonicalize()
        .context("Failed to resolve wallpaper path")?;

    let script = format!(
        r#"
        for (const screen of workspace.screens) {{
            const desktops = workspace.desktopsForScreen(screen);

            for (const desktop of desktops) {{
                desktop.wallpaperPlugin = "org.kde.image";

                desktop.currentConfigGroup = ["Wallpaper", "org.kde.image", "General"];
                desktop.writeConfig("Image", "file://{}");
            }}
        }}
        "#,
        path.display()
    );

    let status = Command::new("qdbus")
        .args([
            "org.kde.plasmashell",
            "/PlasmaShell",
            "org.kde.PlasmaShell.evaluateScript",
            &script,
        ])
        .status()
        .await
        .context("Failed to execute KDE DBus command")?;

    if !status.success() {
        anyhow::bail!("KDE failed to set wallpaper");
    }

    println!("✓ Wallpaper set with KDE");

    Ok(())
}
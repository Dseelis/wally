pub mod detect;
pub mod generic;
pub mod gnome;
pub mod hyprland;
pub mod kde;
pub mod sway;

use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum DesktopEnvironment {
    Hyprland,
    Kde,
    Gnome,
    Sway,
    Xfce,
    Unknown,
}

impl std::fmt::Display for DesktopEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Hyprland => "Hyprland",
            Self::Kde => "KDE Plasma",
            Self::Gnome => "GNOME",
            Self::Sway => "Sway",
            Self::Xfce => "XFCE",
            Self::Unknown => "Unknown",
        };

        write!(f, "{name}")
    }
}

pub async fn set_wallpaper(path: &Path) -> Result<()> {
    let environment = detect::detect();

    println!();
    println!("🖥 Desktop environment: {environment}");

    match environment {
        DesktopEnvironment::Hyprland => {
            hyprland::set_wallpaper(path).await
        }

        DesktopEnvironment::Kde => {
            kde::set_wallpaper(path).await
        }

        DesktopEnvironment::Gnome => {
            gnome::set_wallpaper(path).await
        }

        DesktopEnvironment::Sway => {
            sway::set_wallpaper(path).await
        }

        DesktopEnvironment::Xfce => {
            generic::set_wallpaper(path).await
        }

        DesktopEnvironment::Unknown => {
            generic::set_wallpaper(path).await
        }
    }
}
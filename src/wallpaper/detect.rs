use super::DesktopEnvironment;

pub fn detect() -> DesktopEnvironment {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return DesktopEnvironment::Hyprland;
    }

    if std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase()
        .contains("kde")
    {
        return DesktopEnvironment::Kde;
    }

    if std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase()
        .contains("gnome")
    {
        return DesktopEnvironment::Gnome;
    }

    if std::env::var_os("SWAYSOCK").is_some() {
        return DesktopEnvironment::Sway;
    }

    if std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase()
        .contains("xfce")
    {
        return DesktopEnvironment::Xfce;
    }

    DesktopEnvironment::Unknown
}
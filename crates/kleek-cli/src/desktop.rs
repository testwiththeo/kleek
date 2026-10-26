use anyhow::Result;
use std::process::Command;

pub fn apply_desktop_pointer_settings(accel_profile: &str) -> Result<()> {
    // 1. KDE Plasma detection and configuration (via kwriteconfig6 / kwriteconfig5)
    let kwriteconfig_cmd = if Command::new("which")
        .arg("kwriteconfig6")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("kwriteconfig6")
    } else if Command::new("which")
        .arg("kwriteconfig5")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("kwriteconfig5")
    } else {
        None
    };

    if let Some(cmd) = kwriteconfig_cmd {
        // 1 = Flat (1:1 tracking, no acceleration curve), 2 = Adaptive
        let profile_val = if accel_profile == "flat" { "1" } else { "2" };
        let _ = Command::new(cmd)
            .args([
                "--file",
                "kcminputrc",
                "--group",
                "Mouse",
                "--key",
                "AccelerationProfile",
                profile_val,
            ])
            .status();

        let _ = Command::new(cmd)
            .args([
                "--file",
                "kcminputrc",
                "--group",
                "Mouse",
                "--key",
                "PointerAcceleration",
                "0",
            ])
            .status();

        // Notify KDE input subsystem to reload mouse configuration live
        let _ = Command::new("kcminit").arg("mouse").status();
    }

    // 2. GNOME detection and configuration (via gsettings)
    if Command::new("which")
        .arg("gsettings")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let gsettings_val = if accel_profile == "flat" {
            "flat"
        } else {
            "adaptive"
        };
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.peripherals.mouse",
                "accel-profile",
                gsettings_val,
            ])
            .status();
    }

    // 3. Hyprland detection and configuration (via hyprctl)
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let _ = Command::new("hyprctl")
            .args(["keyword", "input:accel_profile", accel_profile])
            .status();
    }

    Ok(())
}

pub fn apply_desktop_natural_scroll(natural: bool) -> Result<()> {
    let bool_str = if natural { "true" } else { "false" };

    // 1. KDE Plasma detection and configuration
    let kwriteconfig_cmd = if Command::new("which")
        .arg("kwriteconfig6")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("kwriteconfig6")
    } else if Command::new("which")
        .arg("kwriteconfig5")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("kwriteconfig5")
    } else {
        None
    };

    if let Some(cmd) = kwriteconfig_cmd {
        let _ = Command::new(cmd)
            .args([
                "--file",
                "kcminputrc",
                "--group",
                "Mouse",
                "--key",
                "NaturalScroll",
                bool_str,
            ])
            .status();
        let _ = Command::new("kcminit").arg("mouse").status();
    }

    // 2. GNOME detection and configuration
    if Command::new("which")
        .arg("gsettings")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.peripherals.mouse",
                "natural-scroll",
                bool_str,
            ])
            .status();
    }

    // 3. Hyprland detection and configuration
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let _ = Command::new("hyprctl")
            .args(["keyword", "input:natural_scroll", bool_str])
            .status();
    }

    Ok(())
}

pub fn apply_desktop_touchpad(config: &crate::config::TouchpadConfig) -> Result<()> {
    let nat_str = if config.natural_scroll {
        "true"
    } else {
        "false"
    };
    let tap_str = if config.tap_to_click { "true" } else { "false" };
    let dwt_str = if config.disable_while_typing {
        "true"
    } else {
        "false"
    };

    // 1. KDE Plasma
    let kwriteconfig_cmd = if Command::new("which")
        .arg("kwriteconfig6")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("kwriteconfig6")
    } else if Command::new("which")
        .arg("kwriteconfig5")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("kwriteconfig5")
    } else {
        None
    };

    if let Some(cmd) = kwriteconfig_cmd {
        let _ = Command::new(cmd)
            .args([
                "--file",
                "kcminputrc",
                "--group",
                "Touchpad",
                "--key",
                "NaturalScroll",
                nat_str,
            ])
            .status();
        let _ = Command::new(cmd)
            .args([
                "--file",
                "kcminputrc",
                "--group",
                "Touchpad",
                "--key",
                "TapToClick",
                tap_str,
            ])
            .status();
        let _ = Command::new(cmd)
            .args([
                "--file",
                "kcminputrc",
                "--group",
                "Touchpad",
                "--key",
                "DisableWhileTyping",
                dwt_str,
            ])
            .status();
        let _ = Command::new("kcminit").arg("touchpad").status();
    }

    // 2. GNOME
    if Command::new("which")
        .arg("gsettings")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.peripherals.touchpad",
                "natural-scroll",
                nat_str,
            ])
            .status();
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.peripherals.touchpad",
                "tap-to-click",
                tap_str,
            ])
            .status();
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.peripherals.touchpad",
                "disable-while-typing",
                dwt_str,
            ])
            .status();
        if let Some(spd) = config.speed {
            let _ = Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.desktop.peripherals.touchpad",
                    "speed",
                    &spd.to_string(),
                ])
                .status();
        }
    }

    // 3. Hyprland
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let _ = Command::new("hyprctl")
            .args(["keyword", "input:touchpad:natural_scroll", nat_str])
            .status();
        let _ = Command::new("hyprctl")
            .args(["keyword", "input:touchpad:tap-to-click", tap_str])
            .status();
        let _ = Command::new("hyprctl")
            .args(["keyword", "input:touchpad:disable_while_typing", dwt_str])
            .status();
    }

    Ok(())
}

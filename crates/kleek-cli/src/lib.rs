pub mod config;
pub mod desktop;
pub mod update;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "kleekctl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    Dpi {
        value: Option<u16>,
    },
    Battery,
    Rate {
        hz: u16,
    },
    Scroll {
        #[arg(long)]
        smooth: Option<bool>,
        #[arg(long)]
        natural: Option<bool>,
    },
    /// Manage application exclusion list
    Apps {
        #[command(subcommand)]
        action: AppsAction,
    },
    /// Configure touchpad settings (independent from mouse)
    Touchpad {
        /// Toggle natural scrolling on touchpad (on/off)
        #[arg(long)]
        natural: Option<bool>,
        /// Toggle tap-to-click (on/off)
        #[arg(long)]
        tap_to_click: Option<bool>,
        /// Apply recommended macOS trackpad preset (natural on, tap-to-click on)
        #[arg(long)]
        mac_preset: bool,
    },
    /// Configure side button actions and gestures (Mac Mouse Fix style)
    Gestures {
        /// Set side button + scroll action (workspace, volume, tab, disabled)
        #[arg(long)]
        side_scroll: Option<String>,
    },
    Status,
    /// Check and install the latest updates from GitHub Releases
    Update {
        /// Check for updates without installing
        #[arg(long)]
        check: bool,
    },
    /// Manage mouse profiles and configuration templates
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum AppsAction {
    /// List all currently excluded applications
    List,
    /// Add an application to the exclusion list
    Add {
        /// Application name or WM_CLASS (e.g. blender, steam, factorio)
        name: String,
    },
    /// Remove an application from the exclusion list
    Remove {
        /// Application name or WM_CLASS to remove
        name: String,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum ConfigAction {
    /// Initialize config file (~/.config/kleek/config.toml) with a template
    Init {
        /// Template name (default, gaming, productivity)
        #[arg(default_value = "default")]
        template: String,
        /// Overwrite if config already exists
        #[arg(long)]
        force: bool,
    },
    /// List available configuration templates
    List,
    /// Apply an existing template directly to daemon
    Apply {
        /// Template name (default, gaming, productivity)
        template: String,
    },
}

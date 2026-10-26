pub mod config;
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

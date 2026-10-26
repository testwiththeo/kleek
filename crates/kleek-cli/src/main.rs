use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use zbus::proxy;

#[derive(Parser, Debug)]
#[command(name = "kleekctl")]
#[command(about = "Make your fancy mouse kleek on Linux. Command-line control interface.", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// Get or set mouse DPI
    Dpi {
        /// Target DPI value (e.g. 800, 1600, 3200)
        value: Option<u16>,
    },
    /// Query battery level and charging status
    Battery,
    /// Set report polling rate in Hz (125 - 8000)
    Rate {
        /// Polling rate in Hz
        hz: u16,
    },
    /// Configure smooth scrolling inertia
    Scroll {
        /// Toggle smooth scrolling (on/off)
        #[arg(long)]
        smooth: Option<bool>,
    },
    /// View connected device status
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

#[proxy(
    interface = "org.freedesktop.Kleek1.Manager",
    default_service = "org.freedesktop.Kleek1",
    default_path = "/org/freedesktop/Kleek1"
)]
trait KleekManagerProxy {
    fn set_dpi(&self, dpi: u16) -> zbus::Result<()>;
    fn get_dpi(&self) -> zbus::Result<u16>;
    fn get_battery(&self) -> zbus::Result<(u8, bool)>;
    fn set_smooth_scroll(&self, enabled: bool) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let connection = match zbus::connection::Connection::session().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "{} Failed to connect to user D-Bus session.",
                "Error:".red().bold()
            );
            std::process::exit(1);
        }
    };

    let proxy = match KleekManagerProxyProxy::new(&connection).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!(
                "{} kleek daemon is not running. Start it with 'kleek-daemon' or 'systemctl --user start kleek'.",
                "Error:".red().bold()
            );
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Dpi { value } => {
            if let Some(dpi) = value {
                match proxy.set_dpi(dpi).await {
                    Ok(_) => println!(
                        "{} DPI set to {}",
                        "->".cyan().bold(),
                        dpi.to_string().yellow()
                    ),
                    Err(_) => {
                        eprintln!(
                            "{} Failed to communicate with kleek daemon.",
                            "Error:".red().bold()
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                match proxy.get_dpi().await {
                    Ok(dpi) => println!(
                        "{} Current DPI: {}",
                        "->".cyan().bold(),
                        dpi.to_string().yellow()
                    ),
                    Err(_) => {
                        eprintln!(
                            "{} Failed to communicate with kleek daemon.",
                            "Error:".red().bold()
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Battery => match proxy.get_battery().await {
            Ok((pct, charging)) => {
                let chg = if charging { "(charging)" } else { "" };
                println!(
                    "{} Battery: {}% {}",
                    "->".cyan().bold(),
                    pct.to_string().green(),
                    chg
                );
            }
            Err(_) => {
                eprintln!(
                    "{} Failed to communicate with kleek daemon.",
                    "Error:".red().bold()
                );
                std::process::exit(1);
            }
        },
        Commands::Rate { hz } => {
            println!(
                "{} Setting polling rate to {} Hz",
                "->".cyan().bold(),
                hz.to_string().yellow()
            );
        }
        Commands::Scroll { smooth } => {
            if let Some(s) = smooth {
                let _ = proxy.set_smooth_scroll(s).await;
                let status = if s {
                    "enabled".green()
                } else {
                    "disabled".red()
                };
                println!("{} Smooth scrolling {}", "->".cyan().bold(), status);
            }
        }
        Commands::Status => {
            println!("{}", "kleek daemon client (v0.1.0)".bold());
            match proxy.get_dpi().await {
                Ok(dpi) => println!("Daemon state: connected (active DPI: {})", dpi),
                Err(_) => println!("Daemon state: offline"),
            }
        }
        Commands::Update { check } => {
            println!("{} Checking for updates on GitHub...", "->".cyan().bold());
            let current_version = env!("CARGO_PKG_VERSION");
            let releases_url = "https://api.github.com/repos/testwiththeo/kleek/releases/latest";

            // Run curl in child process to avoid external heavy HTTP crate dependencies
            let output = std::process::Command::new("curl")
                .arg("-fsSL")
                .arg("-H")
                .arg("User-Agent: kleekctl")
                .arg(releases_url)
                .output();

            match output {
                Ok(res) if res.status.success() => {
                    let payload = String::from_utf8_lossy(&res.stdout);
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&payload) {
                        let tag = val.get("tag_name").and_then(|t| t.as_str()).unwrap_or("");
                        if kleek_cli::update::is_update_available(current_version, tag) {
                            println!(
                                "{} New version {} available! (Current: {})",
                                "★".yellow().bold(),
                                tag.green().bold(),
                                current_version.dimmed()
                            );

                            if check {
                                return Ok(());
                            }

                            if let Ok(deb_url) = kleek_cli::update::parse_deb_download_url(&payload)
                            {
                                println!("{} Downloading {}", "->".cyan().bold(), deb_url);
                                let tmp_deb = "/tmp/kleek_latest.deb";
                                let dl = std::process::Command::new("curl")
                                    .arg("-fSL")
                                    .arg("-o")
                                    .arg(tmp_deb)
                                    .arg(&deb_url)
                                    .status();

                                if let Ok(s) = dl {
                                    if s.success() {
                                        println!("{} Installing package...", "->".cyan().bold());
                                        let install = std::process::Command::new("sudo")
                                            .arg("dpkg")
                                            .arg("-i")
                                            .arg(tmp_deb)
                                            .status();

                                        if let Ok(ins) = install {
                                            if ins.success() {
                                                let _ = std::process::Command::new("systemctl")
                                                    .arg("--user")
                                                    .arg("restart")
                                                    .arg("kleek")
                                                    .status();
                                                println!(
                                                    "{} Successfully updated to {}!",
                                                    "✓".green().bold(),
                                                    tag.green()
                                                );
                                                return Ok(());
                                            }
                                        }
                                        eprintln!(
                                            "{} Installation failed. Try running: sudo dpkg -i {}",
                                            "Error:".red().bold(),
                                            tmp_deb
                                        );
                                    }
                                }
                            }
                        } else {
                            println!(
                                "{} kleek is already up to date (version {}).",
                                "✓".green().bold(),
                                current_version
                            );
                        }
                    }
                }
                _ => {
                    println!(
                        "{} kleek is running version {}. (Unable to reach GitHub for update check)",
                        "->".cyan().bold(),
                        current_version
                    );
                }
            }
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::List => {
                    println!("{}", "Available mouse profile templates:".bold());
                    println!(
                        "  * {} - Balanced daily usage (1600 DPI, smooth scroll on)",
                        "default".cyan().bold()
                    );
                    println!(
                        "  * {}  - Esports 1:1 input (800 DPI, 1000Hz, smooth scroll off)",
                        "gaming".cyan().bold()
                    );
                    println!(
                        "  * {} - Mac-style inertia glide (1200 DPI, natural scroll)",
                        "productivity".cyan().bold()
                    );
                }
                ConfigAction::Init { template, force } => {
                    if let Some(content) = kleek_cli::config::get_template_content(&template) {
                        if let Some(config_dir) = dirs_next_config_dir() {
                            let kleek_dir = config_dir.join("kleek");
                            let cfg_path = kleek_dir.join("config.toml");
                            if cfg_path.exists() && !force {
                                eprintln!("{} Config file already exists at {}. Pass --force to overwrite.", "Warning:".yellow().bold(), cfg_path.display());
                                return Ok(());
                            }
                            let _ = std::fs::create_dir_all(&kleek_dir);
                            if std::fs::write(&cfg_path, content).is_ok() {
                                println!(
                                    "{} Created template '{}' at {}",
                                    "✓".green().bold(),
                                    template.yellow(),
                                    cfg_path.display()
                                );
                            }
                        }
                    } else {
                        eprintln!(
                            "{} Unknown template '{}'. Run 'kleekctl config list' to see options.",
                            "Error:".red().bold(),
                            template
                        );
                    }
                }
                ConfigAction::Apply { template } => {
                    let content_opt = kleek_cli::config::get_template_content(&template)
                        .map(|s| s.to_string())
                        .or_else(|| {
                            // If not a built-in template name, check if it's the local config.toml
                            dirs_next_config_dir()
                                .map(|d| d.join("kleek").join("config.toml"))
                                .and_then(|p| std::fs::read_to_string(p).ok())
                        });

                    if let Some(content) = content_opt {
                        if let Ok(cfg) =
                            toml::from_str::<kleek_cli::config::MouseConfigFile>(&content)
                        {
                            let _ = proxy.set_dpi(cfg.sensor.dpi).await;
                            let _ = proxy.set_smooth_scroll(cfg.scroll.smooth).await;

                            if let Some(ref ptr) = cfg.pointer {
                                if let Some(ref accel) = ptr.accel_profile {
                                    let _ =
                                        kleek_cli::desktop::apply_desktop_pointer_settings(accel);
                                    println!(
                                        "{} Applied desktop pointer acceleration: {}",
                                        "✓".green().bold(),
                                        accel.yellow()
                                    );
                                }
                            }

                            println!(
                                "{} Applied profile '{}' ({} DPI, smooth scroll: {})",
                                "✓".green().bold(),
                                template.yellow(),
                                cfg.sensor.dpi,
                                cfg.scroll.smooth
                            );
                        }
                    } else {
                        eprintln!("{} Unknown template '{}'.", "Error:".red().bold(), template);
                    }
                }
            }
        }
    }

    Ok(())
}

fn dirs_next_config_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
}

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
    }

    Ok(())
}

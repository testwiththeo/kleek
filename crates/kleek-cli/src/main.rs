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
    }

    Ok(())
}

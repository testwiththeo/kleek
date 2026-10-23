use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser, Debug)]
#[command(name = "kleekctl")]
#[command(about = "Make your fancy mouse kleek on Linux. Command-line control interface.", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dpi { value } => {
            if let Some(dpi) = value {
                println!(
                    "{} Setting DPI to {}",
                    "->".cyan().bold(),
                    dpi.to_string().yellow()
                );
            } else {
                println!("{} Querying current DPI...", "->".cyan().bold());
            }
        }
        Commands::Battery => {
            println!("{} Querying battery status...", "->".cyan().bold());
        }
        Commands::Rate { hz } => {
            println!(
                "{} Setting polling rate to {} Hz",
                "->".cyan().bold(),
                hz.to_string().yellow()
            );
        }
        Commands::Scroll { smooth } => {
            if let Some(s) = smooth {
                let status = if s {
                    "enabled".green()
                } else {
                    "disabled".red()
                };
                println!("{} Smooth scrolling {}", "->".cyan().bold(), status);
            } else {
                println!("{} Querying scroll config...", "->".cyan().bold());
            }
        }
        Commands::Status => {
            println!("{}", "kleek daemon client (v0.1.0)".bold());
            println!("No active D-Bus session found (daemon offline).");
        }
    }

    Ok(())
}

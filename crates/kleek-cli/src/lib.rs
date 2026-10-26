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
}

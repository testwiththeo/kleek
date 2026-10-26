use clap::Parser;
use kleek_cli::{Cli, Commands};

#[test]
fn test_cli_parsing_subcommands() {
    let args = vec!["kleekctl", "dpi", "1600"];
    let parsed = Cli::try_parse_from(args).unwrap();
    assert_eq!(parsed.command, Commands::Dpi { value: Some(1600) });

    let args_batt = vec!["kleekctl", "battery"];
    let parsed_batt = Cli::try_parse_from(args_batt).unwrap();
    assert_eq!(parsed_batt.command, Commands::Battery);

    let args_update = vec!["kleekctl", "update", "--check"];
    let parsed_update = Cli::try_parse_from(args_update).unwrap();
    assert_eq!(parsed_update.command, Commands::Update { check: true });
}

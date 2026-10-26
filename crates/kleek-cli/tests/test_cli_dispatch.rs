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

    let args_cfg = vec!["kleekctl", "config", "init", "gaming"];
    let parsed_cfg = Cli::try_parse_from(args_cfg).unwrap();
    assert_eq!(
        parsed_cfg.command,
        Commands::Config {
            action: kleek_cli::ConfigAction::Init {
                template: "gaming".into(),
                force: false,
            }
        }
    );

    let args_scroll_nat = vec!["kleekctl", "scroll", "--natural", "true"];
    let parsed_scroll_nat = Cli::try_parse_from(args_scroll_nat).unwrap();
    assert_eq!(
        parsed_scroll_nat.command,
        Commands::Scroll {
            smooth: None,
            natural: Some(true),
        }
    );

    let args_apps_add = vec!["kleekctl", "apps", "add", "blender"];
    let parsed_apps_add = Cli::try_parse_from(args_apps_add).unwrap();
    assert_eq!(
        parsed_apps_add.command,
        Commands::Apps {
            action: kleek_cli::AppsAction::Add {
                name: "blender".into()
            }
        }
    );

    let args_tp = vec!["kleekctl", "touchpad", "--mac-preset"];
    let parsed_tp = Cli::try_parse_from(args_tp).unwrap();
    assert_eq!(
        parsed_tp.command,
        Commands::Touchpad {
            natural: None,
            tap_to_click: None,
            mac_preset: true,
        }
    );
}

use kleek_cli::update::{is_update_available, parse_deb_download_url};

#[test]
fn test_version_comparison() {
    assert!(is_update_available("0.1.0", "v0.2.0"));
    assert!(!is_update_available("0.2.0", "v0.1.0"));
    assert!(!is_update_available("0.1.0", "0.1.0"));
}

#[test]
fn test_parse_deb_download_url() {
    let mock_json = r#"{
        "tag_name": "v0.2.0",
        "assets": [
            {
                "name": "kleek_0.2.0-1_amd64.deb",
                "browser_download_url": "https://github.com/testwiththeo/kleek/releases/download/v0.2.0/kleek_0.2.0-1_amd64.deb"
            }
        ]
    }"#;

    let url = parse_deb_download_url(mock_json).unwrap();
    assert_eq!(
        url,
        "https://github.com/testwiththeo/kleek/releases/download/v0.2.0/kleek_0.2.0-1_amd64.deb"
    );
}

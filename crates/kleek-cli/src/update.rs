use anyhow::{anyhow, Result};
use serde_json::Value;

pub fn is_update_available(current: &str, remote_tag: &str) -> bool {
    let current_clean = current.trim().trim_start_matches('v');
    let remote_clean = remote_tag.trim().trim_start_matches('v');

    let curr_parts: Vec<u32> = current_clean
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let remote_parts: Vec<u32> = remote_clean
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if curr_parts.len() >= 3 && remote_parts.len() >= 3 {
        remote_parts > curr_parts
    } else {
        remote_clean != current_clean
    }
}

pub fn parse_deb_download_url(json_payload: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json_payload)?;
    let assets = value
        .get("assets")
        .and_then(|a| a.as_array())
        .ok_or_else(|| anyhow!("No release assets found in payload"))?;

    for asset in assets {
        if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
            if name.ends_with(".deb") {
                if let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                    return Ok(url.to_string());
                }
            }
        }
    }

    Err(anyhow!("No .deb package asset found in latest release"))
}

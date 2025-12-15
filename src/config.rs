//! Configuration loading and validation

use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::path::Path;

use crate::data::Configuration;

const STANDARD_CONFIG_FILE: &str = "openapi-merge.json";

/// Load configuration from file
pub fn load_configuration(config_path: &std::path::Path) -> Result<Configuration> {
    let config_file = if config_path.as_os_str().is_empty() {
        Path::new(STANDARD_CONFIG_FILE)
    } else {
        config_path
    };

    let raw_data = fs::read_to_string(config_file).with_context(|| {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        format!(
            "Could not find or read '{}' in the current directory: {}",
            config_file.display(),
            cwd
        )
    })?;

    validate_configuration(&raw_data)
}

/// Validate and parse configuration
fn validate_configuration(raw_data: &str) -> Result<Configuration> {
    // Parse as JSON or YAML
    let data: serde_json::Value = if let Ok(json) = serde_json::from_str(raw_data) {
        json
    } else if let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(raw_data) {
        yaml
    } else {
        anyhow::bail!("Configuration file must be valid JSON or YAML");
    };

    // TODO: Add JSON schema validation once we have the schema
    // For now, just deserialize directly
    let config: Configuration =
        serde_json::from_value(data).context("Failed to parse configuration")?;

    // Basic validation
    if config.inputs.is_empty() {
        anyhow::bail!("Configuration must have at least one input");
    }

    Ok(config)
}

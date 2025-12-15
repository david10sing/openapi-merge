//! File loading utilities for OpenAPI files

use anyhow::{Context, Result};
use openapiv3::OpenAPI;
use serde_json;
use serde_yaml;
use std::fs;
use std::path::Path;
use url::Url;

use crate::data::ConfigurationInput;

/// Load an OpenAPI file from a configuration input
pub fn load_oas_for_input(
    base_path: &Path,
    input: &ConfigurationInput,
    input_index: usize,
    logger: &mut dyn FnMut(&str),
) -> Result<OpenAPI> {
    match input {
        ConfigurationInput::FromFile(file_input) => {
            let full_path = base_path.join(&file_input.input_file);
            logger(&format!("## Loading input {}: {}", input_index, full_path.display()));
            load_from_file(&full_path)
        }
        ConfigurationInput::FromUrl(url_input) => {
            logger(&format!("## Loading input {} from URL: {}", input_index, url_input.input_url));
            load_from_url(&url_input.input_url)
        }
    }
}

/// Load OpenAPI file from local filesystem
pub fn load_from_file(file_path: &Path) -> Result<OpenAPI> {
    let contents = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    
    parse_yaml_or_json(&contents)
}

/// Load OpenAPI file from URL
pub fn load_from_url(url_str: &str) -> Result<OpenAPI> {
    let url = Url::parse(url_str)
        .with_context(|| format!("Invalid URL: {}", url_str))?;
    
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()
        .with_context(|| format!("Failed to fetch URL: {}", url_str))?;
    
    let contents = response.text()
        .with_context(|| format!("Failed to read response from URL: {}", url_str))?;
    
    parse_yaml_or_json(&contents)
}

/// Parse YAML or JSON content into OpenAPI
fn parse_yaml_or_json(contents: &str) -> Result<OpenAPI> {
    // Try JSON first
    if let Ok(openapi) = serde_json::from_str::<OpenAPI>(contents) {
        return Ok(openapi);
    }

    // Try YAML
    if let Ok(openapi) = serde_yaml::from_str::<OpenAPI>(contents) {
        return Ok(openapi);
    }

    // If both fail, try parsing as generic value first
    let json_value: Result<serde_json::Value, _> = serde_json::from_str(contents);
    let yaml_value: Result<serde_yaml::Value, _> = serde_yaml::from_str(contents);

    match (json_value, yaml_value) {
        (Ok(val), _) => {
            serde_json::from_value(val)
                .context("Failed to parse as OpenAPI from JSON")
        }
        (_, Ok(val)) => {
            // Convert YAML value to JSON value for deserialization
            let json_val = serde_json::to_value(&val)
                .context("Failed to convert YAML to JSON")?;
            serde_json::from_value(json_val)
                .context("Failed to parse as OpenAPI from YAML")
        }
        (Err(json_err), Err(yaml_err)) => {
            anyhow::bail!(
                "Failed to parse the input as either JSON or YAML.\n\nJSON Error: {}\n\nYAML Error: {}",
                json_err,
                yaml_err
            )
        }
    }
}


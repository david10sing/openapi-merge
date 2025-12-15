//! Extension merging logic

use crate::data::MergeInput;
use openapiv3::OpenAPI;
use serde_json::Value as JsonValue;

/// Merge x-extension fields from all inputs
pub fn merge_extensions(output: &mut OpenAPI, inputs: &MergeInput) {
    // Extract extensions from output
    let mut extensions = extract_extensions(output);

    // Extract and merge extensions from all inputs
    for input in inputs {
        let input_extensions = extract_extensions(&input.oas);
        for (key, value) in input_extensions {
            if !extensions.contains_key(&key) {
                extensions.insert(key, value);
            }
        }
    }

    // Apply extensions back to output
    // Note: openapiv3 crate may not support extensions directly,
    // so we may need to serialize/deserialize to add them
    // For now, this is a placeholder
}

fn extract_extensions(oas: &OpenAPI) -> std::collections::HashMap<String, JsonValue> {
    let mut result = std::collections::HashMap::new();
    
    // Convert OpenAPI to JSON value to extract extensions
    if let Ok(json_value) = serde_json::to_value(oas) {
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key.starts_with("x-") {
                    result.insert(key.clone(), value.clone());
                }
            }
        }
    }

    result
}


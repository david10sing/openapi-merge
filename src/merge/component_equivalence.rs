//! Component equivalence checking for deduplication

use openapiv3::*;
use serde_json::Value as JsonValue;

/// Check if two schema references are deeply equal
/// This is a simplified version - full implementation would need reference resolution
pub fn deep_equality_schema(
    x: &ReferenceOr<Schema>,
    y: &ReferenceOr<Schema>,
) -> bool {
    // For now, use JSON equality as a proxy
    // A full implementation would need to resolve references and compare recursively
    let x_json = serde_json::to_value(x).unwrap_or(JsonValue::Null);
    let y_json = serde_json::to_value(y).unwrap_or(JsonValue::Null);
    x_json == y_json
}

/// Check if two components are equal by comparing their JSON representation
pub fn components_equal<T>(x: &T, y: &T) -> bool
where
    T: serde::Serialize,
{
    let x_json = serde_json::to_value(x).unwrap_or(JsonValue::Null);
    let y_json = serde_json::to_value(y).unwrap_or(JsonValue::Null);
    x_json == y_json
}


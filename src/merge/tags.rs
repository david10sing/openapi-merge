//! Tag merging logic

use crate::data::MergeInput;
use openapiv3::Tag;

/// Merge tags from all inputs
pub fn merge_tags(inputs: &MergeInput) -> Option<Vec<Tag>> {
    let mut result = Vec::new();
    let mut seen_tags = std::collections::HashSet::new();

    for input in inputs {
        let exclude_tags: Vec<String> = input
            .operation_selection
            .as_ref()
            .and_then(|os| os.exclude_tags.as_ref())
            .cloned()
            .unwrap_or_default();

        // tags is a Vec<Tag>, iterate directly
        for tag in &input.oas.tags {
            if !exclude_tags.contains(&tag.name) {
                if !seen_tags.contains(&tag.name) {
                    seen_tags.insert(tag.name.clone());
                    result.push(tag.clone());
                }
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

//! Operation selection logic

use crate::data::OperationSelection;
use openapiv3::{OpenAPI, Operation, ReferenceOr};

/// Run operation selection filtering
pub fn run_operation_selection(
    mut oas: OpenAPI,
    operation_selection: Option<&OperationSelection>,
) -> OpenAPI {
    if operation_selection.is_none() {
        return oas;
    }

    let selection = operation_selection.unwrap();
    let include_tags = selection.include_tags.as_deref().unwrap_or(&[]);
    let exclude_tags = selection.exclude_tags.as_deref().unwrap_or(&[]);

    // First include operations with matching tags
    if !include_tags.is_empty() {
        oas = include_operations_that_have_tags(oas, include_tags);
    }

    // Then exclude operations with matching tags
    if !exclude_tags.is_empty() {
        oas = drop_operations_that_have_tags(oas, exclude_tags);
    }

    oas
}

fn operation_contains_any_tag(operation: &Operation, tags: &[String]) -> bool {
    operation.tags.iter().any(|tag| tags.contains(tag))
}

fn drop_operations_that_have_tags(mut oas: OpenAPI, excluded_tags: &[String]) -> OpenAPI {
    if excluded_tags.is_empty() {
        return oas;
    }

    for path_item in oas.paths.paths.values_mut() {
        match path_item {
            ReferenceOr::Item(item) => {
                if let Some(op) = item.get.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.get = None;
                    }
                }
                if let Some(op) = item.put.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.put = None;
                    }
                }
                if let Some(op) = item.post.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.post = None;
                    }
                }
                if let Some(op) = item.delete.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.delete = None;
                    }
                }
                if let Some(op) = item.options.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.options = None;
                    }
                }
                if let Some(op) = item.head.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.head = None;
                    }
                }
                if let Some(op) = item.patch.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.patch = None;
                    }
                }
                if let Some(op) = item.trace.as_mut() {
                    if operation_contains_any_tag(op, excluded_tags) {
                        item.trace = None;
                    }
                }
            }
            ReferenceOr::Reference { .. } => {
                // References are kept as-is
            }
        }
    }

    oas
}

fn include_operations_that_have_tags(mut oas: OpenAPI, include_tags: &[String]) -> OpenAPI {
    if include_tags.is_empty() {
        return oas;
    }

    for path_item in oas.paths.paths.values_mut() {
        match path_item {
            ReferenceOr::Item(item) => {
                if let Some(op) = item.get.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.get = None;
                    }
                }
                if let Some(op) = item.put.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.put = None;
                    }
                }
                if let Some(op) = item.post.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.post = None;
                    }
                }
                if let Some(op) = item.delete.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.delete = None;
                    }
                }
                if let Some(op) = item.options.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.options = None;
                    }
                }
                if let Some(op) = item.head.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.head = None;
                    }
                }
                if let Some(op) = item.patch.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.patch = None;
                    }
                }
                if let Some(op) = item.trace.as_mut() {
                    if !operation_contains_any_tag(op, include_tags) {
                        item.trace = None;
                    }
                }
            }
            ReferenceOr::Reference { .. } => {
                // References are kept as-is
            }
        }
    }

    oas
}

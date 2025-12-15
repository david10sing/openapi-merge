//! Paths and components merging logic

use crate::data::{ErrorMergeResult, ErrorType, MergeInput, PathModification};
use crate::merge::component_equivalence::components_equal;
use crate::merge::dispute::{apply_dispute, get_dispute, DisputeStatus};
use crate::merge::operation_selection::run_operation_selection;
use crate::merge::reference_walker::walk_all_references;
use indexmap::IndexMap;
use openapiv3::*;

/// Result of merging paths and components
pub type PathAndComponents = (Paths, Components);

/// Merge paths and components from all inputs
pub fn merge_paths_and_components(
    inputs: &MergeInput,
) -> Result<PathAndComponents, ErrorMergeResult> {
    let mut seen_operation_ids = std::collections::HashSet::new();
    let mut result_paths = Paths::default();
    let mut result_components = Components::default();

    for (input_index, input) in inputs.iter().enumerate() {
        let dispute = get_dispute(input);

        // Apply operation selection - clone the OAS first
        let oas_json = serde_json::to_value(&input.oas).map_err(|e| ErrorMergeResult {
            error_type: ErrorType::NoInputs,
            message: format!("Failed to serialize OAS: {}", e),
        })?;
        let mut oas: OpenAPI = serde_json::from_value(oas_json).map_err(|e| ErrorMergeResult {
            error_type: ErrorType::NoInputs,
            message: format!("Failed to deserialize OAS: {}", e),
        })?;
        oas = run_operation_selection(oas, input.operation_selection.as_ref());

        // Drop path items with no operations
        oas = drop_path_items_with_no_operations(oas);

        // Reference modification map
        let mut reference_modification: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // Process components first to build reference modification map
        if let Some(components) = &oas.components {
            // Process schemas
            if !components.schemas.is_empty() {
                process_schemas(
                    &mut result_components.schemas,
                    &components.schemas,
                    &dispute,
                    &mut reference_modification,
                )?;
            }

            // Process responses
            if !components.responses.is_empty() {
                process_responses(
                    &mut result_components.responses,
                    &components.responses,
                    &dispute,
                    &mut reference_modification,
                )?;
            }

            // Process parameters
            if !components.parameters.is_empty() {
                process_parameters(
                    &mut result_components.parameters,
                    &components.parameters,
                    &dispute,
                    &mut reference_modification,
                )?;
            }

            // Process examples
            if !components.examples.is_empty() {
                process_components_with_prefix(
                    &mut result_components.examples,
                    &components.examples,
                    &dispute,
                    &mut reference_modification,
                    "examples",
                )?;
            }

            // Process request bodies
            if !components.request_bodies.is_empty() {
                process_components_with_prefix(
                    &mut result_components.request_bodies,
                    &components.request_bodies,
                    &dispute,
                    &mut reference_modification,
                    "requestBodies",
                )?;
            }

            // Process headers
            if !components.headers.is_empty() {
                process_components_with_prefix(
                    &mut result_components.headers,
                    &components.headers,
                    &dispute,
                    &mut reference_modification,
                    "headers",
                )?;
            }

            // Process links
            if !components.links.is_empty() {
                process_components_with_prefix(
                    &mut result_components.links,
                    &components.links,
                    &dispute,
                    &mut reference_modification,
                    "links",
                )?;
            }

            // Process callbacks
            if !components.callbacks.is_empty() {
                process_components_with_prefix(
                    &mut result_components.callbacks,
                    &components.callbacks,
                    &dispute,
                    &mut reference_modification,
                    "callbacks",
                )?;
            }

            // Security schemes - just take from first file that has any
            if result_components.security_schemes.is_empty()
                && !components.security_schemes.is_empty()
            {
                result_components.security_schemes = components.security_schemes.clone();
            }
        }

        // Process paths
        let path_modification = input.path_modification.as_ref();
        for (original_path, path_item) in oas.paths.iter() {
            let new_path = apply_path_modification(original_path, path_modification);

            if original_path != &new_path {
                reference_modification.insert(
                    format!("#/paths/{}", original_path),
                    format!("#/paths/{}", new_path),
                );
            }

            // Check for duplicate paths
            if result_paths.paths.contains_key(&new_path) {
                return Err(ErrorMergeResult {
                    error_type: ErrorType::DuplicatePaths,
                    message: format!(
                        "Input {}: The path '{}' maps to '{}' and this has already been added by another input file",
                        input_index, original_path, new_path
                    ),
                });
            }

            // Clone path item and ensure unique operation IDs
            let mut copy_path_item = path_item.clone();
            ensure_unique_operation_ids(
                &mut copy_path_item,
                &mut seen_operation_ids,
                dispute.as_ref(),
            )?;

            result_paths.paths.insert(new_path, copy_path_item);
        }

        // Update references in the OAS after processing both components and paths
        walk_all_references(&mut oas, |ref_path| {
            if let Some(new_ref) = reference_modification.get(ref_path) {
                return new_ref.clone();
            }

            // Check for prefix matches
            let matching_keys: Vec<_> = reference_modification
                .keys()
                .filter(|key| key.starts_with(&format!("{}/", ref_path)))
                .collect();

            if matching_keys.len() > 1 {
                panic!(
                    "Found more than one matching key for reference '{}': {:?}",
                    ref_path, matching_keys
                );
            } else if matching_keys.len() == 1 {
                return reference_modification[matching_keys[0]].clone();
            }

            ref_path.to_string()
        });
    }

    Ok((result_paths, result_components))
}

fn apply_path_modification(path: &str, path_modification: Option<&PathModification>) -> String {
    let path_modification = match path_modification {
        Some(pm) => pm,
        None => return path.to_string(),
    };

    let mut result = path.to_string();

    // Strip start
    if let Some(strip_start) = &path_modification.strip_start {
        if result.starts_with(strip_start) {
            result = result[strip_start.len()..].to_string();
        }
    }

    // Prepend
    if let Some(prepend) = &path_modification.prepend {
        result = format!("{}{}", prepend, result);
    }

    result
}

fn drop_path_items_with_no_operations(mut oas: OpenAPI) -> OpenAPI {
    oas.paths.paths.retain(|_, path_item| {
        match path_item {
            ReferenceOr::Item(item) => {
                item.get.is_some()
                    || item.put.is_some()
                    || item.post.is_some()
                    || item.delete.is_some()
                    || item.options.is_some()
                    || item.head.is_some()
                    || item.patch.is_some()
                    || item.trace.is_some()
            }
            ReferenceOr::Reference { .. } => true, // Keep references
        }
    });
    oas
}

fn ensure_unique_operation_ids(
    path_item: &mut ReferenceOr<PathItem>,
    seen_operation_ids: &mut std::collections::HashSet<String>,
    dispute: Option<&crate::data::Dispute>,
) -> Result<(), ErrorMergeResult> {
    match path_item {
        ReferenceOr::Item(item) => {
            if let Some(op) = item.get.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.put.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.post.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.delete.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.patch.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.head.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.trace.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
            if let Some(op) = item.options.as_mut() {
                if let Some(operation_id) = &op.operation_id {
                    let unique_id =
                        find_unique_operation_id(operation_id, seen_operation_ids, dispute)?;
                    op.operation_id = Some(unique_id.clone());
                    seen_operation_ids.insert(unique_id);
                }
            }
        }
        ReferenceOr::Reference { .. } => {
            // References don't have operation IDs
        }
    }

    Ok(())
}

fn find_unique_operation_id(
    operation_id: &str,
    seen_operation_ids: &std::collections::HashSet<String>,
    dispute: Option<&crate::data::Dispute>,
) -> Result<String, ErrorMergeResult> {
    if !seen_operation_ids.contains(operation_id) {
        return Ok(operation_id.to_string());
    }

    // Try dispute prefix
    if let Some(dispute) = dispute {
        let dispute_op_id = apply_dispute(Some(dispute), operation_id, DisputeStatus::Disputed);
        if !seen_operation_ids.contains(&dispute_op_id) {
            return Ok(dispute_op_id);
        }
    }

    // Try incremental numbering
    for anti_conflict in 1..1000 {
        let try_op_id = format!("{}{}", operation_id, anti_conflict);
        if !seen_operation_ids.contains(&try_op_id) {
            return Ok(try_op_id);
        }
    }

    Err(ErrorMergeResult {
        error_type: ErrorType::OperationIdConflict,
        message: format!(
            "Could not resolve a conflict for the operationId '{}'",
            operation_id
        ),
    })
}

// Helper functions for processing different component types
pub fn process_schemas(
    results: &mut IndexMap<String, ReferenceOr<Schema>>,
    schemas: &IndexMap<String, ReferenceOr<Schema>>,
    dispute: &Option<crate::data::Dispute>,
    reference_modification: &mut std::collections::HashMap<String, String>,
) -> Result<(), ErrorMergeResult> {
    process_components_with_prefix(results, schemas, dispute, reference_modification, "schemas")
}

pub fn process_responses(
    results: &mut IndexMap<String, ReferenceOr<Response>>,
    responses: &IndexMap<String, ReferenceOr<Response>>,
    dispute: &Option<crate::data::Dispute>,
    reference_modification: &mut std::collections::HashMap<String, String>,
) -> Result<(), ErrorMergeResult> {
    process_components_with_prefix(
        results,
        responses,
        dispute,
        reference_modification,
        "responses",
    )
}

pub fn process_parameters(
    results: &mut IndexMap<String, ReferenceOr<Parameter>>,
    parameters: &IndexMap<String, ReferenceOr<Parameter>>,
    dispute: &Option<crate::data::Dispute>,
    reference_modification: &mut std::collections::HashMap<String, String>,
) -> Result<(), ErrorMergeResult> {
    process_components_with_prefix(
        results,
        parameters,
        dispute,
        reference_modification,
        "parameters",
    )
}

fn process_components_with_prefix<T>(
    results: &mut IndexMap<String, T>,
    components: &IndexMap<String, T>,
    dispute: &Option<crate::data::Dispute>,
    reference_modification: &mut std::collections::HashMap<String, String>,
    prefix: &str,
) -> Result<(), ErrorMergeResult>
where
    T: Clone + serde::Serialize,
{
    for (key, component) in components {
        let modified_key = apply_dispute(dispute.as_ref(), key, DisputeStatus::Undisputed);

        if modified_key != *key {
            reference_modification.insert(
                format!("#/components/{}/{}", prefix, key),
                format!("#/components/{}/{}", prefix, modified_key),
            );
        }

        if results.get(&modified_key).is_none()
            || components_equal::<T>(&results[&modified_key], component)
        {
            results.insert(modified_key.clone(), component.clone());
        } else {
            // Conflict resolution logic (same as before)
            let mut schema_placed = false;

            if let Some(dispute) = dispute {
                let preferred_key = apply_dispute(Some(dispute), key, DisputeStatus::Disputed);
                if results.get(&preferred_key).is_none()
                    || components_equal(&results[&preferred_key], component)
                {
                    results.insert(preferred_key.clone(), component.clone());
                    reference_modification.insert(
                        format!("#/components/{}/{}", prefix, key),
                        format!("#/components/{}/{}", prefix, preferred_key),
                    );
                    schema_placed = true;
                }
            }

            if !schema_placed {
                for anti_conflict in 1..1000 {
                    let try_key = format!("{}{}", key, anti_conflict);
                    if results.get(&try_key).is_none() {
                        results.insert(try_key.clone(), component.clone());
                        reference_modification.insert(
                            format!("#/components/{}/{}", prefix, key),
                            format!("#/components/{}/{}", prefix, try_key),
                        );
                        schema_placed = true;
                        break;
                    }
                }
            }

            if !schema_placed {
                return Err(ErrorMergeResult {
                    error_type: ErrorType::ComponentDefinitionConflict,
                    message: format!(
                        "The \"{}\" definition had a duplicate in a previous input and could not be deduplicated.",
                        key
                    ),
                });
            }
        }
    }

    Ok(())
}

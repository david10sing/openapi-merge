//! OpenAPI merging logic

pub mod component_equivalence;
pub mod dispute;
pub mod extensions;
pub mod info;
pub mod operation_selection;
pub mod paths_components;
pub mod reference_walker;
pub mod tags;

use crate::data::{ErrorMergeResult, ErrorType, MergeInput};
use openapiv3::OpenAPI;

/// Merge multiple OpenAPI files into a single file
pub fn merge(
    inputs: &MergeInput,
    openapi_version: Option<&str>,
) -> Result<OpenAPI, ErrorMergeResult> {
    if inputs.is_empty() {
        return Err(ErrorMergeResult {
            error_type: ErrorType::NoInputs,
            message: "You must provide at least one OAS file as an input.".to_string(),
        });
    }

    // Determine OpenAPI version
    let version = if let Some(version) = openapi_version {
        version.to_string()
    } else {
        // Use version from first input
        inputs[0].oas.openapi.clone()
    };

    // Merge paths and components
    let (paths, components) = paths_components::merge_paths_and_components(inputs)?;

    // Merge other parts
    let info = info::merge_infos(inputs);
    let tags = tags::merge_tags(inputs).unwrap_or_default();
    let servers = inputs
        .iter()
        .find(|input| !input.oas.servers.is_empty())
        .map(|input| input.oas.servers.clone())
        .unwrap_or_default();
    let external_docs = inputs
        .iter()
        .find_map(|input| input.oas.external_docs.as_ref())
        .cloned();
    let security = inputs
        .iter()
        .find_map(|input| input.oas.security.as_ref())
        .cloned();

    // Build output
    let mut output = OpenAPI {
        openapi: version,
        info,
        servers,
        paths,
        components: Some(components),
        security,
        tags,
        external_docs,
        extensions: Default::default(),
    };

    // Merge extensions
    extensions::merge_extensions(&mut output, inputs);

    Ok(output)
}

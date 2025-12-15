//! Info merging logic

use crate::data::{MergeInput, SingleMergeInput};
use openapiv3::Info;

/// Merge info objects from all inputs
pub fn merge_infos(inputs: &MergeInput) -> Info {
    if inputs.is_empty() {
        return Info {
            title: "Merged API".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
            extensions: Default::default(),
        };
    }

    // Start with first input's info
    let mut final_info = inputs[0].oas.info.clone();

    // Collect descriptions to append
    let mut appended_descriptions = Vec::new();

    for input in inputs {
        if let Some(desc_config) = &input.description {
            if desc_config.append {
                if let Some(description) = get_info_description_with_heading(input) {
                    appended_descriptions.push(description);
                }
            }
        }
    }

    // Append descriptions
    if !appended_descriptions.is_empty() {
        final_info.description = Some(appended_descriptions.join("\n\n"));
    }

    final_info
}

fn get_info_description_with_heading(input: &SingleMergeInput) -> Option<String> {
    let description = input.oas.info.description.as_ref()?;
    let trimmed_description = description.trim_end();

    let title = match &input.description {
        Some(desc_config) => desc_config.title.as_ref()?,
        None => return Some(trimmed_description.to_string()),
    };

    let heading_level = title.heading_level.unwrap_or(1);
    let heading = "#".repeat(heading_level as usize);

    Some(format!(
        "{} {}\n\n{}",
        heading, title.value, trimmed_description
    ))
}

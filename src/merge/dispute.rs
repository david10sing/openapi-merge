//! Dispute resolution logic

use crate::data::{Dispute, DisputePrefix, SingleMergeInput};

/// Extract dispute configuration from input
pub fn get_dispute(input: &SingleMergeInput) -> Option<Dispute> {
    // Check for deprecated disputePrefix first
    if let Some(prefix) = &input.dispute_prefix {
        return Some(Dispute::Prefix(DisputePrefix {
            prefix: prefix.clone(),
            always_apply: None,
        }));
    }

    // Check for new dispute format
    input.dispute.clone()
}

/// Dispute status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisputeStatus {
    Disputed,
    Undisputed,
}

/// Apply dispute prefix or suffix to a string
pub fn apply_dispute(dispute: Option<&Dispute>, input: &str, status: DisputeStatus) -> String {
    let dispute = match dispute {
        Some(d) => d,
        None => return input.to_string(),
    };

    let should_apply = match status {
        DisputeStatus::Disputed => true,
        DisputeStatus::Undisputed => match dispute {
            Dispute::Prefix(p) => p.always_apply.unwrap_or(false),
            Dispute::Suffix(s) => s.always_apply.unwrap_or(false),
        },
    };

    if !should_apply {
        return input.to_string();
    }

    match dispute {
        Dispute::Prefix(prefix) => format!("{}{}", prefix.prefix, input),
        Dispute::Suffix(suffix) => format!("{}{}", input, suffix.suffix),
    }
}


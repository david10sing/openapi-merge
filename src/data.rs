//! Core data structures for OpenAPI merging

use serde::{Deserialize, Serialize};
use openapiv3::OpenAPI;

/// Operation selection criteria for filtering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSelection {
    /// Only operations that have these tags will be taken from this OpenAPI file.
    /// If a single Operation contains an includeTag and an excludeTag then it will be excluded;
    /// exclusion takes precedence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tags: Option<Vec<String>>,

    /// Any Operation that has any one of these tags will be excluded from the final result.
    /// If a single Operation contains an includeTag and an excludeTag then it will be excluded;
    /// exclusion takes precedence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tags: Option<Vec<String>>,
}

/// Path modification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathModification {
    /// If a path starts with these characters, then strip them from the beginning of the path.
    /// Will run before prepend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_start: Option<String>,

    /// Append these characters to the start of the paths for this input.
    /// Will run after strip_start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepend: Option<String>,
}

/// Description merge behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionMergeBehaviour {
    /// Whether or not the description for this OpenAPI file will be merged into the description
    /// of the final file.
    pub append: bool,

    /// You may optionally include a Markdown Title to demarcate this particular section
    /// of the merged description files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<DescriptionTitle>,
}

/// Description title configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionTitle {
    /// The value of the included title.
    pub value: String,

    /// What heading level this heading will be at: from h1 through to h6.
    /// The default value is 1 and will create h1 elements in Markdown format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_level: Option<u8>,
}

/// Dispute resolution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dispute {
    /// Dispute with a prefix
    Prefix(DisputePrefix),
    /// Dispute with a suffix
    Suffix(DisputeSuffix),
}

/// Dispute prefix configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputePrefix {
    /// The prefix to use when a schema is in dispute.
    pub prefix: String,

    /// If this is set to true, then this prefix will always be applied to every Schema,
    /// even if there is no dispute for that particular schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_apply: Option<bool>,
}

/// Dispute suffix configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeSuffix {
    /// The suffix to use when a schema is in dispute.
    pub suffix: String,

    /// If this is set to true, then this suffix will always be applied to every Schema,
    /// even if there is no dispute for that particular schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_apply: Option<bool>,
}

/// Single merge input
#[derive(Debug, Clone)]
pub struct SingleMergeInput {
    pub oas: OpenAPI,
    pub path_modification: Option<PathModification>,
    pub operation_selection: Option<OperationSelection>,
    pub description: Option<DescriptionMergeBehaviour>,
    pub dispute: Option<Dispute>,
    #[allow(dead_code)] // Deprecated but kept for compatibility
    pub dispute_prefix: Option<String>,
}

/// Merge input - array of single merge inputs
pub type MergeInput = Vec<SingleMergeInput>;

/// Error types for merge operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    NoInputs,
    DuplicatePaths,
    ComponentDefinitionConflict,
    OperationIdConflict,
}

/// Error result from merge operation
#[derive(Debug, Clone)]
pub struct ErrorMergeResult {
    pub error_type: ErrorType,
    pub message: String,
}

/// Successful merge result
#[derive(Debug, Clone)]
pub struct SuccessfulMergeResult {
    pub output: OpenAPI,
}

/// Merge result - either success or error
#[derive(Debug, Clone)]
pub enum MergeResult {
    Success(SuccessfulMergeResult),
    Error(ErrorMergeResult),
}

impl MergeResult {
    pub fn is_error(&self) -> bool {
        matches!(self, MergeResult::Error(_))
    }
}

/// Configuration input from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationInputFromFile {
    /// The path to the input OpenAPI Schema that will be merged.
    pub input_file: String,

    #[serde(flatten)]
    pub base: ConfigurationInputBase,
}

/// Configuration input from URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationInputFromUrl {
    /// The input url that we should load our configuration file from.
    #[serde(rename = "inputURL")]
    pub input_url: String,

    #[serde(flatten)]
    pub base: ConfigurationInputBase,
}

/// Base configuration input properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationInputBase {
    /// For this input, you can perform these modifications to its paths elements.
    #[serde(rename = "pathModification", skip_serializing_if = "Option::is_none")]
    pub path_modification: Option<PathModification>,

    /// Choose which OpenAPI Operations should be included from this input.
    #[serde(rename = "operationSelection", skip_serializing_if = "Option::is_none")]
    pub operation_selection: Option<OperationSelection>,

    /// This configuration setting lets you configure how the info.description from this OpenAPI
    /// file will be merged into the final resulting OpenAPI file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<DescriptionMergeBehaviour>,

    /// The dispute algorithm that should be used for this input (new format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispute: Option<Dispute>,

    /// The prefix that will be used in the event of a conflict of two definition names (deprecated).
    #[serde(rename = "disputePrefix", skip_serializing_if = "Option::is_none")]
    pub dispute_prefix: Option<String>,
}

/// Configuration input - either from file or URL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigurationInput {
    FromFile(ConfigurationInputFromFile),
    FromUrl(ConfigurationInputFromUrl),
}

impl ConfigurationInput {
    pub fn path_modification(&self) -> Option<&PathModification> {
        match self {
            ConfigurationInput::FromFile(input) => input.base.path_modification.as_ref(),
            ConfigurationInput::FromUrl(input) => input.base.path_modification.as_ref(),
        }
    }

    pub fn operation_selection(&self) -> Option<&OperationSelection> {
        match self {
            ConfigurationInput::FromFile(input) => input.base.operation_selection.as_ref(),
            ConfigurationInput::FromUrl(input) => input.base.operation_selection.as_ref(),
        }
    }

    pub fn description(&self) -> Option<&DescriptionMergeBehaviour> {
        match self {
            ConfigurationInput::FromFile(input) => input.base.description.as_ref(),
            ConfigurationInput::FromUrl(input) => input.base.description.as_ref(),
        }
    }

    pub fn dispute(&self) -> Option<&Dispute> {
        match self {
            ConfigurationInput::FromFile(input) => input.base.dispute.as_ref(),
            ConfigurationInput::FromUrl(input) => input.base.dispute.as_ref(),
        }
    }

    pub fn dispute_prefix(&self) -> Option<&String> {
        match self {
            ConfigurationInput::FromFile(input) => input.base.dispute_prefix.as_ref(),
            ConfigurationInput::FromUrl(input) => input.base.dispute_prefix.as_ref(),
        }
    }
}

/// Configuration for the OpenAPI Merge CLI Tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// The input items for the merge algorithm. You must provide at least one.
    pub inputs: Vec<ConfigurationInput>,

    /// The output file to put the results in. If you use the .yml or .yaml extension then
    /// the schema will be output in YAML format, otherwise, it will be output in JSON format.
    pub output: String,

    /// Optional OpenAPI version to use for the output. If not specified, will use the version
    /// from the first input file.
    #[serde(rename = "openapiVersion", skip_serializing_if = "Option::is_none")]
    pub openapi_version: Option<String>,
}


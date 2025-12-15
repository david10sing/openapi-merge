//! OpenAPI Merge Library
//! 
//! A library for merging multiple OpenAPI 3.0 specification files into a single file.

pub mod config;
pub mod data;
pub mod file_loading;
pub mod merge;

pub use data::{MergeInput, SingleMergeInput, Configuration, ConfigurationInput};
pub use merge::merge;


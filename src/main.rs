//! OpenAPI Merge CLI
//! 
//! Command-line tool for merging multiple OpenAPI specification files.

use anyhow::Result;
use clap::Parser;
use openapi_merge::config::load_configuration;
use openapi_merge::file_loading::load_oas_for_input;
use openapi_merge::merge::merge;
use openapi_merge::data::{ConfigurationInput, SingleMergeInput};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "openapi-merge")]
#[command(version)]
#[command(about = "A CLI tool for merging multiple OpenAPI specification files")]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "openapi-merge.json")]
    config: PathBuf,
}

const ERROR_LOADING_CONFIG: i32 = 1;
const ERROR_LOADING_INPUTS: i32 = 2;
const ERROR_MERGING: i32 = 3;

struct LogWithMillisDiff {
    prev_time: Instant,
}

impl LogWithMillisDiff {
    fn new() -> Self {
        Self {
            prev_time: Instant::now(),
        }
    }

    fn log(&mut self, message: &str) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.prev_time).as_millis();
        println!("{} (+{}ms)", message, elapsed);
        self.prev_time = now;
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut logger = LogWithMillisDiff::new();
    
    logger.log(&format!("## Running openapi-merge v{}", env!("CARGO_PKG_VERSION")));

    // Load configuration
    let config = match load_configuration(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(ERROR_LOADING_CONFIG);
        }
    };

    logger.log(&format!("## Loaded the configuration: {} inputs", config.inputs.len()));

    let base_path = cli.config.parent().unwrap_or(std::path::Path::new("."));

    // Load all input files
    let inputs = match convert_inputs(base_path, &config.inputs, &mut logger) {
        Ok(inputs) => inputs,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(ERROR_LOADING_INPUTS);
        }
    };

    logger.log("## Loaded the inputs into memory, merging the results.");

    // Merge the inputs
    let merge_result = merge(&inputs, config.openapi_version.as_deref());

    match merge_result {
        Ok(output) => {
            let output_path = base_path.join(&config.output);
            logger.log(&format!("## Inputs merged, writing the results out to '{}'", output_path.display()));

            // Write output
            if let Err(e) = write_output(&output_path, &output) {
                eprintln!("Error writing output: {}", e);
                std::process::exit(ERROR_MERGING);
            }

            logger.log(&format!("## Finished writing to '{}'", output_path.display()));
        }
        Err(e) => {
            eprintln!("Error merging files: {:?}", e);
            std::process::exit(ERROR_MERGING);
        }
    }

    Ok(())
}

fn convert_inputs(
    base_path: &std::path::Path,
    config_inputs: &[ConfigurationInput],
    logger: &mut LogWithMillisDiff,
) -> Result<Vec<SingleMergeInput>> {
    let mut inputs = Vec::new();

    for (input_index, config_input) in config_inputs.iter().enumerate() {
        let oas = load_oas_for_input(
            base_path,
            config_input,
            input_index,
            &mut |msg| logger.log(msg),
        )?;
        
        let single_input = SingleMergeInput {
            oas,
            path_modification: config_input.path_modification().cloned(),
            operation_selection: config_input.operation_selection().cloned(),
            description: config_input.description().cloned(),
            dispute: config_input.dispute().cloned(),
            dispute_prefix: config_input.dispute_prefix().cloned(),
        };

        inputs.push(single_input);
    }

    Ok(inputs)
}

fn write_output(output_path: &std::path::Path, output: &openapiv3::OpenAPI) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let extension = output_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("json");

    let content = if extension == "yaml" || extension == "yml" {
        serde_yaml::to_string(output)?
    } else {
        serde_json::to_string_pretty(output)?
    };

    let mut file = File::create(output_path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}


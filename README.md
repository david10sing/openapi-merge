## The openapi-merge repository

Welcome to the openapi-merge repository. This library is intended to be used for merging multiple OpenAPI 3.0 files together. The most common reason that developers want to do this is because they have multiple services that they wish to expose underneath a single API Gateway. Therefore, even though this merging logic is sufficiently generic to be used for most use cases, some of the feature decisions are tailored for that specific use case.

### Screenshots

![Imgur](https://i.imgur.com/GjnSXCS.png)
(An example of creating an openapi-merge.json configuration file for the CLI tool)

### About this repository

This repository contains a Rust CLI tool for merging OpenAPI specification files. The tool can merge multiple OpenAPI files into a single file, handling path modifications, operation selection, component deduplication, and dispute resolution.

### Installation

This project requires Rust (1.70 or later). To build from source:

```bash
cargo build --release
```

The binary will be located at `target/release/openapi-merge`.

### Usage

Create a configuration file `openapi-merge.json`:

```json
{
  "inputs": [
    {
      "inputFile": "./api1.yaml",
      "pathModification": {
        "prepend": "/api/v1"
      }
    },
    {
      "inputURL": "https://example.com/api2.yaml",
      "dispute": {
        "prefix": "Api2"
      }
    }
  ],
  "output": "./merged.yaml",
  "openapiVersion": "3.0.3"
}
```

Run the tool:

```bash
openapi-merge --config openapi-merge.json
```

Or use the default configuration file name:

```bash
openapi-merge
```

### Configuration

The configuration file supports:

- **inputs**: Array of input OpenAPI files (from local files or URLs)
- **output**: Output file path (YAML if `.yaml`/`.yml`, JSON otherwise)
- **openapiVersion**: Optional OpenAPI version for output (defaults to version from first input)

Each input can specify:
- **pathModification**: Modify paths (stripStart, prepend)
- **operationSelection**: Filter operations by tags (includeTags, excludeTags)
- **description**: Merge description with optional markdown title
- **dispute**: Resolve component name conflicts (prefix or suffix)

### Developing on openapi-merge

After checking out this repository, you can build and test:

```bash
# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --config openapi-merge.json
```

### Features

- Merge multiple OpenAPI files into one
- Path modifications (strip prefix, prepend prefix)
- Operation selection by tags
- Component deduplication with conflict resolution
- Dispute resolution (prefix/suffix for conflicting component names)
- Reference updating across merged documents
- Support for both YAML and JSON input/output
- Load files from local paths or URLs
- Configurable OpenAPI version
//! Reference walking and updating logic

use openapiv3::*;

/// Modify function type for reference updates
pub type Modify = Box<dyn Fn(&str) -> String>;

/// Walk all references in an OpenAPI document and update them
pub fn walk_all_references<F>(oas: &mut OpenAPI, modify: F)
where
    F: Fn(&str) -> String,
{
    // Use paths.paths to access the inner IndexMap
    for path_item in oas.paths.paths.values_mut() {
        walk_path_item_references(path_item, &modify);
    }

    if let Some(components) = &mut oas.components {
        walk_component_references(components, &modify);
    }
}

fn walk_path_item_references<F>(path_item: &mut ReferenceOr<PathItem>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match path_item {
        ReferenceOr::Item(item) => {
            // Operations are Option<Operation>, not ReferenceOr<Operation>
            if let Some(op) = &mut item.get {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.put {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.post {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.delete {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.options {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.head {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.patch {
                walk_operation_references(op, modify);
            }
            if let Some(op) = &mut item.trace {
                walk_operation_references(op, modify);
            }
            // Parameters is Vec, use iter_mut
            for param in item.parameters.iter_mut() {
                walk_parameter_references(param, modify);
            }
        }
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
    }
}

fn walk_operation_references<F>(operation: &mut Operation, modify: &F)
where
    F: Fn(&str) -> String,
{
    // Parameters is Vec
    for param in operation.parameters.iter_mut() {
        walk_parameter_references(param, modify);
    }
    if let Some(request_body) = &mut operation.request_body {
        walk_request_body_references(request_body, modify);
    }
    for response in operation.responses.responses.values_mut() {
        walk_response_references(response, modify);
    }
    // Note: callbacks are not directly on Operation in openapiv3 v1.0.4
    // They would be handled through extensions if needed
}

fn walk_schema_references<F>(schema: &mut ReferenceOr<Schema>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match schema {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(schema_item) => {
            walk_schema_kind_references(&mut schema_item.schema_kind, modify);
        }
    }
}

fn walk_boxed_schema_references<F>(schema: &mut ReferenceOr<Box<Schema>>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match schema {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(schema_item) => {
            walk_schema_kind_references(&mut schema_item.schema_kind, modify);
        }
    }
}

/// Walk references in a Box<ReferenceOr<Schema>>
fn walk_box_ref_schema_references<F>(schema: &mut Box<ReferenceOr<Schema>>, modify: &F)
where
    F: Fn(&str) -> String,
{
    walk_schema_references(schema.as_mut(), modify);
}

fn walk_schema_kind_references<F>(schema_kind: &mut SchemaKind, modify: &F)
where
    F: Fn(&str) -> String,
{
    match schema_kind {
        SchemaKind::Type(typ) => {
            walk_type_references(typ, modify);
        }
        SchemaKind::OneOf { one_of } => {
            for s in one_of.iter_mut() {
                walk_schema_references(s, modify);
            }
        }
        SchemaKind::AllOf { all_of } => {
            for s in all_of.iter_mut() {
                walk_schema_references(s, modify);
            }
        }
        SchemaKind::AnyOf { any_of } => {
            for s in any_of.iter_mut() {
                walk_schema_references(s, modify);
            }
        }
        SchemaKind::Not { not } => {
            walk_box_ref_schema_references(not, modify);
        }
        SchemaKind::Any(any_schema) => {
            walk_any_schema_references(any_schema, modify);
        }
    }
}

fn walk_type_references<F>(typ: &mut Type, modify: &F)
where
    F: Fn(&str) -> String,
{
    match typ {
        Type::Object(obj) => {
            for prop in obj.properties.values_mut() {
                walk_boxed_schema_references(prop, modify);
            }
            if let Some(additional_properties) = &mut obj.additional_properties {
                if let AdditionalProperties::Schema(s) = additional_properties {
                    walk_box_ref_schema_references(s, modify);
                }
            }
        }
        Type::Array(arr) => {
            if let Some(items) = &mut arr.items {
                walk_boxed_schema_references(items, modify);
            }
        }
        Type::String(_) | Type::Number(_) | Type::Integer(_) | Type::Boolean {} => {}
    }
}

fn walk_any_schema_references<F>(any_schema: &mut AnySchema, modify: &F)
where
    F: Fn(&str) -> String,
{
    for prop in any_schema.properties.values_mut() {
        walk_boxed_schema_references(prop, modify);
    }
    if let Some(additional_properties) = &mut any_schema.additional_properties {
        if let AdditionalProperties::Schema(s) = additional_properties {
            walk_box_ref_schema_references(s, modify);
        }
    }
    if let Some(items) = &mut any_schema.items {
        walk_boxed_schema_references(items, modify);
    }
    for s in any_schema.one_of.iter_mut() {
        walk_schema_references(s, modify);
    }
    for s in any_schema.all_of.iter_mut() {
        walk_schema_references(s, modify);
    }
    for s in any_schema.any_of.iter_mut() {
        walk_schema_references(s, modify);
    }
    if let Some(not) = &mut any_schema.not {
        walk_box_ref_schema_references(not, modify);
    }
}

fn walk_media_type_references<F>(media_type: &mut MediaType, modify: &F)
where
    F: Fn(&str) -> String,
{
    if let Some(schema) = &mut media_type.schema {
        walk_schema_references(schema, modify);
    }
    for example in media_type.examples.values_mut() {
        walk_example_references(example, modify);
    }
}

fn walk_example_references<F>(example: &mut ReferenceOr<Example>, modify: &F)
where
    F: Fn(&str) -> String,
{
    if let ReferenceOr::Reference { reference } = example {
        *reference = modify(reference);
    }
}

fn walk_parameter_references<F>(parameter: &mut ReferenceOr<Parameter>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match parameter {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(param) => {
            // Parameter is an enum with parameter_data
            let param_data = get_parameter_data_mut(param);
            walk_parameter_schema_or_content_references(&mut param_data.format, modify);
            for example in param_data.examples.values_mut() {
                walk_example_references(example, modify);
            }
        }
    }
}

fn get_parameter_data_mut(param: &mut Parameter) -> &mut ParameterData {
    match param {
        Parameter::Query { parameter_data, .. } => parameter_data,
        Parameter::Header { parameter_data, .. } => parameter_data,
        Parameter::Path { parameter_data, .. } => parameter_data,
        Parameter::Cookie { parameter_data, .. } => parameter_data,
    }
}

fn walk_parameter_schema_or_content_references<F>(format: &mut ParameterSchemaOrContent, modify: &F)
where
    F: Fn(&str) -> String,
{
    match format {
        ParameterSchemaOrContent::Schema(schema) => {
            walk_schema_references(schema, modify);
        }
        ParameterSchemaOrContent::Content(content) => {
            for media_type in content.values_mut() {
                walk_media_type_references(media_type, modify);
            }
        }
    }
}

fn walk_request_body_references<F>(request_body: &mut ReferenceOr<RequestBody>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match request_body {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(body) => {
            for media_type in body.content.values_mut() {
                walk_media_type_references(media_type, modify);
            }
        }
    }
}

fn walk_header_references<F>(header: &mut ReferenceOr<Header>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match header {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(header_item) => {
            walk_parameter_schema_or_content_references(&mut header_item.format, modify);
            for example in header_item.examples.values_mut() {
                walk_example_references(example, modify);
            }
        }
    }
}

fn walk_link_references<F>(link: &mut ReferenceOr<Link>, modify: &F)
where
    F: Fn(&str) -> String,
{
    if let ReferenceOr::Reference { reference } = link {
        *reference = modify(reference);
    }
}

fn walk_response_references<F>(response: &mut ReferenceOr<Response>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match response {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(resp) => {
            for header in resp.headers.values_mut() {
                walk_header_references(header, modify);
            }
            for media_type in resp.content.values_mut() {
                walk_media_type_references(media_type, modify);
            }
            for link in resp.links.values_mut() {
                walk_link_references(link, modify);
            }
        }
    }
}

fn walk_callback_references<F>(callback: &mut ReferenceOr<Callback>, modify: &F)
where
    F: Fn(&str) -> String,
{
    match callback {
        ReferenceOr::Reference { reference } => {
            *reference = modify(reference);
        }
        ReferenceOr::Item(callback_item) => {
            // Callback is IndexMap<String, PathItem>, not ReferenceOr<PathItem>
            for path_item in callback_item.values_mut() {
                walk_path_item_inner_references(path_item, modify);
            }
        }
    }
}

/// Walk references in a PathItem directly (not wrapped in ReferenceOr)
fn walk_path_item_inner_references<F>(item: &mut PathItem, modify: &F)
where
    F: Fn(&str) -> String,
{
    if let Some(op) = &mut item.get {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.put {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.post {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.delete {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.options {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.head {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.patch {
        walk_operation_references(op, modify);
    }
    if let Some(op) = &mut item.trace {
        walk_operation_references(op, modify);
    }
    for param in item.parameters.iter_mut() {
        walk_parameter_references(param, modify);
    }
}

fn walk_component_references<F>(components: &mut Components, modify: &F)
where
    F: Fn(&str) -> String,
{
    for schema in components.schemas.values_mut() {
        walk_schema_references(schema, modify);
    }
    for response in components.responses.values_mut() {
        walk_response_references(response, modify);
    }
    for parameter in components.parameters.values_mut() {
        walk_parameter_references(parameter, modify);
    }
    for example in components.examples.values_mut() {
        walk_example_references(example, modify);
    }
    for request_body in components.request_bodies.values_mut() {
        walk_request_body_references(request_body, modify);
    }
    for header in components.headers.values_mut() {
        walk_header_references(header, modify);
    }
    for link in components.links.values_mut() {
        walk_link_references(link, modify);
    }
    for callback in components.callbacks.values_mut() {
        walk_callback_references(callback, modify);
    }
}

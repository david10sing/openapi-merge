# Type Fixes for openapiv3 Crate Compatibility

This document lists common type mismatches and their fixes when using the `openapiv3` crate.

## Common Issues and Fixes

### 1. Paths Type
**Issue**: `Paths::new()` doesn't exist
**Fix**: Use `Paths::default()` or `Default::default()`

### 2. Reference Path Access
**Issue**: `ref_.ref_path` might not exist
**Fix**: Check if it's `ref_.$ref` or `ref_.ref` instead

### 3. PathItem::Reference Structure
**Issue**: Reference structure might be different
**Fix**: May need to use `PathItem::Ref` or check actual field names

### 4. ReferenceOr Structure
**Issue**: Pattern matching might be incorrect
**Fix**: Verify if it's `ReferenceOr::Item` and `ReferenceOr::Ref` or different variant names

### 5. OpenAPI Construction
**Issue**: Field names or types might differ
**Fix**: Check actual OpenAPI struct definition

### 6. Info Struct Fields
**Issue**: Field names might be different (e.g., `terms_of_service` vs `termsOfService`)
**Fix**: Use serde field renaming or check actual struct

### 7. Components Structure
**Issue**: Components might use different collection types
**Fix**: Verify HashMap vs BTreeMap usage

## Quick Fixes to Apply

1. Replace `Paths::new()` with `Paths::default()`
2. Check reference field names (`ref_path` vs `$ref` vs `ref`)
3. Verify enum variant names for `PathItem` and `ReferenceOr`
4. Check Info struct field names and types
5. Verify OpenAPI struct field names match crate definition


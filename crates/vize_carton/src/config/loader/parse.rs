//! Config format dispatch.
//!
//! Discovery feeds candidate paths into this module. It picks the concrete
//! reader from the file extension and implements the fallback rule used during
//! auto-discovery: malformed configs warn, while an unavailable PKL runtime can
//! continue to the next lower-priority config format.

use std::path::{Path, PathBuf};

use crate::config::ConfigDocument;

use super::{
    BeforeConfigRead,
    discovery::{CONFIG_MIGRATION, is_retired_config},
    js::parse_js_config_observed,
    pkl,
};

/// Parse a single config file path without discovery fallback.
pub(super) fn parse_raw_config_file(
    path: &Path,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    parse_raw_config_file_with_dependencies(path, &mut Vec::new(), &mut true)
}

pub(super) fn parse_raw_config_file_with_dependencies(
    path: &Path,
    dependencies: &mut Vec<PathBuf>,
    complete: &mut bool,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    parse_raw_config_file_observed(path, dependencies, complete, &mut None)
}

pub(super) fn parse_raw_config_file_observed(
    path: &Path,
    dependencies: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    parse_raw_config_file_with_compile_cache(path, dependencies, complete, before_read, None)
}

pub(super) fn parse_raw_config_file_with_compile_cache(
    path: &Path,
    dependencies: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
    compile_cache: Option<&Path>,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    if let Some(observe) = before_read.as_deref_mut() {
        observe(&[path.to_path_buf()]);
    }
    dependencies.push(path.to_path_buf());
    if is_retired_config(path) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, CONFIG_MIGRATION).into());
    }
    let mut config = match path.extension().and_then(|ext| ext.to_str()) {
        Some("pkl") => pkl::parse_pkl_config(path)?,
        Some("ts" | "js" | "mjs" | "cjs" | "mts" | "cts") => parse_js_config_observed(
            path,
            dependencies,
            complete,
            before_read,
            compile_cache,
            false,
        )?,
        Some("json") => {
            let content = std::fs::read_to_string(path)?;
            document_from_json(content.as_bytes(), path)?
        }
        _ => return Ok(ConfigDocument::default()),
    };

    if !super::CONFIG_FILE_NAMES
        .iter()
        .any(|name| path.file_name().is_some_and(|file| file == *name))
    {
        config.clear_project_root();
    }
    Ok(config)
}

/// Parse an explicitly selected CLI path as a Vite configuration regardless
/// of its basename. The retired dedicated shape is never accepted here.
pub(super) fn parse_explicit_vite_config_file_with_compile_cache(
    path: &Path,
    dependencies: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
    compile_cache: Option<&Path>,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    // Differential and host contracts reserve this basename for an explicit
    // serialized ConfigDocument. It bypasses Vite projection while arbitrary
    // user-selected filenames continue to use the Vite config shape.
    if is_native_serialization_carrier(path) {
        return parse_raw_config_file_with_compile_cache(
            path,
            dependencies,
            complete,
            before_read,
            compile_cache,
        );
    }
    if let Some(observe) = before_read.as_deref_mut() {
        observe(&[path.to_path_buf()]);
    }
    dependencies.push(path.to_path_buf());
    if is_retired_config(path) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, CONFIG_MIGRATION).into());
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ts" | "js" | "mjs" | "cjs" | "mts" | "cts") => parse_js_config_observed(
            path,
            dependencies,
            complete,
            before_read,
            compile_cache,
            true,
        ),
        Some("json") => explicit_vite_json(path),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "explicit --config must be a JavaScript, TypeScript, or JSON Vite config",
        )
        .into()),
    }
}

fn is_native_serialization_carrier(path: &Path) -> bool {
    path.file_stem()
        .is_some_and(|stem| stem == "serialized-config")
        && matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("pkl" | "ts" | "js" | "mjs" | "cjs" | "mts" | "cts" | "json")
        )
}

fn explicit_vite_json(path: &Path) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    let content = std::fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&content)?;
    reject_custom_config_extends(&value)?;
    let Some(source) = value.as_object() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Vite config must export an object",
        )
        .into());
    };
    if [
        "linter",
        "formatter",
        "languageServer",
        "globalTypes",
        "files",
        "ignores",
        "entries",
    ]
    .iter()
    .any(|key| source.contains_key(*key))
    {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, CONFIG_MIGRATION).into());
    }
    let Some(vue) = source.get("vue") else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, CONFIG_MIGRATION).into());
    };
    let Some(vue) = vue.as_object() else {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "vue: must be object").into(),
        );
    };
    let mut projected = vue.clone();
    if let Some(version) = projected.remove("version") {
        projected.insert("vue".into(), serde_json::json!({ "version": version }));
    }
    let type_checker = projected
        .entry("typeChecker")
        .or_insert_with(|| serde_json::json!({}));
    let Some(type_checker) = type_checker.as_object_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "vue.typeChecker: must be object",
        )
        .into());
    };
    type_checker
        .entry("jsxTypecheck")
        .or_insert(serde_json::Value::Bool(true));
    reject_custom_config_extends(&serde_json::Value::Object(projected.clone()))?;
    Ok(serde_json::from_value(serde_json::Value::Object(
        projected,
    ))?)
}

fn reject_custom_config_extends(
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let has_extends = |value: &serde_json::Value| {
        value
            .as_object()
            .is_some_and(|object| object.contains_key("extends"))
    };
    let entries = value
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .or_else(|| value.as_array());
    if has_extends(value) || entries.is_some_and(|entries| entries.iter().any(has_extends)) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput,
            "Custom config extends is no longer supported; use defineConfig(vpConfig).withVue(vueConfig); standard tsconfig extends remains supported").into());
    }
    Ok(())
}

/// Deserialize one evaluated native payload, refusing retired custom inheritance.
///
/// Every format shares this deserializer; keeping a separate str reader would
/// instantiate the entire model twice. Payloads without `extends` skip the
/// intermediate JSON value.
pub(super) fn document_from_json(
    json: &[u8],
    _path: &Path,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    // Internal serialized native payloads remain supported, but never load a
    // second custom inheritance graph beside TypeScript's own project model.
    if json
        .windows(b"\"extends\"".len())
        .any(|part| part == b"\"extends\"")
    {
        let value: serde_json::Value = serde_json::from_slice(json)?;
        reject_custom_config_extends(&value)?;
        return Ok(serde_json::from_value::<ConfigDocument>(value)?);
    }
    Ok(serde_json::from_slice::<ConfigDocument>(json)?)
}

/// Parse a discovered candidate and map recoverable failures to `None`.
pub(super) fn try_parse_raw_candidate(path: &Path) -> Option<ConfigDocument> {
    match parse_raw_config_file(path) {
        Ok(config) => Some(config),
        Err(error) => {
            let should_try_next = should_try_next_config(path, error.as_ref());
            eprintln!(
                "\x1b[33mWarning:\x1b[0m Failed to parse {}: {}",
                path.display(),
                error
            );
            if should_try_next {
                None
            } else {
                Some(ConfigDocument::default())
            }
        }
    }
}

fn should_try_next_config(path: &Path, error: &(dyn std::error::Error + 'static)) -> bool {
    if path.extension().and_then(|ext| ext.to_str()) != Some("pkl") {
        return true;
    }

    pkl::is_process_error_box(error)
}

#[cfg(test)]
mod tests {
    use super::parse_raw_config_file;

    #[test]
    fn json_config_preserves_unicode_and_escaped_strings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("serialized-config.json");
        std::fs::write(
            &path,
            r#"{
                "basePath": "日本語/🎨",
                "files": ["src/\u65e5\u672c.vue"],
                "formatter": {
                    "attributeGroups": [["é", "data-\"quoted\""]]
                }
            }"#,
        )
        .unwrap();

        let config = parse_raw_config_file(&path).unwrap();
        let entries = config.clone().into_entry_files();
        assert_eq!(entries[0].base_path.as_deref(), Some("日本語/🎨"));
        assert_eq!(entries[0].files[0], "src/日本.vue");
        let groups = config
            .into_config_and_features()
            .0
            .formatter
            .attribute_groups
            .unwrap();
        assert_eq!(groups[0][0], "é");
        assert_eq!(groups[0][1], "data-\"quoted\"");
    }

    #[test]
    fn json_config_rejects_invalid_utf8_before_deserialization() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("serialized-config.json");
        std::fs::write(&path, b"{\"basePath\":\"\xff\"}").unwrap();

        let error = parse_raw_config_file(&path).unwrap_err();
        let error = error.downcast_ref::<std::io::Error>().unwrap();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn json_config_preserves_trailing_input_error_location() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("serialized-config.json");
        std::fs::write(&path, "{}\r\n\r\n[]").unwrap();

        let error = parse_raw_config_file(&path).unwrap_err();
        let error = error.downcast_ref::<serde_json::Error>().unwrap();
        assert!(error.is_syntax());
        assert_eq!((error.line(), error.column()), (3, 1));
        assert_eq!(
            crate::cstr!("{error}"),
            "trailing characters at line 3 column 1"
        );
    }
}

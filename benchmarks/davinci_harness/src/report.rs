//! Bench-report JSON export.
//!
//! Every bench run is exported as one JSON file under
//! `bench/results/davinci/<bench_id>.json` (workspace-relative), shaped by
//! `schema/davinci-bench.schema.json`. In debug builds the exporter validates
//! each report against the committed schema before writing; the validator is
//! strict - a schema keyword it does not implement is an error, never a
//! silently skipped check.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Version stamped into every report as `harness_version`.
pub const HARNESS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Report directory relative to the workspace root.
pub const RESULTS_SUBDIR: &str = "bench/results/davinci";

/// Schema location relative to the workspace root.
pub const SCHEMA_SUBPATH: &str = "benchmarks/davinci_harness/schema/davinci-bench.schema.json";

/// Wall-clock percentiles in nanoseconds over the criterion measurement samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WallNs {
    /// Median wall time per iteration.
    pub p50: u64,
    /// 95th-percentile wall time per iteration.
    pub p95: u64,
}

/// One bench run, as exported to `bench/results/davinci/<bench_id>.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BenchReport<'a> {
    /// Stable bench identity; doubles as the report file name.
    pub bench_id: &'a str,
    /// Fixture identity (fixture path or a `synthetic:` tag).
    pub fixture: &'a str,
    /// Wall-clock percentiles.
    pub wall_ns: WallNs,
    /// Allocation-like calls; `None` when the counting allocator is not installed.
    pub allocs: Option<u64>,
    /// Peak live-byte growth over the measured window; `None` as above.
    pub alloc_bytes_peak: Option<u64>,
    /// Peak-RSS growth over the process baseline; `None` off macOS/Linux.
    pub rss_peak_bytes: Option<u64>,
    /// Producing harness version.
    pub harness_version: &'a str,
}

/// Everything that can go wrong while exporting a report.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("bench id `{0}` has characters outside [A-Za-z0-9._-]")]
    InvalidBenchId(Box<str>),
    #[error("no `[workspace]` Cargo.toml found above `{}`", .0.display())]
    WorkspaceRootNotFound(PathBuf),
    #[error("schema violation at `{path}`: expected {expected}, found {found}")]
    SchemaType {
        path: Box<str>,
        expected: Box<str>,
        found: Box<str>,
    },
    #[error("schema violation at `{path}`: missing required property `{property}`")]
    SchemaRequired { path: Box<str>, property: Box<str> },
    #[error("schema violation at `{path}`: unexpected property `{property}`")]
    SchemaUnexpectedProperty { path: Box<str>, property: Box<str> },
    #[error("schema violation at `{path}`: value is below minimum {minimum}")]
    SchemaMinimum { path: Box<str>, minimum: u64 },
    #[error("schema violation at `{path}`: string is shorter than minLength {min_length}")]
    SchemaMinLength { path: Box<str>, min_length: u64 },
    #[error("schema violation at `{path}`: string does not match pattern `{pattern}`")]
    SchemaPattern { path: Box<str>, pattern: Box<str> },
    #[error("schema pattern `{pattern}` at `{path}` does not compile")]
    SchemaBadPattern { path: Box<str>, pattern: Box<str> },
    #[error(
        "schema keyword `{keyword}` at `{path}` is not implemented by the davinci_harness validator"
    )]
    SchemaUnimplementedKeyword { path: Box<str>, keyword: Box<str> },
    #[error("schema at `{path}` must be a JSON object")]
    SchemaNotAnObject { path: Box<str> },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Reject bench ids that cannot serve as report file names.
pub fn validate_bench_id(bench_id: &str) -> Result<(), ReportError> {
    let valid = !bench_id.is_empty()
        && bench_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(ReportError::InvalidBenchId(bench_id.into()))
    }
}

/// Walk up from the current directory to the `[workspace]` Cargo.toml.
///
/// Cargo runs bench and test binaries with the package directory as the
/// working directory, so this resolves deterministically for every workspace
/// member without environment configuration.
pub fn workspace_root() -> Result<PathBuf, ReportError> {
    let start = std::env::current_dir()?;
    let mut dir = start.as_path();
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = fs::read_to_string(&manifest)?;
            if text.lines().any(|line| line.trim() == "[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Err(ReportError::WorkspaceRootNotFound(start)),
        }
    }
}

/// Write `report` to `bench/results/davinci/<bench_id>.json` under the
/// workspace root, returning the written path.
pub fn write(report: &BenchReport<'_>) -> Result<PathBuf, ReportError> {
    let dir = workspace_root()?.join(RESULTS_SUBDIR);
    write_to_dir(&dir, report)
}

/// Write `report` into an explicit directory (test seam for [`write`]).
pub fn write_to_dir(dir: &Path, report: &BenchReport<'_>) -> Result<PathBuf, ReportError> {
    validate_bench_id(report.bench_id)?;
    #[cfg(debug_assertions)]
    validate_against_schema(&serde_json::to_value(report)?)?;
    fs::create_dir_all(dir)?;
    let mut file_name = report.bench_id.to_owned();
    file_name.push_str(".json");
    let path = dir.join(file_name);
    let mut text = serde_json::to_string_pretty(report)?;
    text.push('\n');
    fs::write(&path, text)?;
    Ok(path)
}

/// Load the committed schema from the workspace.
pub fn load_schema() -> Result<serde_json::Value, ReportError> {
    let path = workspace_root()?.join(SCHEMA_SUBPATH);
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// Validate a serialized report against the committed schema.
pub fn validate_against_schema(report_json: &serde_json::Value) -> Result<(), ReportError> {
    let schema = load_schema()?;
    schema_check::validate(&schema, report_json, "$")
}

/// Minimal, strict JSON Schema subset validator.
///
/// Implements exactly the keywords the committed schema uses (`type`,
/// `required`, `properties`, `additionalProperties: false`, `minimum`,
/// `minLength`, `pattern`) and rejects any other validation keyword instead
/// of skipping it, so a schema edit cannot silently outrun the validator.
mod schema_check {
    use serde_json::Value;

    use super::ReportError;

    /// Schema keys that annotate rather than validate.
    const ANNOTATION_KEYWORDS: [&str; 4] = ["$schema", "$id", "title", "description"];
    /// Validation keywords this subset implements.
    const IMPLEMENTED_KEYWORDS: [&str; 7] = [
        "type",
        "required",
        "properties",
        "additionalProperties",
        "minimum",
        "minLength",
        "pattern",
    ];

    pub(super) fn validate(
        schema: &Value,
        instance: &Value,
        path: &str,
    ) -> Result<(), ReportError> {
        let Some(schema_object) = schema.as_object() else {
            return Err(ReportError::SchemaNotAnObject { path: path.into() });
        };
        for keyword in schema_object.keys() {
            let known = ANNOTATION_KEYWORDS.contains(&keyword.as_str())
                || IMPLEMENTED_KEYWORDS.contains(&keyword.as_str());
            if !known {
                return Err(ReportError::SchemaUnimplementedKeyword {
                    path: path.into(),
                    keyword: keyword.as_str().into(),
                });
            }
        }

        if let Some(expected) = schema_object.get("type") {
            check_type(expected, instance, path)?;
        }
        check_object_keywords(schema, instance, path)?;
        check_scalar_bounds(schema, instance, path)?;
        Ok(())
    }

    fn json_type_name(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(number) => {
                if number.is_i64() || number.is_u64() {
                    "integer"
                } else {
                    "number"
                }
            }
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    fn type_entry_matches(entry: &Value, instance: &Value) -> bool {
        let Some(name) = entry.as_str() else {
            return false;
        };
        let actual = json_type_name(instance);
        // JSON Schema treats integers as a subset of numbers.
        name == actual || (name == "number" && actual == "integer")
    }

    fn check_type(expected: &Value, instance: &Value, path: &str) -> Result<(), ReportError> {
        let matches = match expected {
            Value::Array(entries) => entries
                .iter()
                .any(|entry| type_entry_matches(entry, instance)),
            single => type_entry_matches(single, instance),
        };
        if matches {
            Ok(())
        } else {
            let mut expected_text = serde_json::to_string(expected).unwrap_or_default();
            expected_text.retain(|character| character != '"');
            Err(ReportError::SchemaType {
                path: path.into(),
                expected: expected_text.into(),
                found: json_type_name(instance).into(),
            })
        }
    }

    fn child_path(parent: &str, key: &str) -> Box<str> {
        let mut path = parent.to_owned();
        path.push('.');
        path.push_str(key);
        path.into()
    }

    fn check_object_keywords(
        schema: &Value,
        instance: &Value,
        path: &str,
    ) -> Result<(), ReportError> {
        let Some(instance_object) = instance.as_object() else {
            // Non-object instances have nothing for the object keywords to
            // check; `type` has already policed whether that is legal.
            return Ok(());
        };

        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for entry in required {
                if let Some(property) = entry.as_str()
                    && !instance_object.contains_key(property)
                {
                    return Err(ReportError::SchemaRequired {
                        path: path.into(),
                        property: property.into(),
                    });
                }
            }
        }

        let properties = schema.get("properties").and_then(Value::as_object);

        if let Some(additional) = schema.get("additionalProperties") {
            match additional {
                Value::Bool(true) => {}
                Value::Bool(false) => {
                    for key in instance_object.keys() {
                        let declared =
                            properties.is_some_and(|entries| entries.contains_key(key.as_str()));
                        if !declared {
                            return Err(ReportError::SchemaUnexpectedProperty {
                                path: path.into(),
                                property: key.as_str().into(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ReportError::SchemaUnimplementedKeyword {
                        path: path.into(),
                        keyword: "additionalProperties (non-boolean form)".into(),
                    });
                }
            }
        }

        if let Some(entries) = properties {
            for (key, subschema) in entries {
                if let Some(child) = instance_object.get(key.as_str()) {
                    validate(subschema, child, &child_path(path, key))?;
                }
            }
        }
        Ok(())
    }

    fn check_scalar_bounds(
        schema: &Value,
        instance: &Value,
        path: &str,
    ) -> Result<(), ReportError> {
        if let Some(minimum) = schema.get("minimum").and_then(Value::as_u64)
            && let Some(number) = instance.as_number()
        {
            let below = match number.as_u64() {
                Some(value) => value < minimum,
                None => number.as_f64().is_some_and(|value| value < minimum as f64),
            };
            if below {
                return Err(ReportError::SchemaMinimum {
                    path: path.into(),
                    minimum,
                });
            }
        }

        if let Some(min_length) = schema.get("minLength").and_then(Value::as_u64)
            && let Some(text) = instance.as_str()
            && (text.chars().count() as u64) < min_length
        {
            return Err(ReportError::SchemaMinLength {
                path: path.into(),
                min_length,
            });
        }

        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
            && let Some(text) = instance.as_str()
        {
            let regex =
                regex_lite::Regex::new(pattern).map_err(|_| ReportError::SchemaBadPattern {
                    path: path.into(),
                    pattern: pattern.into(),
                })?;
            if !regex.is_match(text) {
                return Err(ReportError::SchemaPattern {
                    path: path.into(),
                    pattern: pattern.into(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> BenchReport<'static> {
        BenchReport {
            bench_id: "unit-sample",
            fixture: "synthetic:unit",
            wall_ns: WallNs {
                p50: 1200,
                p95: 3400,
            },
            allocs: Some(7),
            alloc_bytes_peak: Some(4096),
            rss_peak_bytes: None,
            harness_version: "0.0.0-test",
        }
    }

    #[test]
    fn bench_id_validation_is_exact() {
        assert!(validate_bench_id("armature_parse.small-01").is_ok());
        let error = validate_bench_id("bad id").expect_err("space must be rejected");
        assert_eq!(
            error.to_string(),
            "bench id `bad id` has characters outside [A-Za-z0-9._-]"
        );
        let error = validate_bench_id("").expect_err("empty id must be rejected");
        assert_eq!(
            error.to_string(),
            "bench id `` has characters outside [A-Za-z0-9._-]"
        );
    }

    #[test]
    fn workspace_root_contains_this_crate() {
        let root = workspace_root().expect("workspace root must resolve from a member");
        assert!(root.join(SCHEMA_SUBPATH).is_file());
    }

    #[test]
    fn written_report_bytes_are_exact_and_schema_valid() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_to_dir(dir.path(), &sample_report()).expect("write must succeed");
        assert_eq!(path, dir.path().join("unit-sample.json"));
        let written = fs::read_to_string(&path).expect("written file must be readable");
        let expected = concat!(
            "{\n",
            "  \"bench_id\": \"unit-sample\",\n",
            "  \"fixture\": \"synthetic:unit\",\n",
            "  \"wall_ns\": {\n",
            "    \"p50\": 1200,\n",
            "    \"p95\": 3400\n",
            "  },\n",
            "  \"allocs\": 7,\n",
            "  \"alloc_bytes_peak\": 4096,\n",
            "  \"rss_peak_bytes\": null,\n",
            "  \"harness_version\": \"0.0.0-test\"\n",
            "}\n",
        );
        assert_eq!(written, expected);
        let value: serde_json::Value =
            serde_json::from_str(&written).expect("written report must parse");
        validate_against_schema(&value).expect("written report must satisfy the schema");
    }

    #[test]
    fn schema_rejects_missing_required_property() {
        let mut value = serde_json::to_value(sample_report()).expect("serialize");
        value
            .as_object_mut()
            .expect("report is an object")
            .remove("allocs");
        let error = validate_against_schema(&value).expect_err("missing key must fail");
        assert_eq!(
            error.to_string(),
            "schema violation at `$`: missing required property `allocs`"
        );
    }

    #[test]
    fn schema_rejects_wrong_type() {
        let mut value = serde_json::to_value(sample_report()).expect("serialize");
        value
            .as_object_mut()
            .expect("report is an object")
            .insert("allocs".into(), serde_json::Value::from("seven"));
        let error = validate_against_schema(&value).expect_err("wrong type must fail");
        assert_eq!(
            error.to_string(),
            "schema violation at `$.allocs`: expected [integer,null], found string"
        );
    }

    #[test]
    fn schema_rejects_unexpected_property() {
        let mut value = serde_json::to_value(sample_report()).expect("serialize");
        value
            .as_object_mut()
            .expect("report is an object")
            .insert("extra".into(), serde_json::Value::from(1));
        let error = validate_against_schema(&value).expect_err("extra key must fail");
        assert_eq!(
            error.to_string(),
            "schema violation at `$`: unexpected property `extra`"
        );
    }

    #[test]
    fn schema_rejects_nested_violations_with_paths() {
        let mut value = serde_json::to_value(sample_report()).expect("serialize");
        value["wall_ns"]["p95"] = serde_json::Value::from(-1);
        let error = validate_against_schema(&value).expect_err("negative p95 must fail");
        assert_eq!(
            error.to_string(),
            "schema violation at `$.wall_ns.p95`: value is below minimum 0"
        );
    }

    #[test]
    fn schema_rejects_pattern_violation() {
        let mut value = serde_json::to_value(sample_report()).expect("serialize");
        value["bench_id"] = serde_json::Value::from("bad id");
        let error = validate_against_schema(&value).expect_err("pattern must fail");
        assert_eq!(
            error.to_string(),
            "schema violation at `$.bench_id`: string does not match pattern `^[A-Za-z0-9._-]+$`"
        );
    }

    #[test]
    fn validator_rejects_unimplemented_keywords() {
        let schema: serde_json::Value =
            serde_json::from_str("{\"type\": \"object\", \"maxProperties\": 3}")
                .expect("literal schema parses");
        let instance = serde_json::json!({});
        let error = super::schema_check::validate(&schema, &instance, "$")
            .expect_err("unknown keyword must fail");
        assert_eq!(
            error.to_string(),
            "schema keyword `maxProperties` at `$` is not implemented by the davinci_harness validator"
        );
    }

    #[test]
    fn invalid_bench_id_never_reaches_disk() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut report = sample_report();
        report.bench_id = "../escape";
        let error = write_to_dir(dir.path(), &report).expect_err("traversal id must be rejected");
        assert_eq!(
            error.to_string(),
            "bench id `../escape` has characters outside [A-Za-z0-9._-]"
        );
        let entries = fs::read_dir(dir.path()).expect("tempdir listing").count();
        assert_eq!(entries, 0);
    }
}

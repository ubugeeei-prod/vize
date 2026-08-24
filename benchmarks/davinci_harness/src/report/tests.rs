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
    let error = davinci_test_support::schema::validate(&schema, &instance, "$")
        .expect_err("unknown keyword must fail");
    assert_eq!(
        error.to_string(),
        "schema keyword `maxProperties` at `$` is not implemented by this validator"
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

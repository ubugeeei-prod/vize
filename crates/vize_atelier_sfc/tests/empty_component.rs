#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "whole original-source compiler diagnostics and module regression evidence"
)]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use vize_atelier_sfc::{SfcCompileOptions, SfcError, SfcParseOptions, compile_sfc, parse_sfc};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    file: String,
    filename: String,
    issue: u32,
    original: bool,
    expect: String,
    bytes: usize,
    sha256: String,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/sfc-entry-validation-11262-11263")
}

fn cases() -> Vec<Case> {
    let raw = std::fs::read(fixture_root().join("cases.json")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&raw)),
        "a16ca0e07eaefff3925e7f6a015b6e22396034c4915a6ed518ecc7cc7f9e4710"
    );
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let cases: Vec<Case> = serde_json::from_value(value.get("cases").unwrap().clone()).unwrap();
    assert_eq!(cases.len(), 20);
    assert_eq!(cases.iter().filter(|case| case.original).count(), 8);
    for issue in [11262, 11263] {
        assert_eq!(cases.iter().filter(|case| case.issue == issue).count(), 4);
    }
    cases
}

fn source(case: &Case) -> String {
    let raw = std::fs::read(fixture_root().join(&case.file)).unwrap();
    assert_eq!(raw.len(), case.bytes);
    assert_eq!(format!("{:x}", Sha256::digest(&raw)), case.sha256);
    String::from_utf8(raw).unwrap()
}

fn expected_error(case: &Case, source: &str) -> serde_json::Value {
    if case.expect == "missing-block" {
        return serde_json::json!({
            "message": "At least one <template> or <script> is required in a single file component.",
            "code": "SFC_MISSING_BLOCK",
            "loc": null
        });
    }
    assert_eq!(case.expect, "export-error");
    let start = source.find("export").unwrap();
    let end = source.rfind("\n</script>").unwrap();
    let position = |offset: usize| {
        let prefix = source.get(..offset).unwrap();
        (
            prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
            prefix.rsplit('\n').next().unwrap().encode_utf16().count() + 1,
        )
    };
    let (start_line, start_column) = position(start);
    let (end_line, end_column) = position(end);
    serde_json::json!({
        "message": "<script setup> cannot contain ES module exports.",
        "code": "SCRIPT_SETUP_EXPORT",
        "loc": { "start": start, "end": end, "tag_start": start, "tag_end": end,
            "start_line": start_line, "start_column": start_column,
            "end_line": end_line, "end_column": end_column }
    })
}

#[test]
fn original_invalid_inputs_fail_before_emission_in_both_compiler_modes() {
    for case in cases().into_iter().filter(|case| case.expect != "valid") {
        let source = source(&case);
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
        for vapor in [false, true] {
            let mut options = SfcCompileOptions::default();
            options.vapor = vapor;
            for _ in 0..2 {
                let error = compile_sfc(&descriptor, options.clone()).unwrap_err();
                let whole = serde_json::to_value(error).unwrap();
                println!("sfc-entry-error {} vapor={vapor} {whole}", case.id);
                assert_eq!(whole, expected_error(&case, &source), "{}", case.id);
            }
        }
    }
}

#[test]
fn type_only_and_ordinary_script_or_template_controls_still_emit_parseable_javascript() {
    for case in cases().into_iter().filter(|case| case.expect == "valid") {
        let source = source(&case);
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
        for vapor in [false, true] {
            let mut options = SfcCompileOptions::default();
            options.vapor = vapor;
            let first = compile_sfc(&descriptor, options.clone()).unwrap();
            assert!(first.errors.is_empty(), "{}", case.id);
            assert!(!first.code.is_empty(), "{}", case.id);
            let allocator = oxc_allocator::Allocator::default();
            let parsed =
                oxc_parser::Parser::new(&allocator, &first.code, oxc_span::SourceType::mjs())
                    .parse();
            assert!(!parsed.panicked, "{}", case.id);
            assert!(
                parsed.diagnostics.is_empty(),
                "{}: {:?}",
                case.id,
                parsed.diagnostics
            );
            let repeated = compile_sfc(&descriptor, options).unwrap();
            assert_eq!(format!("{first:?}"), format!("{repeated:?}"), "{}", case.id);
            println!("sfc-entry-valid {} vapor={vapor} {first:#?}", case.id);
        }
    }
}

#[test]
fn empty_descriptor_parsing_remains_tolerant_for_editor_inputs() {
    for source in [
        "",
        "\n",
        "<!-- just a comment -->",
        "<style>.a{color:red}</style>",
    ] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        assert_eq!(descriptor.source.as_ref(), source);
        let error = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap_err();
        assert_eq!(error.code.as_deref(), Some("SFC_MISSING_BLOCK"));
    }
}

#[test]
fn setup_export_errors_keep_original_utf8_offsets_and_utf16_columns() {
    let source = "<!-- 雪🌸 -->\r\n<script setup lang=\"ts\">\r\nexport const n = 1;\r\n</script>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let SfcError { code, loc, .. } =
        compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap_err();
    assert_eq!(code.as_deref(), Some("SCRIPT_SETUP_EXPORT"));
    let loc = loc.unwrap();
    assert_eq!(source.get(loc.start..loc.end), Some("export const n = 1;"));
    assert_eq!((loc.start_line, loc.start_column), (3, 1));
    assert_eq!((loc.end_line, loc.end_column), (3, 19));
}

#[test]
fn artifact_erasure_keeps_export_errors_on_original_setup_text() {
    use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
    use vize_atelier_sfc::{
        SfcCompileExperimentalOptions, SfcScriptOutputMode,
        compile_sfc_for_adapter_with_nuxt_page_meta,
    };

    let source =
        "<script setup>\ndefinePageMeta({ title: '雪🌸' });\nexport const n = 1;\n</script>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    for source_map in [false, true] {
        for vapor in [false, true] {
            let mut options = SfcCompileOptions::default();
            options.vapor = vapor;
            let error = compile_sfc_for_adapter_with_nuxt_page_meta(
                &descriptor,
                options,
                TemplateSyntaxMode::Standard,
                Default::default(),
                CodegenOptions {
                    source_map,
                    ..Default::default()
                },
                SfcScriptOutputMode::SeparateTemplate,
                SfcCompileExperimentalOptions::default(),
            )
            .unwrap_err();
            let start = source.find("export").unwrap();
            let end = source.rfind("\n</script>").unwrap();
            assert_eq!(
                serde_json::to_value(error).unwrap(),
                serde_json::json!({
                    "message": "<script setup> cannot contain ES module exports.",
                    "code": "SCRIPT_SETUP_EXPORT",
                    "loc": { "start": start, "end": end, "tag_start": start, "tag_end": end,
                        "start_line": 3, "start_column": 1, "end_line": 3, "end_column": 19 }
                })
            );
        }
    }
}

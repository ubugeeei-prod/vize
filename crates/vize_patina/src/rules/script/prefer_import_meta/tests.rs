use super::{META, PreferImportMeta};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule};
use serde_json::Value;
use vize_carton::{String, ToCompactString};

const CORPUS: &str = include_str!(
    "../../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/corpus.json"
);
const RECORDING: &str = include_str!(
    "../../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/nuxt-eslint-output.json"
);

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(PreferImportMeta));
    linter.lint(source, 0)
}

fn apply_all_fixes(source: &str, result: &ScriptLintResult) -> String {
    let mut edits = result
        .diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));

    let mut fixed = source.to_compact_string();
    for edit in edits {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    fixed
}

fn line_column(source: &str, offset: usize) -> (u64, u64) {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u64 + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, tail)| tail)
        .len() as u64
        + 1;
    (line, column)
}

#[test]
fn metadata_matches_the_nuxt_rule_contract() {
    let rule = PreferImportMeta;
    let meta = rule.meta();
    assert_eq!(meta.name, "nuxt/prefer-import-meta");
    assert_eq!(
        meta.description,
        "Prefer using `import.meta.*` over `process.*`"
    );
    assert_eq!(meta.default_severity, Severity::Error);
}

#[test]
fn exact_diagnostic_and_fix_contract() {
    let source = "const enabled = process.client";
    let result = lint(source);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);

    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.message,
        "Replace `process.client` with `import.meta.client`."
    );
    assert_eq!((diagnostic.start, diagnostic.end), (16, 30));
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("Use `import.meta.client` instead.")
    );

    let fix = diagnostic.fix.as_ref().unwrap();
    assert_eq!(fix.message, "Replace with `import.meta.client`");
    assert_eq!(fix.edits.len(), 1);
    assert_eq!((fix.edits[0].start, fix.edits[0].end), (16, 30));
    assert_eq!(fix.edits[0].new_text, "import.meta.client");
    assert_eq!(fix.apply(source), "const enabled = import.meta.client");
}

#[test]
fn diagnostic_offsets_include_the_sfc_block_offset() {
    let source = "process.server";
    let mut result = ScriptLintResult::default();
    PreferImportMeta.check(source, 41, &mut result);
    let diagnostic = &result.diagnostics[0];
    assert_eq!((diagnostic.start, diagnostic.end), (41, 55));
    let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
    assert_eq!((edit.start, edit.end), (41, 55));
}

#[test]
fn matches_the_recorded_nuxt_eslint_plugin_oracle() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let recording: Value = serde_json::from_str(RECORDING).unwrap();
    let cases = corpus["preferImportMetaCases"].as_array().unwrap();
    let recorded = recording["preferImportMetaCases"].as_object().unwrap();

    for case in cases {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let upstream = &recorded[id];
        let expected_messages = upstream["messages"].as_array().unwrap();
        let result = lint(source);

        assert_eq!(
            result.diagnostics.len(),
            expected_messages.len(),
            "diagnostic count for {id}"
        );
        for (diagnostic, expected) in result.diagnostics.iter().zip(expected_messages) {
            assert_eq!(
                diagnostic.rule_name,
                expected["ruleId"].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(diagnostic.severity, Severity::Error, "{id}");
            assert_eq!(
                diagnostic.message,
                expected["message"].as_str().unwrap(),
                "{id}"
            );

            let range = expected["fix"]["range"].as_array().unwrap();
            let start = range[0].as_u64().unwrap();
            let end = range[1].as_u64().unwrap();
            assert_eq!(
                (u64::from(diagnostic.start), u64::from(diagnostic.end)),
                (start, end),
                "diagnostic range for {id}"
            );
            assert_eq!(
                line_column(source, start as usize),
                (
                    expected["line"].as_u64().unwrap(),
                    expected["column"].as_u64().unwrap()
                ),
                "diagnostic start location for {id}"
            );
            assert_eq!(
                line_column(source, end as usize),
                (
                    expected["endLine"].as_u64().unwrap(),
                    expected["endColumn"].as_u64().unwrap()
                ),
                "diagnostic end location for {id}"
            );
            let fix = diagnostic.fix.as_ref().unwrap();
            assert_eq!(fix.edits.len(), 1, "{id}");
            assert_eq!(
                (u64::from(fix.edits[0].start), u64::from(fix.edits[0].end)),
                (start, end),
                "fix range for {id}"
            );
            assert_eq!(
                fix.edits[0].new_text,
                expected["fix"]["text"].as_str().unwrap(),
                "{id}"
            );
        }

        let fixed = apply_all_fixes(source, &result);
        assert_eq!(
            upstream["fixed"].as_bool().unwrap(),
            !expected_messages.is_empty(),
            "upstream first fix pass for {id}"
        );
        assert_eq!(
            fixed,
            upstream["output"].as_str().unwrap(),
            "fixed output for {id}"
        );
        assert!(
            lint(&fixed).diagnostics.is_empty(),
            "second fix pass for {id}"
        );
        assert_eq!(
            upstream["secondPassMessages"].as_array().unwrap().len(),
            0,
            "upstream second fix pass for {id}"
        );
        assert_eq!(upstream["secondPassFixed"].as_bool(), Some(false), "{id}");
        assert_eq!(upstream["secondPassOutput"], upstream["output"], "{id}");
    }
}

#[test]
fn uses_utf8_byte_offsets() {
    let source = "const cafe = 'é'; process.prerender";
    let result = lint(source);
    let start = source.find("process").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(result.diagnostics[0].end, source.len() as u32);
}

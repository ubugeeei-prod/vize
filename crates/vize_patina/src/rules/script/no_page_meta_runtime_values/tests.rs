use super::{META, NoPageMetaRuntimeValues};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule};
use serde_json::Value;

const CORPUS: &str = include_str!(
    "../../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/corpus.json"
);
const RECORDING: &str = include_str!(
    "../../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/nuxt-eslint-output.json"
);

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoPageMetaRuntimeValues));
    linter.lint(source, 0)
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
    let meta = NoPageMetaRuntimeValues.meta();
    assert_eq!(meta.name, "nuxt/no-page-meta-runtime-values");
    assert_eq!(
        meta.description,
        "Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup"
    );
    assert_eq!(meta.default_severity, Severity::Error);

    let registered = crate::builtin_script_rules()
        .into_iter()
        .find(|entry| entry.name == meta.name)
        .unwrap();
    assert_eq!(registered.category, "Nuxt");
    assert!(!registered.fixable);
    assert_eq!(registered.presets, &["nuxt"]);
}

#[test]
fn exact_diagnostic_is_non_fixable() {
    let source = "definePageMeta({ title: useRoute() })";
    let result = lint(source);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);

    let diagnostic = &result.diagnostics[0];
    let start = source.find("useRoute").unwrap() as u32;
    let end = start + "useRoute()".len() as u32;
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.message,
        "`definePageMeta()` is extracted at build time and runs before component setup. `useRoute()` requires a Nuxt/Vue runtime context that is not available here. Move it inside a `middleware` or `validate` function."
    );
    assert_eq!((diagnostic.start, diagnostic.end), (start, end));
    assert!(diagnostic.fix.is_none());
}

#[test]
fn diagnostic_offsets_include_the_sfc_block_offset() {
    let source = "definePageMeta({ owner: this })";
    let mut result = ScriptLintResult::default();
    NoPageMetaRuntimeValues.check(source, 41, &mut result);
    let local_start = source.find("this").unwrap() as u32;
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        (diagnostic.start, diagnostic.end),
        (41 + local_start, 41 + local_start + 4)
    );
    assert!(diagnostic.fix.is_none());
}

#[test]
fn matches_the_recorded_nuxt_eslint_plugin_oracle() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let recording: Value = serde_json::from_str(RECORDING).unwrap();
    let cases = corpus["noPageMetaRuntimeValuesCases"].as_array().unwrap();
    let recorded = recording["noPageMetaRuntimeValuesCases"]
        .as_object()
        .unwrap();

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
            assert!(diagnostic.fix.is_none(), "{id}");
            assert!(expected["fix"].is_null(), "upstream fixability for {id}");

            let start = expected["range"][0].as_u64().unwrap();
            let end = expected["range"][1].as_u64().unwrap();
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
        }

        assert_eq!(upstream["fixed"].as_bool(), Some(false), "{id}");
        assert_eq!(upstream["output"].as_str(), Some(source), "{id}");
        assert_eq!(upstream["secondPassFixed"].as_bool(), Some(false), "{id}");
        assert_eq!(upstream["secondPassOutput"].as_str(), Some(source), "{id}");
        assert_eq!(
            upstream["secondPassMessageCount"].as_u64(),
            Some(expected_messages.len() as u64),
            "second pass diagnostic count for {id}"
        );
        assert_eq!(
            upstream["secondPassMessagesMatch"].as_bool(),
            Some(true),
            "second pass diagnostics for {id}"
        );

        let second_pass = lint(source);
        assert_eq!(
            second_pass.diagnostics.len(),
            result.diagnostics.len(),
            "Rust second pass for {id}"
        );
    }
}

#[test]
fn uses_utf8_byte_offsets() {
    let source = "definePageMeta({ title: 'é', route: useRoute() })";
    let result = lint(source);
    let start = source.find("useRoute").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(result.diagnostics[0].end, start + "useRoute()".len() as u32);
}

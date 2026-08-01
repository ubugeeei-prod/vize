use super::{META, NuxtConfigKeysOrder};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule};
use serde_json::Value;
use vize_carton::{String, ToCompactString};

const CORPUS: &str = include_str!(
    "../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/corpus.json"
);
const RECORDING: &str = include_str!(
    "../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/nuxt-eslint-output.json"
);

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NuxtConfigKeysOrder));
    linter.lint(source, 0)
}

fn apply_non_overlapping_fixes(source: &str, result: &ScriptLintResult) -> String {
    let mut edits = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.fix.as_ref())
        .flat_map(|fix| fix.edits.iter())
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| (edit.start, edit.end));
    let mut last_end = 0;
    let mut selected = Vec::new();
    for edit in edits {
        if edit.start >= last_end {
            last_end = edit.end;
            selected.push(edit);
        }
    }
    let mut fixed = source.to_compact_string();
    for edit in selected.into_iter().rev() {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    fixed
}

fn fix_until_stable(source: &str) -> String {
    let mut fixed = source.to_compact_string();
    for _ in 0..10 {
        let next = apply_non_overlapping_fixes(&fixed, &lint(&fixed));
        if next == fixed {
            return fixed;
        }
        fixed = next;
    }
    panic!("Nuxt config order fix did not converge");
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
    let meta = NuxtConfigKeysOrder.meta();
    assert_eq!(meta.name, "nuxt/nuxt-config-keys-order");
    assert_eq!(
        meta.description,
        "Prefer recommended order of Nuxt config properties"
    );
    assert_eq!(meta.default_severity, Severity::Error);
}

#[test]
fn exact_single_line_diagnostic_and_fix_contract() {
    let source = "export default defineNuxtConfig({ ssr: true, modules: [] })";
    let result = lint(source);
    assert_eq!(result.error_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.message,
        "Expected config key \"modules\" to come before \"ssr\""
    );
    assert_eq!((diagnostic.start, diagnostic.end), (32, 58));
    assert!(diagnostic.help.is_none());

    let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
    assert_eq!((edit.start, edit.end), (33, 56));
    assert_eq!(edit.new_text, " modules: [], ssr: true,");
    assert_eq!(
        fix_until_stable(source),
        "export default defineNuxtConfig({ modules: [], ssr: true, })"
    );
}

#[test]
fn preserves_the_upstream_comment_and_comma_contract() {
    let source =
        "export default defineNuxtConfig({\n  ssr: true, // ssr\n  // modules\n  modules: []\n})";
    assert_eq!(
        fix_until_stable(source),
        "export default defineNuxtConfig({\n // ssr\n  // modules\n  modules: [],\n  ssr: true,})"
    );
}

#[test]
fn sorts_top_level_and_environment_objects_to_convergence() {
    let source = "export default { ssr: true, $test: { ssr: true, modules: [] }, $production: { build: {}, app: {} }, modules: [] }";
    let result = lint(source);
    assert_eq!(result.diagnostics.len(), 3);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        [
            "Expected config key \"modules\" to come before \"$production\"",
            "Expected config key \"modules\" to come before \"ssr\"",
            "Expected config key \"app\" to come before \"build\"",
        ]
    );
    let fixed = fix_until_stable(source);
    assert_eq!(
        fixed,
        "export default { modules: [], $production: { app: {}, build: {}, }, $test: { modules: [], ssr: true, }, ssr: true, }"
    );
    assert!(lint(&fixed).diagnostics.is_empty());
}

#[test]
fn spreads_are_boundaries_between_sortable_segments() {
    let source = "export default { modules: [], ssr: true, ...base, css: [], app: {}, ...tail, vite: {}, build: {} }";
    assert_eq!(
        fix_until_stable(source),
        "export default { modules: [], ssr: true, ...base, app: {}, css: [], ...tail, build: {}, vite: {}, }"
    );
}

#[test]
fn unknown_and_literal_keys_follow_upstream_collation() {
    assert_eq!(
        fix_until_stable("export default { zebra: 1, Zebra: 2, alpha: 3, Alpha: 4 }"),
        "export default { alpha: 3, Alpha: 4, zebra: 1, Zebra: 2, }"
    );
    assert_eq!(
        fix_until_stable("export default { \"ssr\": true, modules: [], 'app': {}, css: [] }"),
        "export default { modules: [], css: [], 'app': {}, \"ssr\": true, }"
    );
}

#[test]
fn matches_the_recorded_nuxt_eslint_plugin_oracle() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let recording: Value = serde_json::from_str(RECORDING).unwrap();
    let cases = corpus["nuxtConfigKeysOrderCases"].as_array().unwrap();
    let recorded = recording["nuxtConfigKeysOrderCases"].as_object().unwrap();

    for case in cases {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let upstream = &recorded[id];
        let expected_messages = upstream["messages"].as_array().unwrap();
        let result = lint(source);
        assert_eq!(result.diagnostics.len(), expected_messages.len(), "{id}");

        for (diagnostic, expected) in result.diagnostics.iter().zip(expected_messages) {
            assert_eq!(diagnostic.rule_name, "nuxt/nuxt-config-keys-order", "{id}");
            assert_eq!(diagnostic.severity, Severity::Error, "{id}");
            assert_eq!(
                diagnostic.message,
                expected["message"].as_str().unwrap(),
                "{id}"
            );

            let range = expected["range"].as_array().unwrap();
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
                "diagnostic start for {id}"
            );

            let fix = diagnostic.fix.as_ref().unwrap();
            assert_eq!(fix.edits.len(), 1, "{id}");
            let expected_fix = &expected["fix"];
            let fix_range = expected_fix["range"].as_array().unwrap();
            assert_eq!(
                (u64::from(fix.edits[0].start), u64::from(fix.edits[0].end)),
                (
                    fix_range[0].as_u64().unwrap(),
                    fix_range[1].as_u64().unwrap()
                ),
                "fix range for {id}"
            );
            assert_eq!(
                fix.edits[0].new_text,
                expected_fix["text"].as_str().unwrap(),
                "fix text for {id}"
            );
        }

        let fixed = fix_until_stable(source);
        assert_eq!(
            fixed,
            upstream["output"].as_str().unwrap(),
            "output for {id}"
        );
        assert_eq!(
            upstream["fixed"].as_bool().unwrap(),
            !expected_messages.is_empty(),
            "first fix status for {id}"
        );
        assert!(lint(&fixed).diagnostics.is_empty(), "idempotence for {id}");
        assert_eq!(upstream["secondPassMessageCount"], 0, "{id}");
        assert_eq!(upstream["secondPassFixed"], false, "{id}");
        assert_eq!(upstream["secondPassOutput"], upstream["output"], "{id}");
    }
}

#[test]
fn only_direct_default_export_shapes_are_inspected() {
    for source in [
        "const config={ssr:true,modules:[]}; export default config",
        "export default wrap(extra, { ssr: true, modules: [] })",
        "const config = defineNuxtConfig({ ssr: true, modules: [] })",
        "export const config = { ssr: true, modules: [] }",
    ] {
        assert!(lint(source).diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn diagnostic_and_fix_offsets_include_the_sfc_block_offset() {
    let source = "export default { ssr: true, modules: [] }";
    let mut result = ScriptLintResult::default();
    NuxtConfigKeysOrder.check(source, 41, &mut result);
    let diagnostic = &result.diagnostics[0];
    assert_eq!((diagnostic.start, diagnostic.end), (56, 82));
    let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
    assert_eq!((edit.start, edit.end), (57, 80));
}

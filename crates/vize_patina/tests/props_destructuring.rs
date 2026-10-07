//! Legacy props style corpus: default and every configured mode share dispatch.

#![expect(
    clippy::panic,
    reason = "invalid corpus modes fail the fixture contract"
)]

use serde::Deserialize;
use vize_l0::{String, config::PropsDestructureMode};
use vize_patina::{LintPreset, Linter};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    mode: String,
    source: String,
    diagnostics: usize,
}

#[test]
fn props_destructuring_modes_match_corpus() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/props-destructuring/cases.json"))
            .expect("valid props style corpus");
    assert_eq!(cases.len(), 28);
    for case in cases {
        let mut linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["script/define-props-destructuring".into()]));
        let mode = match case.mode.as_str() {
            "only-when-assigned" => Some(PropsDestructureMode::OnlyWhenAssigned),
            "always" => Some(PropsDestructureMode::Always),
            "never" => Some(PropsDestructureMode::Never),
            "default" => None,
            _ => unreachable!("validated fixture mode"),
        };
        if let Some(mode) = mode {
            linter = linter.with_define_props_destructuring(mode);
        }
        let result = linter.lint_sfc(&case.source, "Badge.vue");
        assert_eq!(
            result.diagnostics.len(),
            case.diagnostics,
            "{}: {:?}",
            case.id,
            result.diagnostics
        );
        for diagnostic in result.diagnostics {
            assert_eq!(diagnostic.rule_name, "script/define-props-destructuring");
            assert!(diagnostic.end as usize <= case.source.len());
            assert!(!diagnostic.message.contains("drop reactivity"));
        }
    }
}

#[test]
fn reactive_defaults_pass_both_props_style_rules() {
    let source = "<script setup lang=\"ts\">const { size = 'md' } = defineProps<{ size?: string }>()</script>";
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "script/define-props-destructuring".into(),
        "script/no-with-defaults".into(),
    ]));
    assert!(linter.lint_sfc(source, "Badge.vue").diagnostics.is_empty());
}

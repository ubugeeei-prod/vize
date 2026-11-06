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
    expected: Option<Expected>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    start: u32,
    end: u32,
    message: String,
    help: String,
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
        if let Some(expected) = case.expected {
            let diagnostic = result.diagnostics.first().expect("corpus diagnostic");
            assert_eq!(diagnostic.rule_name, "script/define-props-destructuring");
            assert_eq!(
                (diagnostic.start, diagnostic.end),
                (expected.start, expected.end),
                "{}",
                case.id
            );
            assert_eq!(
                diagnostic.message.as_str(),
                expected.message.as_str(),
                "{}",
                case.id
            );
            assert_eq!(
                diagnostic.help.as_deref(),
                Some(expected.help.as_str()),
                "{}",
                case.id
            );
            assert!(diagnostic.fix.is_none());
            assert_eq!(diagnostic.labels.len(), 0);
            assert_eq!(diagnostic.severity, vize_patina::Severity::Warning);
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

#[test]
fn happy_path_requires_bare_typed_props_destructuring_by_default() {
    let bare = "<script setup lang=\"ts\">defineProps<{ size?: string }>()</script><template><span>badge</span></template>";
    let result = Linter::with_preset(LintPreset::HappyPath).lint_sfc(bare, "Badge.vue");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "script/define-props-destructuring"
    );

    let fixed = "<script setup lang=\"ts\">const { size } = defineProps<{ size?: string }>()</script><template><span>{{ size }}</span></template>";
    assert!(
        Linter::with_preset(LintPreset::HappyPath)
            .lint_sfc(fixed, "Badge.vue")
            .diagnostics
            .is_empty()
    );
    assert!(
        Linter::with_preset(LintPreset::HappyPath)
            .with_disabled_rules(vec!["script/define-props-destructuring".into()])
            .lint_sfc(bare, "Badge.vue")
            .diagnostics
            .is_empty()
    );
}

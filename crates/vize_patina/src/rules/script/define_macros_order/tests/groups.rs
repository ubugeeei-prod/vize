use super::{NOT_AT_TOP, lint, lint_with_order};
use crate::{
    diagnostic::Severity,
    rules::script::{DefineMacrosOrder, ScriptLinter},
};

const ORDER: &[&str] = &["defineOptions", "defineModel", "defineProps", "defineEmits"];

#[test]
fn every_occurrence_belongs_to_its_configured_group() {
    for source in [
        "const a = defineModel<string>('a'); const b = defineModel<string>('b'); const p = defineProps();",
        "defineOptions({}); const a = defineModel(); const b = defineModel(); defineProps(); defineEmits();",
        "defineModel(); defineModel(); defineProps(); defineProps(); defineEmits(); defineEmits();",
        "const { value: a } = defineModel('a'); const { value: b } = defineModel('b'); defineProps();",
        "const a = withDefaults(defineModel(), {}); const b = withDefaults(defineModel(), {}); defineProps();",
        "import type { Props } from './types'; type T = Props; defineModel(); /* group */ defineModel(); defineProps<T>();",
    ] {
        assert!(lint_with_order(source, ORDER).is_empty(), "{source}");
    }
    assert!(lint("defineProps(); defineProps(); defineEmits(); defineEmits();").is_empty());
    assert!(lint_with_order("defineModel(); defineModel();", &["defineModel"]).is_empty());
}

#[test]
fn split_groups_still_report_the_first_configured_violation() {
    for (source, expected) in [
        (
            "defineModel(); defineProps(); defineModel();",
            "`defineModel` should be above `defineProps`.".to_owned(),
        ),
        (
            "defineProps(); defineModel(); defineModel();",
            "`defineModel` should be above `defineProps`.".to_owned(),
        ),
        (
            "defineModel(); ordinary(); defineModel(); defineProps();",
            format!("`defineModel` {NOT_AT_TOP}"),
        ),
        (
            "defineModel(); defineModel(); ordinary(); defineProps();",
            format!("`defineProps` {NOT_AT_TOP}"),
        ),
        (
            "defineModel(); type T = string; defineModel();",
            format!("`defineModel` {NOT_AT_TOP}"),
        ),
        (
            "defineModel(); defineModel(); defineSlots(); defineProps();",
            format!("`defineProps` {NOT_AT_TOP}"),
        ),
    ] {
        assert_eq!(lint_with_order(source, ORDER), [expected], "{source}");
    }
    assert_eq!(
        lint("defineModel(); defineModel(); defineProps();"),
        [format!("`defineProps` {NOT_AT_TOP}")]
    );
}

#[test]
fn empty_unknown_and_project_orders_keep_their_meaning() {
    assert!(lint_with_order("defineModel(); ordinary(); defineProps();", &[]).is_empty());
    assert!(
        lint_with_order(
            "defineModel(); defineModel(); defineProps();",
            &["defineMissing", "defineModel", "defineProps"]
        )
        .is_empty()
    );
    assert!(
        lint_with_order(
            "definePage(); definePage(); defineModel(); defineModel(); defineProps();",
            &["definePage", "defineModel", "defineProps"]
        )
        .is_empty()
    );
    assert_eq!(
        lint_with_order(
            "defineModel(); definePage(); definePage();",
            &["definePage", "defineModel"]
        ),
        ["`definePage` should be above `defineModel`."]
    );
    // The typed API may bypass the unique-name configuration validator.
    assert!(
        lint_with_order(
            "defineModel(); defineModel(); defineProps();",
            &["defineModel", "defineModel", "defineProps"]
        )
        .is_empty()
    );
}

#[test]
fn group_violation_preserves_the_complete_diagnostic_and_byte_coordinates() {
    for newline in ["\n", "\r\n"] {
        let source = [
            "// 🦀 authored prefix",
            "const first = defineModel();",
            "const props = defineProps();",
            "const second = defineModel();",
        ]
        .join(newline);
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefineMacrosOrder::configured(
            vize_l0::config::DefineMacrosOrderOptions {
                order: ORDER.iter().map(|name| (*name).into()).collect(),
            },
        )));
        let result = linter.lint(&source, 41);
        assert_eq!((result.error_count, result.warning_count), (0, 1));
        let [diagnostic] = result.diagnostics.as_slice() else {
            panic!("one group violation")
        };
        let start = 41 + source.find("const second").unwrap() as u32;
        let end = start + "const second = defineModel();".len() as u32;
        assert_eq!(diagnostic.rule_name, "script/define-macros-order");
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(
            diagnostic.message,
            "`defineModel` should be above `defineProps`."
        );
        assert_eq!((diagnostic.start, diagnostic.end), (start, end));
        assert_eq!(
            diagnostic.help.as_deref(),
            Some(
                "Declare `defineProps` and then `defineEmits` first in `<script setup>`, right after imports and type declarations."
            )
        );
        assert!(diagnostic.fix.is_none());
        let [label] = diagnostic.labels.as_slice() else {
            panic!("one original label")
        };
        assert_eq!(label.message, "compiler macro out of order");
        assert_eq!((label.start, label.end), (start, end));
    }
}

#[test]
fn bounded_large_groups_emit_zero_or_one_diagnostic() {
    for count in [1, 2, 64, 4096] {
        let source = "defineModel();\n".repeat(count);
        let valid = lint_with_order(&source, ORDER);
        assert!(valid.is_empty());
        let invalid = lint_with_order(&format!("{source}ordinary();\ndefineModel();"), ORDER);
        assert_eq!(invalid, [format!("`defineModel` {NOT_AT_TOP}")]);
        eprintln!(
            "macro-group-statements={count}, valid-diagnostics={}, barrier-diagnostics={}",
            valid.len(),
            invalid.len()
        );
    }
}

//! #12982: `allowNullish: false` inspects union constituents while preserving
//! the default `true` behavior and every unrelated diagnostic packet.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintDiagnostic, LintPreset, Linter, Severity};
use serde_json::json;
use vize_l0::config::TypeAwareRuleOptions;

const RULE: &str = "type/restrict-plus-operands";
const SOURCE: &str = r##"<script setup lang="ts">
declare const label: string | undefined;
declare const maybeNull: string | null;
declare const count: number | undefined;
declare const flag: boolean;

const a = label + "!";
const b = "#" + maybeNull;
const c = "n=" + count;
let total = "";
total += label;
function suffix(text?: string): string {
  return text + "!";
}
const d = "on: " + flag;
</script>"##;

fn configured(allow_nullish: bool) -> Linter {
    let options: TypeAwareRuleOptions = serde_json::from_value(json!({
        (RULE): {
            "allowAny": false,
            "allowBoolean": false,
            "allowNullish": allow_nullish,
            "allowNumberAndString": true,
            "allowRegExp": false,
            "skipCompoundAssignments": false
        }
    }))
    .expect("type-aware options");
    Linter::with_preset(LintPreset::Opinionated).with_type_aware_rule_options(options)
}

fn at(needle: &str, prefix: &str) -> u32 {
    let statement = SOURCE.find(needle).expect("statement");
    u32::try_from(statement + prefix.len()).expect("fixture range")
}

fn target(result: &[LintDiagnostic]) -> Vec<&LintDiagnostic> {
    let mut diagnostics = result
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .collect::<Vec<_>>();
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    diagnostics
}

fn packet(diagnostic: &LintDiagnostic) -> (Severity, &str, u32, u32, Option<&str>, usize, bool) {
    (
        diagnostic.severity,
        diagnostic.message.as_str(),
        diagnostic.start,
        diagnostic.end,
        diagnostic.help.as_deref(),
        diagnostic.labels.len(),
        diagnostic.fix.is_some(),
    )
}

fn complete_packet(diagnostic: &LintDiagnostic) -> serde_json::Value {
    json!({
        "ruleName": diagnostic.rule_name,
        "severity": diagnostic.severity,
        "message": diagnostic.message,
        "start": diagnostic.start,
        "end": diagnostic.end,
        "help": diagnostic.help,
        "labels": diagnostic
            .labels
            .iter()
            .map(|label| json!({
                "message": label.message,
                "start": label.start,
                "end": label.end,
            }))
            .collect::<Vec<_>>(),
        "fix": diagnostic.fix.as_ref().map(|fix| json!({
            "message": fix.message,
            "edits": fix.edits,
        })),
    })
}

#[test]
fn restrict_plus_matches_the_reported_nullish_option_packets() {
    if !corsa_available() {
        return;
    }
    let strict = lint_sfc_with_corsa(&configured(false), SOURCE, "Plus.vue");
    let permissive = lint_sfc_with_corsa(&configured(true), SOURCE, "Plus.vue");

    let label = at("const a = label", "const a = ");
    let maybe_null = at("const b = \"#\" + maybeNull", "const b = \"#\" + ");
    let count = at("const c = \"n=\" + count", "const c = \"n=\" + ");
    let assigned_label = at("total += label", "total += ");
    let parameter = at("return text + \"!\"", "return ");
    let boolean = at("const d = \"on: \" + flag", "const d = \"on: \" + ");
    let expected = [
        (
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `string | undefined`.",
            label,
            label + 5,
        ),
        (
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `string | null`.",
            maybe_null,
            maybe_null + 9,
        ),
        (
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `number | undefined`.",
            count,
            count + 5,
        ),
        (
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `string | undefined`.",
            assigned_label,
            assigned_label + 5,
        ),
        (
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `string | undefined`.",
            parameter,
            parameter + 4,
        ),
        (
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `boolean`.",
            boolean,
            boolean + 4,
        ),
    ];
    let actual = target(&strict.diagnostics);
    assert_eq!(actual.len(), expected.len(), "{:#?}", strict.diagnostics);
    for (diagnostic, (message, start, end)) in actual.into_iter().zip(expected) {
        assert_eq!(
            packet(diagnostic),
            (Severity::Warning, message, start, end, None, 0, false)
        );
    }

    let permissive_target = target(&permissive.diagnostics);
    assert_eq!(permissive_target.len(), 1, "{:#?}", permissive.diagnostics);
    assert_eq!(permissive_target[0].start, boolean);
    assert_eq!(permissive_target[0].end, boolean + 4);

    let unrelated = |diagnostics: &[LintDiagnostic]| {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name != RULE)
            .map(complete_packet)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        unrelated(&strict.diagnostics),
        unrelated(&permissive.diagnostics),
        "the option must not change another rule's packet"
    );
}

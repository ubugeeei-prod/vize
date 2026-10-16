use serde_json::Value;

use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::{child_list, rule_option_bool};

/// Type-aware rule requiring `switch` statements to exhaustively cover every
/// member of the discriminant's type (and flagging unnecessary `default`
/// clauses on switches that are already exhaustive).
///
/// Faithful port of the tsgolint Go rule `switch-exhaustiveness-check`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SwitchExhaustivenessCheckRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "switchIsNotExhaustive",
        description: "Switch is not exhaustive. Cases not matched: {{missingBranches}}",
    },
    RuleMessage {
        id: "dangerousDefaultCase",
        description: "The switch statement is exhaustive, so the default case is unnecessary.",
    },
    RuleMessage {
        id: "addMissingCases",
        description: "Add branches for missing cases.",
    },
];

const LISTENERS: &[&str] = &["SwitchStatement"];

impl RustLintRule for SwitchExhaustivenessCheckRule {
    fn name(&self) -> &'static str {
        "switch-exhaustiveness-check"
    }

    fn docs_description(&self) -> &'static str {
        "Require switch statements over unions to cover every case."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "SwitchStatement" {
            return;
        }
        let Some(discriminant) = node.child("discriminant") else {
            return;
        };

        let metadata = switch_metadata(node);

        // Options. Note the upstream default for `allowDefaultCaseForExhaustiveSwitch`
        // is `true`, while the other two booleans default to `false`.
        let allow_default_case_for_exhaustive_switch =
            rule_option_bool(node, "allowDefaultCaseForExhaustiveSwitch").unwrap_or(true);
        let consider_default_exhaustive_for_unions = rule_option_bool(node, "considerDefaultExhaustiveForUnions").unwrap_or(false)
                // `defaultCaseCommentPattern`: a default case whose leading
                // comment matches the configured pattern also satisfies
                // exhaustiveness for unions. The host evaluates the pattern
                // (a JS regular expression) and reports the raw match.
                || (has_default_case_comment_pattern(node)
                    && node.field_bool("__defaultCaseCommentMatches") == Some(true));
        let require_default_for_non_union =
            rule_option_bool(node, "requireDefaultForNonUnion").unwrap_or(false);

        // checkSwitchExhaustive
        //
        // The decision (report iff there is at least one uncovered literal
        // branch, unless `considerDefaultExhaustiveForUnions` + a default case
        // suppress it) stays entirely in Rust. The branch *texts* are raw
        // checker renderings supplied host-side and joined with `" | "` by the
        // bridge through the `{{missingBranches}}` placeholder, matching the Go
        // `strings.Join(missingBranches, " | ")`.
        if !(metadata.missing_branch_texts.is_empty()
            || consider_default_exhaustive_for_unions && metadata.has_default_case)
        {
            ctx.report_with_suggestions(
                "switchIsNotExhaustive",
                discriminant.range,
                vec![add_missing_cases_suggestion(node, &metadata, false)],
            );
        }

        // checkSwitchUnnecessaryDefaultCase
        if !allow_default_case_for_exhaustive_switch
            && metadata.missing_branch_texts.is_empty()
            && metadata.has_default_case
            && !metadata.contains_non_literal_type
        {
            if let Some(default_range) = metadata.default_case_range {
                ctx.report("dangerousDefaultCase", default_range);
            }
        }

        // checkSwitchNoUnionDefaultCase
        if require_default_for_non_union
            && metadata.contains_non_literal_type
            && !metadata.has_default_case
        {
            ctx.report_with_suggestions(
                "switchIsNotExhaustive",
                discriminant.range,
                vec![add_missing_cases_suggestion(node, &metadata, true)],
            );
        }
    }
}

/// Mirror of the Go `SwitchMetadata`, assembled from LintNode facts.
struct SwitchMetadata {
    contains_non_literal_type: bool,
    has_default_case: bool,
    default_case_range: Option<TextRange>,
    /// `TypeToString`-rendered texts for missing literal branches (already
    /// computed by the host checker; subtraction of present cases happens
    /// host-side, matching `getSwitchMetadata`).
    missing_branch_texts: Vec<String>,
    /// `buildCaseTest`-rendered case expressions for the same branches, used by
    /// the autofix. Parallel to `missing_branch_texts`.
    missing_case_tests: Vec<String>,
}

fn switch_metadata(node: &LintNode) -> SwitchMetadata {
    let mut has_default_case = false;
    let mut default_case_range = None;
    // Mirror Go's `slices.IndexFunc`, which keeps the *first* default clause.
    for case in child_list(node, "cases") {
        if case.child("test").is_none() {
            has_default_case = true;
            if default_case_range.is_none() {
                default_case_range = Some(case.range);
            }
        }
    }

    SwitchMetadata {
        contains_non_literal_type: node
            .field_bool("__discriminantContainsNonLiteral")
            .unwrap_or(false),
        has_default_case,
        default_case_range,
        missing_branch_texts: string_array_field(node, "__missingBranchTexts"),
        missing_case_tests: string_array_field(node, "__missingBranchCaseTests"),
    }
}

/// Whether the `defaultCaseCommentPattern` option is configured.
fn has_default_case_comment_pattern(node: &LintNode) -> bool {
    node.fields
        .get("__ruleOptions")
        .and_then(serde_json::Value::as_array)
        .and_then(|options| options.first())
        .and_then(|options| options.get("defaultCaseCommentPattern"))
        .and_then(serde_json::Value::as_str)
        .is_some()
}

/// Builds the `addMissingCases` suggestion. When `require_default` is true the
/// only missing branch is a synthetic `default` clause (mirrors the Go path
/// `fixSwitch(..., []*checker.Type{nil}, ...)`).
fn add_missing_cases_suggestion(
    node: &LintNode,
    metadata: &SwitchMetadata,
    require_default: bool,
) -> LintSuggestion {
    let lines: Vec<String> = if require_default {
        vec!["default: { throw new Error('default case') }".to_owned()]
    } else {
        metadata
            .missing_case_tests
            .iter()
            .map(|case_test| {
                let escaped = case_test.replace('\\', "\\\\").replace('\'', "\\'");
                format!(
                    "case {case_test}: {{ throw new Error('Not implemented yet: {escaped} case') }}"
                )
            })
            .collect()
    };

    let fixes = build_fixes(node, metadata, &lines);

    LintSuggestion {
        message_id: "addMissingCases".to_owned(),
        message: "Add branches for missing cases.".to_owned(),
        fixes,
    }
}

/// Re-implements `applyMissingCases`: insert after the last case, before the
/// default case, or replace the whole case block when there are no cases.
fn build_fixes(node: &LintNode, metadata: &SwitchMetadata, lines: &[String]) -> Vec<LintFix> {
    let cases = child_list(node, "cases");
    let indent = case_indent(node, cases);

    let indented: Vec<String> = lines.iter().map(|line| format!("{indent}{line}")).collect();
    let fix_string = indented.join("\n");

    if let Some(last_case) = cases.last() {
        if metadata.has_default_case {
            // Insert before the default clause.
            if let Some(default_range) = metadata.default_case_range {
                let mut before = String::new();
                for line in lines {
                    before.push_str(line);
                    before.push('\n');
                    before.push_str(&indent);
                }
                let insertion = TextRange::new(default_range.start, default_range.start);
                return vec![LintFix::replace_range(insertion, before)];
            }
        }
        // Insert after the last case.
        let insertion = TextRange::new(last_case.range.end, last_case.range.end);
        return vec![LintFix::replace_range(insertion, format!("\n{fix_string}"))];
    }

    // No cases at all: replace the case block contents.
    if let Some(case_block) = case_block_range(node) {
        return vec![LintFix::replace_range(
            case_block,
            format!("{{\n{fix_string}\n{indent}}}"),
        )];
    }
    Vec::new()
}

/// Best-effort indentation: the host may provide `__caseIndent`; otherwise no
/// indentation is applied (the upstream computes column from source text).
fn case_indent(node: &LintNode, _cases: &[LintNode]) -> String {
    node.field_str("__caseIndent").unwrap_or("").to_owned()
}

/// Range of the case block `{ ... }` for whole-block replacement, when the host
/// supplies it via `__caseBlockRange` `[start, end]`.
fn case_block_range(node: &LintNode) -> Option<TextRange> {
    let array = node.fields.get("__caseBlockRange")?.as_array()?;
    let start = array.first()?.as_u64()? as u32;
    let end = array.get(1)?.as_u64()? as u32;
    Some(TextRange::new(start, end))
}

fn string_array_field(node: &LintNode, key: &str) -> Vec<String> {
    node.fields
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, LintRuleRegistry};

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        LintRuleRegistry::with_default_type_aware_rules()
            .run_rule("switch-exhaustiveness-check", node)
            .unwrap()
    }

    /// Helper: a switch node with a discriminant child, a list of case ranges
    /// (None test => default), and host facts.
    fn switch_node(cases: serde_json::Value, fields: serde_json::Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "SwitchStatement",
            "range": { "start": 0, "end": 100 },
            "children": {
                "discriminant": {
                    "kind": "Identifier",
                    "range": { "start": 8, "end": 13 },
                    "fields": { "name": "value" }
                }
            },
            "childLists": { "cases": cases },
            "fields": fields,
        }))
        .unwrap()
    }

    // SHOULD report: union 'a' | 'b' | 'c' with only 'a' covered -> not exhaustive.
    #[test]
    fn reports_non_exhaustive_union() {
        let node = switch_node(
            json!([
                {
                    "kind": "SwitchCase",
                    "range": { "start": 20, "end": 30 },
                    "children": { "test": { "kind": "Literal", "range": { "start": 25, "end": 28 }, "fields": { "value": "a" } } }
                }
            ]),
            json!({
                "__missingBranchTexts": ["\"b\"", "\"c\""],
                "__missingBranchCaseTests": ["\"b\"", "\"c\""],
                "__discriminantContainsNonLiteral": false,
            }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "switchIsNotExhaustive");
        // Reported on the discriminant range.
        assert_eq!(diagnostics[0].range, super::TextRange::new(8, 13));
        assert!(!diagnostics[0].suggestions.is_empty());
        assert_eq!(diagnostics[0].suggestions[0].message_id, "addMissingCases");
    }

    // SHOULD report: dangerousDefaultCase when exhaustive + default present +
    // allowDefaultCaseForExhaustiveSwitch=false and no non-literal type.
    #[test]
    fn reports_dangerous_default_case() {
        let node = switch_node(
            json!([
                {
                    "kind": "SwitchCase",
                    "range": { "start": 20, "end": 30 },
                    "children": { "test": { "kind": "Literal", "range": { "start": 25, "end": 28 }, "fields": { "value": "a" } } }
                },
                {
                    "kind": "SwitchCase",
                    "range": { "start": 40, "end": 60 },
                    "children": {}
                }
            ]),
            json!({
                "__missingBranchTexts": [],
                "__missingBranchCaseTests": [],
                "__discriminantContainsNonLiteral": false,
                "__ruleOptions": [{ "allowDefaultCaseForExhaustiveSwitch": false }],
            }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "dangerousDefaultCase");
        // Reported on the default clause range.
        assert_eq!(diagnostics[0].range, super::TextRange::new(40, 60));
    }

    // SHOULD report: requireDefaultForNonUnion with a non-literal type and no default.
    #[test]
    fn reports_missing_default_for_non_union() {
        let node = switch_node(
            json!([
                {
                    "kind": "SwitchCase",
                    "range": { "start": 20, "end": 30 },
                    "children": { "test": { "kind": "Literal", "range": { "start": 25, "end": 28 }, "fields": { "value": "a" } } }
                }
            ]),
            json!({
                "__missingBranchTexts": [],
                "__missingBranchCaseTests": [],
                "__discriminantContainsNonLiteral": true,
                "__ruleOptions": [{ "requireDefaultForNonUnion": true }],
            }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "switchIsNotExhaustive");
        assert_eq!(diagnostics[0].suggestions[0].message_id, "addMissingCases");
        assert!(
            diagnostics[0].suggestions[0].fixes[0]
                .replacement_text
                .contains("default: { throw new Error('default case') }")
        );
    }

    // SHOULD NOT report: fully exhaustive union, no default, default options.
    #[test]
    fn allows_exhaustive_union() {
        let node = switch_node(
            json!([
                {
                    "kind": "SwitchCase",
                    "range": { "start": 20, "end": 30 },
                    "children": { "test": { "kind": "Literal", "range": { "start": 25, "end": 28 }, "fields": { "value": "a" } } }
                }
            ]),
            json!({
                "__missingBranchTexts": [],
                "__missingBranchCaseTests": [],
                "__discriminantContainsNonLiteral": false,
            }),
        );
        assert!(run(&node).is_empty());
    }

    // SHOULD NOT report: considerDefaultExhaustiveForUnions makes a default-bearing
    // union switch exhaustive even with missing branches.
    #[test]
    fn allows_default_when_considered_exhaustive() {
        let node = switch_node(
            json!([
                {
                    "kind": "SwitchCase",
                    "range": { "start": 20, "end": 30 },
                    "children": { "test": { "kind": "Literal", "range": { "start": 25, "end": 28 }, "fields": { "value": "a" } } }
                },
                {
                    "kind": "SwitchCase",
                    "range": { "start": 40, "end": 60 },
                    "children": {}
                }
            ]),
            json!({
                "__missingBranchTexts": ["\"b\"", "\"c\""],
                "__missingBranchCaseTests": ["\"b\"", "\"c\""],
                "__discriminantContainsNonLiteral": false,
                // allowDefaultCaseForExhaustiveSwitch defaults to true, so no
                // dangerousDefaultCase either.
                "__ruleOptions": [{ "considerDefaultExhaustiveForUnions": true }],
            }),
        );
        assert!(run(&node).is_empty());
    }

    // SHOULD NOT report: default present but allowDefaultCaseForExhaustiveSwitch
    // defaults to true.
    #[test]
    fn allows_default_on_exhaustive_switch_by_default() {
        let node = switch_node(
            json!([
                {
                    "kind": "SwitchCase",
                    "range": { "start": 20, "end": 30 },
                    "children": { "test": { "kind": "Literal", "range": { "start": 25, "end": 28 }, "fields": { "value": "a" } } }
                },
                {
                    "kind": "SwitchCase",
                    "range": { "start": 40, "end": 60 },
                    "children": {}
                }
            ]),
            json!({
                "__missingBranchTexts": [],
                "__missingBranchCaseTests": [],
                "__discriminantContainsNonLiteral": false,
            }),
        );
        assert!(run(&node).is_empty());
    }
}

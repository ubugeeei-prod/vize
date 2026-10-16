use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};

/// Type-aware rule that flags explicit type arguments equal to the type
/// parameter's default, mirroring the tsgolint `no-unnecessary-type-arguments`
/// rule.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryTypeArgumentsRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unnecessaryTypeParameter",
    description: "This is the default value for this type parameter, so it can be omitted.",
}];

// The Go rule registers listeners for ExpressionWithTypeArguments, TypeReference,
// CallExpression, NewExpression, TaggedTemplateExpression, JsxOpeningElement and
// JsxSelfClosingElement. These map onto the TS-ESTree node kinds emitted by the
// host adapter.
const LISTENERS: &[&str] = &[
    "CallExpression",
    "NewExpression",
    "TaggedTemplateExpression",
    "TSTypeReference",
    "TSClassImplements",
    "TSInterfaceHeritage",
    "JSXOpeningElement",
];

impl RustLintRule for NoUnnecessaryTypeArgumentsRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-arguments"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow type arguments that are equal to the default."
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
        check_args_and_parameters(ctx, node);
    }
}

/// Mirrors the Go `checkArgsAndParameters`: it only ever inspects the *last*
/// explicit type argument, because earlier ones must be present if a later one
/// is. When the last argument is type-identical to the matching type
/// parameter's default it is reported with a fix that removes it.
fn check_args_and_parameters(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let argument_ranges = type_argument_ranges(node);
    if argument_ranges.is_empty() {
        return;
    }

    let parameter_count = node
        .field_f64("__typeParameterCount")
        .map(|count| count as usize)
        .unwrap_or(0);
    if parameter_count == 0 {
        return;
    }

    // Just check the last one.
    let last_index = argument_ranges.len() - 1;

    // `if i >= len(parameters) { return }` — too many type arguments.
    if last_index >= parameter_count {
        return;
    }

    // The host only resolves the facts below for the LAST type parameter, which
    // is the one aligned with the last type argument.
    if node.field_bool("__lastTypeParameterHasDefault") != Some(true) {
        return;
    }

    // `utils.IsIntrinsicErrorType(paramType)` / `argType`.
    if node.field_bool("__lastTypeParameterDefaultIsErrorType") == Some(true) {
        return;
    }
    if node.field_bool("__lastTypeArgumentIsErrorType") == Some(true) {
        return;
    }

    // Go bails when:
    //   argType != paramType &&
    //   (isAny(argType) || isAny(paramType) || flags differ || !identical)
    // i.e. it only reports when the two types are the *same* type, or are
    // structurally identical with matching flags and neither is `any`.
    if !last_argument_matches_default(node) {
        return;
    }

    let range = argument_ranges[last_index];
    let removal = removal_range(node, &argument_ranges, last_index);
    ctx.report_with_suggestions(
        "unnecessaryTypeParameter",
        range,
        vec![LintSuggestion {
            message_id: "unnecessaryTypeParameter".to_owned(),
            message: MESSAGES[0].description.to_owned(),
            fixes: vec![LintFix::remove_range(removal)],
        }],
    );
}

/// Reproduces the Go acceptance condition for "this argument is unnecessary".
fn last_argument_matches_default(node: &LintNode) -> bool {
    // `argType == paramType` — the exact same checker type object.
    if node.field_bool("__lastTypeArgumentEqualsDefault") == Some(true) {
        return true;
    }
    // Otherwise require: neither is `any`, identical type flags, and the checker
    // reports the types as identical.
    if node.field_bool("__lastTypeArgumentIsAny") == Some(true) {
        return false;
    }
    if node.field_bool("__lastTypeParameterDefaultIsAny") == Some(true) {
        return false;
    }
    if node.field_bool("__lastTypeArgumentSameTypeFlagsAsDefault") != Some(true) {
        return false;
    }
    node.field_bool("__lastTypeArgumentIdenticalToDefault") == Some(true)
}

/// Reads the explicit type-argument node ranges supplied by the host
/// (`__typeArgumentRanges`).
fn type_argument_ranges(node: &LintNode) -> Vec<TextRange> {
    let Some(array) = node
        .fields
        .get("__typeArgumentRanges")
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };
    array
        .iter()
        .filter_map(|entry| {
            let start = entry.get("start")?.as_u64()? as u32;
            let end = entry.get("end")?.as_u64()? as u32;
            Some(TextRange::new(start, end))
        })
        .collect()
}

/// Computes the byte range to remove so the source remains valid, mirroring the
/// Go fix:
///   - for the only/first argument (`i == 0`): swallow the surrounding `<...>`
///     using the whole type-argument list span (`__typeArgumentListRange`);
///   - otherwise: remove from the end of the previous argument through the end
///     of this argument (dropping the preceding comma).
fn removal_range(node: &LintNode, argument_ranges: &[TextRange], index: usize) -> TextRange {
    if index == 0 {
        if let Some(list) = type_argument_list_range(node) {
            return list;
        }
        return argument_ranges[0];
    }
    let previous_end = argument_ranges[index - 1].end;
    TextRange::new(previous_end, argument_ranges[index].end)
}

/// Reads the full `<...>` span (inclusive of the angle brackets) provided by the
/// host as `__typeArgumentListRange`.
fn type_argument_list_range(node: &LintNode) -> Option<TextRange> {
    let entry = node.fields.get("__typeArgumentListRange")?;
    let start = entry.get("start")?.as_u64()? as u32;
    let end = entry.get("end")?.as_u64()? as u32;
    Some(TextRange::new(start, end))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::NoUnnecessaryTypeArgumentsRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NoUnnecessaryTypeArgumentsRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn node_from(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    // `f<number>()` where `function f<T = number>()`.
    #[test]
    fn reports_single_type_argument_equal_to_default() {
        let node = node_from(json!({
            "kind": "CallExpression",
            "range": { "start": 29, "end": 40 },
            "fields": {
                "__typeParameterCount": 1,
                "__lastTypeParameterHasDefault": true,
                "__lastTypeArgumentEqualsDefault": true,
                "__typeArgumentRanges": [{ "start": 31, "end": 37 }],
                "__typeArgumentListRange": { "start": 30, "end": 38 }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "unnecessaryTypeParameter");
        // Reports on the argument node, not the whole call.
        assert_eq!(diagnostics[0].range.start, 31);
        // Fix removes the whole `<number>` span.
        assert_eq!(diagnostics[0].suggestions[0].fixes[0].range.start, 30);
        assert_eq!(diagnostics[0].suggestions[0].fixes[0].range.end, 38);
    }

    // `g<string, string>()` where `function g<T = number, U = string>()`.
    // Only the last argument matches its default; fix drops the trailing arg + comma.
    #[test]
    fn reports_last_type_argument_and_drops_preceding_comma() {
        let node = node_from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 20 },
            "fields": {
                "__typeParameterCount": 2,
                "__lastTypeParameterHasDefault": true,
                "__lastTypeArgumentEqualsDefault": true,
                "__typeArgumentRanges": [
                    { "start": 2, "end": 8 },
                    { "start": 10, "end": 16 }
                ],
                "__typeArgumentListRange": { "start": 1, "end": 17 }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start, 10);
        // Removal spans from end of first arg through end of last arg.
        assert_eq!(diagnostics[0].suggestions[0].fixes[0].range.start, 8);
        assert_eq!(diagnostics[0].suggestions[0].fixes[0].range.end, 16);
    }

    // `f<string>()` where `function f<T = number>()` — string != number default.
    #[test]
    fn ignores_argument_that_differs_from_default() {
        let node = node_from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 11 },
            "fields": {
                "__typeParameterCount": 1,
                "__lastTypeParameterHasDefault": true,
                "__lastTypeArgumentEqualsDefault": false,
                "__lastTypeArgumentIsAny": false,
                "__lastTypeParameterDefaultIsAny": false,
                "__lastTypeArgumentSameTypeFlagsAsDefault": false,
                "__lastTypeArgumentIdenticalToDefault": false,
                "__typeArgumentRanges": [{ "start": 2, "end": 8 }],
                "__typeArgumentListRange": { "start": 1, "end": 9 }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // `g<number, number>()` where `function g<T = number, U = string>()` — last
    // (number) does not equal default (string), so nothing is reported.
    #[test]
    fn ignores_when_last_argument_does_not_match_default() {
        let node = node_from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 20 },
            "fields": {
                "__typeParameterCount": 2,
                "__lastTypeParameterHasDefault": true,
                "__lastTypeArgumentEqualsDefault": false,
                "__lastTypeArgumentIsAny": false,
                "__lastTypeParameterDefaultIsAny": false,
                "__lastTypeArgumentSameTypeFlagsAsDefault": true,
                "__lastTypeArgumentIdenticalToDefault": false,
                "__typeArgumentRanges": [
                    { "start": 2, "end": 8 },
                    { "start": 10, "end": 16 }
                ],
                "__typeArgumentListRange": { "start": 1, "end": 17 }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // Last type parameter has no default — never report (e.g. `f<string>()`,
    // `function f<T>()`).
    #[test]
    fn ignores_type_parameter_without_default() {
        let node = node_from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 11 },
            "fields": {
                "__typeParameterCount": 1,
                "__lastTypeParameterHasDefault": false,
                "__lastTypeArgumentEqualsDefault": true,
                "__typeArgumentRanges": [{ "start": 2, "end": 8 }],
                "__typeArgumentListRange": { "start": 1, "end": 9 }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // More type arguments than parameters (`OneParam<string, number>` with a
    // single-parameter alias) must not panic and must not report.
    #[test]
    fn ignores_when_more_arguments_than_parameters() {
        let node = node_from(json!({
            "kind": "TSTypeReference",
            "range": { "start": 0, "end": 24 },
            "fields": {
                "__typeParameterCount": 1,
                "__lastTypeParameterHasDefault": true,
                "__lastTypeArgumentEqualsDefault": true,
                "__typeArgumentRanges": [
                    { "start": 9, "end": 15 },
                    { "start": 17, "end": 23 }
                ],
                "__typeArgumentListRange": { "start": 8, "end": 24 }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // `any` argument against `any` default still reports when they are the same
    // type object (Go: `argType == paramType`).
    #[test]
    fn reports_any_argument_equal_to_any_default() {
        let node = node_from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 9 },
            "fields": {
                "__typeParameterCount": 1,
                "__lastTypeParameterHasDefault": true,
                "__lastTypeArgumentEqualsDefault": true,
                "__lastTypeArgumentIsAny": true,
                "__lastTypeParameterDefaultIsAny": true,
                "__typeArgumentRanges": [{ "start": 4, "end": 7 }],
                "__typeArgumentListRange": { "start": 3, "end": 8 }
            }
        }));
        assert_eq!(run(&node).len(), 1);
    }
}

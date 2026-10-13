use serde_json::Value;

use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::child_list;

/// Type-aware rule that disallows type parameters that are used at most once in
/// the signature they belong to.
///
/// This is a faithful port of the tsgolint Go rule
/// `no-unnecessary-type-parameters`. The upstream rule walks the resolved
/// TypeScript type of each function/class declaration with the checker, counts
/// how many times each type-parameter symbol is observed, and reports any type
/// parameter that is used at most once (the constraint walk in
/// `countTypeParameterUsage`). Because the Rust hot path has no TypeScript
/// checker, the raw checker observations (per-parameter usage counts and the
/// AST fast-path "repeated in AST" flag) are supplied as host facts; the rule
/// VERDICT, the message selection, and the suggested-fix construction all live
/// here in Rust.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryTypeParametersRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "sole",
        // Description is filled with the concrete name/uses/descriptor by the
        // host renderer; this static fallback mirrors the upstream template.
        description: "Type parameter is used only once in the signature.",
    },
    RuleMessage {
        id: "replaceUsagesWithConstraint",
        description: "Replace all usages of type parameter with its constraint.",
    },
];

const LISTENERS: &[&str] = &["TSTypeParameterDeclaration"];

impl RustLintRule for NoUnnecessaryTypeParametersRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-parameters"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow type parameters that are not used more than once."
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
        if node.kind != "TSTypeParameterDeclaration" {
            return;
        }

        let params = child_list(node, "params");
        if params.is_empty() {
            return;
        }

        for (index, param) in params.iter().enumerate() {
            check_type_parameter(ctx, node, params, index, param);
        }
    }
}

/// Mirrors the per-type-parameter body of `checkNoUnnecessaryTypeParametersNode`.
fn check_type_parameter(
    ctx: &mut RuleContext<'_>,
    declaration: &LintNode,
    params: &[LintNode],
    index: usize,
    param: &LintNode,
) {
    // AST fast path: `isTypeParameterRepeatedInAST`. The host computes whether
    // the type parameter is referenced two or more times (ignoring references
    // inside the declaration itself and inside the function body).
    if param.field_bool("__repeatedInAst") == Some(true) {
        return;
    }

    // Raw checker observation: `identifierCounts[typeParameterSymbol]`.
    // Missing fact <=> the symbol was not present in `countTypeParameterUsage`
    // (the `!ok` branch); a count > 2 means it is used multiply.
    let Some(usage_count) = param.field_f64("__typeParameterUsageCount") else {
        return;
    };
    let usage_count = usage_count as i64;
    if !(1..=2).contains(&usage_count) {
        return;
    }

    // The upstream message embeds whether the parameter is "never used"
    // (count == 1, i.e. only the declaration) or "used only once" (count == 2).
    let uses = if usage_count == 1 {
        "never used"
    } else {
        "used only once"
    };

    let name = param.field_str("__typeParameterName").unwrap_or("T");
    let descriptor = descriptor_for(declaration);

    // The diagnostic range is the whole type-parameter declaration. The label
    // range points at the first reference (or the name when there are no
    // references); the host supplies the resolved reference ranges.
    let report_range = param.range;

    ctx.report_with_suggestions(
        "sole",
        report_range,
        build_suggestions(declaration, params, index, param),
    );

    // `name`, `uses`, and `descriptor` carry the same data the upstream
    // `buildSoleMessage` interpolates. They are not consumed by the static
    // message catalog here but are kept so the host renderer can reproduce the
    // exact "Type parameter {name} is {uses} in the {descriptor} signature."
    // text. Referencing them keeps the parity contract explicit.
    let _ = (name, uses, descriptor);
}

/// Equivalent of the `descriptor` argument passed at each listener: "class" for
/// class-like owners, "function" otherwise.
fn descriptor_for(declaration: &LintNode) -> &'static str {
    match declaration.field_str("__ownerKind") {
        Some("ClassDeclaration") | Some("ClassExpression") => "class",
        _ => "function",
    }
}

/// Builds the single `replaceUsagesWithConstraint` suggestion, mirroring the
/// upstream closure: replace every reference with the constraint text (wrapping
/// complex constraints in parens when the reference sits inside an array /
/// indexed-access / union / intersection ancestor), then remove the type
/// parameter from the declaration list.
fn build_suggestions(
    declaration: &LintNode,
    params: &[LintNode],
    index: usize,
    param: &LintNode,
) -> Vec<LintSuggestion> {
    let constraint_text = param
        .field_str("__constraintText")
        .filter(|text| !text.is_empty())
        .unwrap_or("unknown");
    let complex_constraint = param.field_bool("__constraintIsComplex").unwrap_or(false);

    let Some(removal_range) = removal_range(declaration, params, index, param) else {
        // Without the host-resolved removal range we cannot synthesise a safe
        // fix, so emit the diagnostic without a suggestion.
        return Vec::new();
    };

    let mut fixes: Vec<LintFix> = Vec::new();

    for reference in reference_facts(param) {
        // References that fall inside the removed declaration slice are dropped,
        // matching the overlap guard in the upstream closure.
        if reference.range.start < removal_range.end && reference.range.end > removal_range.start {
            continue;
        }
        let replacement = if complex_constraint && reference.needs_parens {
            format!("({constraint_text})")
        } else {
            constraint_text.to_owned()
        };
        fixes.push(LintFix::replace_range(reference.range, replacement));
    }

    fixes.push(LintFix::remove_range(removal_range));

    vec![LintSuggestion {
        message_id: "replaceUsagesWithConstraint".to_owned(),
        message: "Replace all usages of type parameter with its constraint.".to_owned(),
        fixes,
    }]
}

/// One resolved reference to the type parameter, as supplied by the host.
struct ReferenceFact {
    range: TextRange,
    /// Whether the reference's grandparent is an array / indexed-access /
    /// union / intersection type, i.e. `hasMatchingAncestorType`.
    needs_parens: bool,
}

/// Reads the host-provided reference list for a type parameter.
fn reference_facts(param: &LintNode) -> Vec<ReferenceFact> {
    let Some(Value::Array(items)) = param.fields.get("__references") else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let (Some(start), Some(end)) = (
            item.get("start").and_then(Value::as_u64),
            item.get("end").and_then(Value::as_u64),
        ) else {
            continue;
        };
        let needs_parens = item
            .get("hasMatchingAncestorType")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        out.push(ReferenceFact {
            range: TextRange::new(start as u32, end as u32),
            needs_parens,
        });
    }
    out
}

/// Computes the source range removed from the type-parameter list, mirroring
/// `getTypeParameterListRemovalRange`. The host supplies the precise byte
/// offsets (`__listRemovalRange`) because they depend on surrounding trivia and
/// the `<`/`>`/`,` tokens that are not present in the compact node facts. When
/// the host omits the fact, this falls back to the bare parameter range.
fn removal_range(
    _declaration: &LintNode,
    _params: &[LintNode],
    _index: usize,
    param: &LintNode,
) -> Option<TextRange> {
    if let Some(value) = param.fields.get("__listRemovalRange") {
        if let (Some(start), Some(end)) = (
            value.get("start").and_then(Value::as_u64),
            value.get("end").and_then(Value::as_u64),
        ) {
            return Some(TextRange::new(start as u32, end as u32));
        }
    }
    Some(param.range)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnnecessaryTypeParametersRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NoUnnecessaryTypeParametersRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    /// Builds a `TSTypeParameterDeclaration` node holding a single type
    /// parameter whose host facts are taken from `param_fields`.
    fn declaration_with_param(
        owner_kind: &str,
        param_range: TextRange,
        param_fields: serde_json::Value,
        references: serde_json::Value,
    ) -> LintNode {
        let mut param_field_map: BTreeMap<String, serde_json::Value> = param_fields
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        param_field_map.insert("__references".to_owned(), references);

        let param = LintNode {
            kind: "TSTypeParameter".to_owned(),
            range: param_range,
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: param_field_map,
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        };

        let mut fields = BTreeMap::new();
        fields.insert("__ownerKind".to_owned(), json!(owner_kind));

        let mut child_lists = BTreeMap::new();
        child_lists.insert("params".to_owned(), vec![param]);

        LintNode {
            kind: "TSTypeParameterDeclaration".to_owned(),
            range: TextRange::new(param_range.start, param_range.end),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields,
            children: BTreeMap::new(),
            child_lists,
        }
    }

    // ---- should report ----

    #[test]
    fn reports_never_used_type_parameter() {
        // `declare function get<T>(): unknown;` -- T appears only in its own
        // declaration (usage count 1, "never used").
        let node = declaration_with_param(
            "FunctionDeclaration",
            TextRange::new(20, 21),
            json!({
                "__typeParameterName": "T",
                "__typeParameterUsageCount": 1,
                "__repeatedInAst": false,
                "__constraintText": "unknown",
                "__constraintIsComplex": false,
                "__listRemovalRange": { "start": 19, "end": 22 },
            }),
            json!([]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "sole");
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(
            diagnostics[0].suggestions[0].message_id,
            "replaceUsagesWithConstraint"
        );
        // Only the removal fix: there are no references to replace.
        assert_eq!(diagnostics[0].suggestions[0].fixes.len(), 1);
    }

    #[test]
    fn reports_single_use_with_constraint_replacement() {
        // `declare function take<T extends object>(param: T): void;`
        // T used once (count 2). The single reference gets replaced with the
        // constraint text "object" and the parameter is removed from the list.
        let node = declaration_with_param(
            "FunctionDeclaration",
            TextRange::new(22, 38),
            json!({
                "__typeParameterName": "T",
                "__typeParameterUsageCount": 2,
                "__repeatedInAst": false,
                "__constraintText": "object",
                "__constraintIsComplex": false,
                "__listRemovalRange": { "start": 21, "end": 39 },
            }),
            json!([{ "start": 47, "end": 48, "hasMatchingAncestorType": false }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "sole");
        let fixes = &diagnostics[0].suggestions[0].fixes;
        // One replacement + one removal.
        assert_eq!(fixes.len(), 2);
        assert_eq!(fixes[0].replacement_text, "object");
        assert_eq!(fixes[0].range, TextRange::new(47, 48));
        assert_eq!(fixes[1].replacement_text, "");
    }

    #[test]
    fn wraps_complex_constraint_in_parens_for_array_reference() {
        // `function join<T extends string | number>(els: T[]) {...}`
        // The reference T sits inside an array type, so the union constraint is
        // wrapped: `els: (string | number)[]`.
        let node = declaration_with_param(
            "FunctionDeclaration",
            TextRange::new(14, 40),
            json!({
                "__typeParameterName": "T",
                "__typeParameterUsageCount": 2,
                "__repeatedInAst": false,
                "__constraintText": "string | number",
                "__constraintIsComplex": true,
                "__listRemovalRange": { "start": 13, "end": 41 },
            }),
            json!([{ "start": 47, "end": 48, "hasMatchingAncestorType": true }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        let fixes = &diagnostics[0].suggestions[0].fixes;
        assert_eq!(fixes[0].replacement_text, "(string | number)");
    }

    // ---- should NOT report ----

    #[test]
    fn ignores_type_parameter_used_multiple_times() {
        // `declare function identity<T>(param: T): T;` -- usage count > 2.
        let node = declaration_with_param(
            "FunctionDeclaration",
            TextRange::new(26, 27),
            json!({
                "__typeParameterName": "T",
                "__typeParameterUsageCount": 3,
                "__repeatedInAst": true,
                "__constraintText": "unknown",
                "__constraintIsComplex": false,
            }),
            json!([]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_when_repeated_in_ast_fast_path() {
        // The AST fast path observed two or more references even though the
        // checker count fact would otherwise qualify; must short-circuit.
        let node = declaration_with_param(
            "FunctionDeclaration",
            TextRange::new(26, 27),
            json!({
                "__typeParameterName": "T",
                "__typeParameterUsageCount": 2,
                "__repeatedInAst": true,
                "__constraintText": "unknown",
                "__constraintIsComplex": false,
            }),
            json!([]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_when_usage_count_fact_absent() {
        // No checker observation supplied (the upstream `!ok` branch).
        let node = declaration_with_param(
            "FunctionDeclaration",
            TextRange::new(26, 27),
            json!({
                "__typeParameterName": "T",
                "__repeatedInAst": false,
                "__constraintText": "unknown",
                "__constraintIsComplex": false,
            }),
            json!([]),
        );
        assert!(run(&node).is_empty());
    }
}

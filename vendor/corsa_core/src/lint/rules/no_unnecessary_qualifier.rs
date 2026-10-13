use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};

/// Type-aware rule that flags namespace/enum qualifiers that are redundant
/// because the qualified member is already directly in scope.
///
/// Faithful port of tsgolint `no-unnecessary-qualifier`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryQualifierRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unnecessaryQualifier",
    description: "Qualifier is unnecessary since the referenced name is in scope.",
}];

// The upstream rule visits property-access (member) expressions and qualified
// names. corsa renders TS-ESTree, so `PropertyAccessExpression` arrives as
// `MemberExpression` and `QualifiedName` as `TSQualifiedName`.
const LISTENERS: &[&str] = &["TSQualifiedName", "MemberExpression"];

impl RustLintRule for NoUnnecessaryQualifierRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-qualifier"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow unnecessary namespace qualifiers."
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
        let Some((qualifier, name)) = qualifier_and_name(node) else {
            return;
        };

        // Upstream only considers member accesses whose left side is an
        // "entity name expression" (an identifier, or a property-access chain
        // bottoming out in an identifier). Qualified names are always entity
        // names, so this gate only matters for `MemberExpression`.
        if node.kind == "MemberExpression" && !is_entity_name_expression(qualifier) {
            return;
        }

        // The "is the qualifier unnecessary?" decision is intrinsically a
        // type-checker question (symbol resolution + scope analysis). The host
        // supplies the two RAW checker observations and Rust combines them into
        // the rule verdict, keeping the decision in Rust.
        if !qualifier_is_unnecessary(node) {
            return;
        }

        // The upstream rule suppresses reports for nodes nested inside an
        // already-reported namespace expression (its `currentFailedNamespaceExpression`
        // guard). The host marks such nested nodes so the Rust verdict stays here.
        if node
            .field_bool("__qualifierReportSuppressed")
            .unwrap_or(false)
        {
            return;
        }

        // Report on the qualifier, with a fix that removes `qualifier.` so the
        // bare name remains (mirrors `RuleFixRemoveRange(qualifierStart..nameStart)`).
        let remove_range = TextRange::new(qualifier.range.start, name.range.start);
        let fix = if remove_range.is_valid() && !remove_range.is_empty() {
            vec![LintFix::remove_range(remove_range)]
        } else {
            Vec::new()
        };
        ctx.report_with_suggestions(
            "unnecessaryQualifier",
            qualifier.range,
            vec![LintSuggestion {
                message_id: "unnecessaryQualifier".to_owned(),
                message: "Remove the unnecessary qualifier.".to_owned(),
                fixes: fix,
            }],
        );
    }
}

/// Returns the `(qualifier, name)` pair for the supported node kinds.
///
/// - `TSQualifiedName`: qualifier = `left`, name = `right`.
/// - `MemberExpression`: qualifier = `object`, name = `property` (non-computed).
fn qualifier_and_name(node: &LintNode) -> Option<(&LintNode, &LintNode)> {
    match node.kind.as_str() {
        "TSQualifiedName" => Some((node.child("left")?, node.child("right")?)),
        "MemberExpression" => {
            // Upstream uses `PropertyAccessExpression`, which is never a computed
            // (`a[b]`) access, so only dotted access qualifies.
            if node.field_bool("computed").unwrap_or(false) {
                return None;
            }
            Some((node.child("object")?, node.child("property")?))
        }
        _ => None,
    }
}

/// Mirrors upstream `isEntityNameExpression`: an identifier, or a
/// `MemberExpression`/`TSQualifiedName` whose object/left is itself an entity
/// name expression.
fn is_entity_name_expression(node: &LintNode) -> bool {
    match node.kind.as_str() {
        // Upstream `isEntityNameExpression` treats only `Identifier` as the base
        // case (a `this` qualifier is not an entity-name expression upstream).
        "Identifier" => true,
        "MemberExpression" => {
            if node.field_bool("computed").unwrap_or(false) {
                return false;
            }
            node.child("object").is_some_and(is_entity_name_expression)
        }
        "TSQualifiedName" => node.child("left").is_some_and(is_entity_name_expression),
        _ => false,
    }
}

/// Combines the two raw checker observations supplied by the host into the
/// upstream `qualifierIsUnnecessary` verdict. The host computes neither verdict:
/// each field is a single primitive checker query result.
fn qualifier_is_unnecessary(node: &LintNode) -> bool {
    // `__qualifierNamespaceInScope`: the qualifier's symbol resolves (through
    // alias chains) to a namespace/enum declaration that is one of the
    // enclosing scope declarations of this node.
    let namespace_in_scope = node
        .field_bool("__qualifierNamespaceInScope")
        .unwrap_or(false);
    // `__accessedNameAlreadyInScope`: the accessed name's symbol is identical to
    // the export symbol of the symbol found by an unqualified name lookup at the
    // qualifier's location.
    let accessed_in_scope = node
        .field_bool("__accessedNameAlreadyInScope")
        .unwrap_or(false);
    namespace_in_scope && accessed_in_scope
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Value, json};

    use super::super::super::{LintDiagnostic, LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnnecessaryQualifierRule;

    fn node(kind: &str, range: TextRange) -> LintNode {
        LintNode {
            kind: kind.to_owned(),
            range,
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        }
    }

    fn ident(name: &str, range: TextRange) -> LintNode {
        let mut n = node("Identifier", range);
        n.fields.insert("name".to_owned(), json!(name));
        n
    }

    fn run(node: &LintNode) -> Vec<LintDiagnostic> {
        let rule = NoUnnecessaryQualifierRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn set(node: &mut LintNode, key: &str, value: Value) {
        node.fields.insert(key.to_owned(), value);
    }

    /// `A.B` inside `namespace A`, where `B` is already in scope: should report.
    fn qualified_name_unnecessary() -> LintNode {
        // text: "A.B", A at [11,12], B at [13,14]
        let mut n = LintNode {
            kind: "TSQualifiedName".to_owned(),
            range: TextRange::new(11, 14),
            text: Some("A.B".to_owned()),
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        };
        n.children
            .insert("left".to_owned(), ident("A", TextRange::new(11, 12)));
        n.children
            .insert("right".to_owned(), ident("B", TextRange::new(13, 14)));
        n
    }

    #[test]
    fn reports_unnecessary_qualified_name() {
        let mut n = qualified_name_unnecessary();
        set(&mut n, "__qualifierNamespaceInScope", json!(true));
        set(&mut n, "__accessedNameAlreadyInScope", json!(true));

        let diagnostics = run(&n);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_name, "no-unnecessary-qualifier");
        assert_eq!(diagnostics[0].message_id, "unnecessaryQualifier");
        // Reports on the qualifier `A`.
        assert_eq!(diagnostics[0].range, TextRange::new(11, 12));
        // Fix removes "A." (from qualifier start to name start).
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[0].range,
            TextRange::new(11, 13)
        );
    }

    #[test]
    fn reports_unnecessary_member_access() {
        // `A.x` where x is already in scope (export const usage), member access.
        let mut n = node("MemberExpression", TextRange::new(20, 23));
        n.fields.insert("computed".to_owned(), json!(false));
        n.children
            .insert("object".to_owned(), ident("A", TextRange::new(20, 21)));
        n.children
            .insert("property".to_owned(), ident("x", TextRange::new(22, 23)));
        set(&mut n, "__qualifierNamespaceInScope", json!(true));
        set(&mut n, "__accessedNameAlreadyInScope", json!(true));

        let diagnostics = run(&n);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range, TextRange::new(20, 21));
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[0].range,
            TextRange::new(20, 22)
        );
    }

    #[test]
    fn ignores_when_namespace_not_in_scope() {
        // `const x: A.B = 3;` at top level: A is not an enclosing namespace.
        let mut n = qualified_name_unnecessary();
        set(&mut n, "__qualifierNamespaceInScope", json!(false));
        set(&mut n, "__accessedNameAlreadyInScope", json!(true));

        assert!(run(&n).is_empty());
    }

    #[test]
    fn ignores_when_accessed_name_not_in_scope() {
        // Inner namespace shadows the outer type, so the qualifier is required.
        let mut n = qualified_name_unnecessary();
        set(&mut n, "__qualifierNamespaceInScope", json!(true));
        set(&mut n, "__accessedNameAlreadyInScope", json!(false));

        assert!(run(&n).is_empty());
    }

    #[test]
    fn ignores_computed_member_access() {
        // `A["B"]` is not a property-access (qualified) expression upstream.
        let mut n = node("MemberExpression", TextRange::new(0, 6));
        n.fields.insert("computed".to_owned(), json!(true));
        n.children
            .insert("object".to_owned(), ident("A", TextRange::new(0, 1)));
        n.children
            .insert("property".to_owned(), node("Literal", TextRange::new(2, 5)));
        set(&mut n, "__qualifierNamespaceInScope", json!(true));
        set(&mut n, "__accessedNameAlreadyInScope", json!(true));

        assert!(run(&n).is_empty());
    }

    #[test]
    fn ignores_member_with_non_entity_name_object() {
        // `f().x` — object is a call, not an entity-name expression.
        let mut n = node("MemberExpression", TextRange::new(0, 5));
        n.fields.insert("computed".to_owned(), json!(false));
        n.children.insert(
            "object".to_owned(),
            node("CallExpression", TextRange::new(0, 3)),
        );
        n.children
            .insert("property".to_owned(), ident("x", TextRange::new(4, 5)));
        set(&mut n, "__qualifierNamespaceInScope", json!(true));
        set(&mut n, "__accessedNameAlreadyInScope", json!(true));

        assert!(run(&n).is_empty());
    }

    #[test]
    fn ignores_when_report_suppressed_by_outer_namespace() {
        // Nested inside an already-reported namespace expression (e.g. inner of
        // `A.B.T`): upstream's currentFailedNamespaceExpression guard.
        let mut n = qualified_name_unnecessary();
        set(&mut n, "__qualifierNamespaceInScope", json!(true));
        set(&mut n, "__accessedNameAlreadyInScope", json!(true));
        set(&mut n, "__qualifierReportSuppressed", json!(true));

        assert!(run(&n).is_empty());
    }
}

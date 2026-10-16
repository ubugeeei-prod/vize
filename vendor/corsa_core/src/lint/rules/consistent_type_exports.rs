use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::child_list;

// NOTE on options: the only upstream option,
// `fixMixedExportsWithInlineTypeSpecifier` (boolean, default false), changes
// only the *autofix* shape (inline `type ` prefix vs. split statements). It
// never changes whether a diagnostic is reported nor which message id is used,
// so it has no effect on the Rust decision logic below. When the host autofix
// path is wired up it reads the option via `__ruleOptions`
// (helpers::rule_option_bool(node, "fixMixedExportsWithInlineTypeSpecifier")).

/// Type-aware rule that requires exports used only as types to be written with
/// `export type` (faithful port of tsgolint `consistent-type-exports`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ConsistentTypeExportsRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "typeOverValue",
        description: "All exports in the declaration are only used as types. Use `export type`.",
    },
    RuleMessage {
        id: "singleExportIsType",
        description: "Type export is not a value and should be exported using `export type`.",
    },
    RuleMessage {
        id: "multipleExportsAreTypes",
        description: "Type exports are not values and should be exported using `export type`.",
    },
];

// TS-ESTree node kind for a named/star export with a clause. The host emits the
// upstream Go listener `KindExportDeclaration` as the ESTree `ExportNamedDeclaration`
// (named/namespace re-exports) and `ExportAllDeclaration` (bare `export *`).
const LISTENERS: &[&str] = &["ExportNamedDeclaration", "ExportAllDeclaration"];

impl RustLintRule for ConsistentTypeExportsRule {
    fn name(&self) -> &'static str {
        "consistent-type-exports"
    }

    fn docs_description(&self) -> &'static str {
        "Require type-only exports where possible."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        // `export type ...` is already type-only; nothing to do.
        if export_kind(node) == Some("type") {
            return;
        }

        match node.kind.as_str() {
            // `export * from '...'` / `export * as ns from '...'`.
            "ExportAllDeclaration" => check_star_export(ctx, node),
            "ExportNamedDeclaration" => {
                // A namespace re-export (`export * as ns from '...'`) can also be
                // modelled as ExportNamedDeclaration with a namespace clause; the
                // host marks it so we route it through the star-export logic.
                if is_namespace_export(node) {
                    check_star_export(ctx, node);
                    return;
                }
                check_named_export(ctx, node);
            }
            _ => {}
        }
    }
}

/// Reads the ESTree `exportKind` field ("type" | "value").
fn export_kind(node: &LintNode) -> Option<&str> {
    node.field_str("exportKind")
}

/// Whether this export carries a namespace clause (`export * as ns from '...'`).
///
/// TS-ESTree models `export * as ns from '...'` as an `ExportAllDeclaration`
/// with an `exported` identifier, but some host shapes surface it as an
/// `ExportNamedDeclaration`; the host marks it with `__isNamespaceExport`.
fn is_namespace_export(node: &LintNode) -> bool {
    node.field_bool("__isNamespaceExport").unwrap_or(false)
}

/// Whether the export statement targets another module (`from '...'`).
fn has_module_source(node: &LintNode) -> bool {
    node.child("source").is_some() || node.field_str("source").is_some()
}

/// Star-export check: `export * from '...'` (or namespace form). Reports
/// `typeOverValue` when the re-exported module contributes no runtime value.
fn check_star_export(ctx: &mut RuleContext<'_>, node: &LintNode) {
    // The upstream rule only fires when there is a module specifier.
    if !has_module_source(node) {
        return;
    }
    // Raw checker fact: does the resolved module export at least one value?
    // `None` means the module symbol/type could not be resolved -> stay silent
    // (mirrors the early `return` paths in the Go `checkStarExport`).
    if node.field_bool("__starExportHasExportedValue") == Some(false) {
        ctx.report("typeOverValue", node.range);
    }
}

/// Named-export check: `export { A, B as C, type D } [from '...']`.
fn check_named_export(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let specifiers = child_list(node, "specifiers");
    if specifiers.is_empty() {
        return;
    }

    // Partition the value-position specifiers exactly as the Go rule does.
    // Already-inline `export { type X }` specifiers count only toward the type
    // bucket on the autofix side and never toward the report decision (the Go
    // rule keys the decision off `typeBasedNodes`, which excludes them), so we
    // simply skip them here.
    let mut type_based_count = 0usize; // value-position specifiers that resolve to a type
    let mut value_count = 0usize; // resolved runtime values

    for specifier in specifiers {
        if specifier_is_inline_type(specifier) {
            continue;
        }
        match specifier.field_bool("__isTypeBasedSymbol") {
            // Symbol resolved and is a value -> keep as value export.
            Some(false) => value_count += 1,
            // Symbol resolved and is type-only -> should become a type export.
            Some(true) => type_based_count += 1,
            // Unresolved symbol -> skip (Go `if !resolved { continue }`).
            None => {}
        }
    }

    // No specifier needs converting (matches `len(typeBasedNodes) == 0`). Note
    // the Go rule keys off `typeBasedNodes`, which excludes already-inline-type
    // specifiers, so a declaration consisting solely of `type X` entries does
    // not report here.
    if type_based_count == 0 {
        return;
    }

    if value_count == 0 {
        // Every (non-inline) export is a type: `export type { ... }`.
        ctx.report("typeOverValue", node.range);
        return;
    }

    // Mixed: some types, some values.
    if type_based_count == 1 {
        ctx.report("singleExportIsType", node.range);
    } else {
        ctx.report("multipleExportsAreTypes", node.range);
    }
}

/// Whether an `ExportSpecifier` is written with an inline `type` keyword
/// (`export { type X }`).
fn specifier_is_inline_type(specifier: &LintNode) -> bool {
    specifier.field_str("exportKind") == Some("type")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(node: &LintNode) -> Vec<crate::lint::LintDiagnostic> {
        let rule = ConsistentTypeExportsRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn node_from(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    fn specifier(name: &str, type_based: Option<bool>, inline_type: bool) -> serde_json::Value {
        let mut fields = serde_json::Map::new();
        if inline_type {
            fields.insert("exportKind".to_owned(), json!("type"));
        } else {
            fields.insert("exportKind".to_owned(), json!("value"));
        }
        if let Some(value) = type_based {
            fields.insert("__isTypeBasedSymbol".to_owned(), json!(value));
        }
        let _ = name;
        json!({
            "kind": "ExportSpecifier",
            "range": { "start": 0, "end": 1 },
            "fields": fields,
        })
    }

    fn named_export(specifiers: Vec<serde_json::Value>, with_source: bool) -> LintNode {
        let mut fields = serde_json::Map::new();
        fields.insert("exportKind".to_owned(), json!("value"));
        let mut children = serde_json::Map::new();
        if with_source {
            children.insert(
                "source".to_owned(),
                json!({ "kind": "Literal", "range": { "start": 0, "end": 1 } }),
            );
        }
        node_from(json!({
            "kind": "ExportNamedDeclaration",
            "range": { "start": 0, "end": 50 },
            "fields": fields,
            "children": children,
            "childLists": { "specifiers": specifiers },
        }))
    }

    #[test]
    fn reports_type_over_value_when_all_specifiers_are_types() {
        // export { Type1 } from './consistent-type-exports';
        let node = named_export(vec![specifier("Type1", Some(true), false)], true);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "typeOverValue");
    }

    #[test]
    fn reports_single_export_is_type_for_mixed_one_type() {
        // export { Type1, value1 } from './consistent-type-exports';
        let node = named_export(
            vec![
                specifier("Type1", Some(true), false),
                specifier("value1", Some(false), false),
            ],
            true,
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "singleExportIsType");
    }

    #[test]
    fn reports_multiple_exports_are_types_for_mixed_two_types() {
        // export { Type1, value1, Type2, value2 } from './consistent-type-exports';
        let node = named_export(
            vec![
                specifier("Type1", Some(true), false),
                specifier("value1", Some(false), false),
                specifier("Type2", Some(true), false),
                specifier("value2", Some(false), false),
            ],
            true,
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "multipleExportsAreTypes");
    }

    #[test]
    fn does_not_report_when_all_values() {
        // export { variable, Class, Enum };
        let node = named_export(
            vec![
                specifier("variable", Some(false), false),
                specifier("Class", Some(false), false),
                specifier("Enum", Some(false), false),
            ],
            false,
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_already_type_only_export() {
        // export type { Type1 } from './consistent-type-exports';
        let mut node = named_export(vec![specifier("Type1", Some(true), false)], true);
        node.fields.insert("exportKind".to_owned(), json!("type"));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_pure_inline_type_specifiers() {
        // export { type X } -> no value-position type-based specifier, no report.
        let node = named_export(vec![specifier("X", None, true)], false);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn skips_unresolved_symbols() {
        // export { Foo } from 'foo'; -- symbol unresolved -> stay silent.
        let node = named_export(vec![specifier("Foo", None, false)], true);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_star_export_with_no_exported_value() {
        // export * from './consistent-type-exports/type-only-exports';
        let node = node_from(json!({
            "kind": "ExportAllDeclaration",
            "range": { "start": 0, "end": 40 },
            "fields": { "exportKind": "value", "__starExportHasExportedValue": false },
            "children": {
                "source": { "kind": "Literal", "range": { "start": 14, "end": 39 } }
            },
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "typeOverValue");
    }

    #[test]
    fn does_not_report_star_export_with_exported_value() {
        // export * from './consistent-type-exports/value-reexport';
        let node = node_from(json!({
            "kind": "ExportAllDeclaration",
            "range": { "start": 0, "end": 40 },
            "fields": { "exportKind": "value", "__starExportHasExportedValue": true },
            "children": {
                "source": { "kind": "Literal", "range": { "start": 14, "end": 39 } }
            },
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_star_export_without_source() {
        // (defensive) star export without a module specifier.
        let node = node_from(json!({
            "kind": "ExportAllDeclaration",
            "range": { "start": 0, "end": 10 },
            "fields": { "exportKind": "value", "__starExportHasExportedValue": false },
        }));
        assert!(run(&node).is_empty());
    }
}

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};

/// Type-aware rule that disallows use of `@deprecated` declarations.
///
/// Faithful port of the tsgolint builtin `no-deprecated` rule. The host
/// performs the symbol/JSDoc resolution (a raw type-checker query) and reports
/// it back through the `__deprecated` and `__deprecationReason` facts. All of
/// the decision logic (which use sites to skip, which message to emit, what name
/// to render, and the `allow` option) lives here in Rust.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDeprecatedRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "deprecated",
        description: "This declaration is deprecated.",
    },
    RuleMessage {
        id: "deprecatedWithReason",
        description: "This declaration is deprecated.",
    },
];

// Go listens to KindIdentifier, KindElementAccessExpression, KindPrivateIdentifier
// and KindSuperKeyword. In the TS-ESTree shape the host forwards:
//   - Identifier                (KindIdentifier)
//   - MemberExpression          (computed element access => KindElementAccessExpression)
//   - PrivateIdentifier         (KindPrivateIdentifier)
//   - Super                     (KindSuperKeyword)
const LISTENERS: &[&str] = &[
    "Identifier",
    "MemberExpression",
    "PrivateIdentifier",
    "Super",
];

impl RustLintRule for NoDeprecatedRule {
    fn name(&self) -> &'static str {
        "no-deprecated"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow use of deprecated declarations."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        // The host only resolves a deprecated symbol for the use-site nodes it
        // recognises (Identifier / MemberExpression property). When the symbol
        // is not deprecated, there is nothing to report.
        if !node.field_bool("__deprecated").unwrap_or(false) {
            return;
        }

        // Mirror the JS-side listener guards in the Go source.
        if should_skip_for_context(node) {
            return;
        }

        // `allow` option: skip when this exact name is allow-listed. The Go rule
        // matches by TypeOrValueSpecifier; with only fact data available we
        // honour the common case of allow-listing by reported name.
        if is_allowed(node) {
            return;
        }

        // Choose the message ID exactly as the Go rule does: `deprecatedWithReason`
        // when a non-empty JSDoc reason is present, otherwise `deprecated`.
        let has_reason = node
            .field_str("__deprecationReason")
            .map(str::trim)
            .is_some_and(|reason| !reason.is_empty());

        let message_id = if has_reason {
            "deprecatedWithReason"
        } else {
            "deprecated"
        };

        // The Go rule reports on the offending identifier node itself.
        ctx.report(message_id, node.range);
    }
}

/// Returns the parent node kind reported by the host (`__parentKind`).
fn parent_kind(node: &LintNode) -> Option<&str> {
    node.field_str("__parentKind")
}

/// Mirrors the listener-level skips in the Go `KindIdentifier` handler plus the
/// `isDeclaration` / `isInsideImport` guards, to the extent derivable from facts.
fn should_skip_for_context(node: &LintNode) -> bool {
    let Some(parent) = parent_kind(node) else {
        // Go bails out when `node.Parent == nil`.
        return true;
    };

    match parent {
        // Skip JSX closing elements to avoid duplicate reports.
        "JSXClosingElement" => true,
        // Skip identifiers directly in export declarations (not in specifiers).
        "ExportNamedDeclaration" | "ExportAllDeclaration" => true,
        // Skip namespace exports: `export * as ns from 'module'`.
        "ExportNamespaceSpecifier" => true,
        // `isInsideImport`: imported bindings are not "uses".
        "ImportDeclaration"
        | "ImportSpecifier"
        | "ImportDefaultSpecifier"
        | "ImportNamespaceSpecifier" => true,
        // Export specifiers (`export { foo }`, `export { foo as bar }`,
        // `export { foo } from './m'`). The Go rule has nuanced handling here:
        // it suppresses the local PropertyName side and any specifier that
        // carries its own `@deprecated` JSDoc tag, but DOES report when the
        // re-exported symbol is deprecated (see the `deprecatedWithReason`
        // re-export cases in the upstream fixtures). That PropertyName-vs-Name
        // and self-tagged distinction is not derivable from `__parentKind`
        // alone, so we defer entirely to the host's `__deprecated` computation:
        // when it is set on an export specifier node, treat it as a real use
        // and fall through to reporting.
        "ExportSpecifier" => false,
        // `isDeclaration`: declaration name positions are not "uses".
        "VariableDeclarator"
        | "FunctionDeclaration"
        | "FunctionExpression"
        | "ArrowFunctionExpression"
        | "ClassDeclaration"
        | "ClassExpression"
        | "TSEnumDeclaration"
        | "TSEnumMember"
        | "TSInterfaceDeclaration"
        | "TSModuleDeclaration"
        | "TSTypeAliasDeclaration"
        | "TSTypeParameter"
        | "TSMethodSignature"
        | "TSPropertySignature"
        | "MethodDefinition"
        | "PropertyDefinition"
        | "AccessorProperty"
        | "TSParameterProperty"
        | "Property"
        | "TSImportEqualsDeclaration" => {
            // For these parents the Go rule only treats the *name* position as a
            // declaration. The host marks the symbol deprecated only at genuine
            // use sites, so when the fact is present here it is a value use
            // (e.g. a property value `{ prop: deprecatedThing }`) and must be
            // reported. We therefore only skip when the host explicitly says the
            // node sits in a binding/name position.
            node.field_bool("__declarationName").unwrap_or(false)
        }
        _ => false,
    }
}

/// Honours the `allow` option (array of names/specifiers). With fact-only data
/// we match the reported identifier name against allow-list string entries.
fn is_allowed(node: &LintNode) -> bool {
    let Some(options) = node.fields.get("__ruleOptions") else {
        return false;
    };
    let Some(first) = options.as_array().and_then(|opts| opts.first()) else {
        return false;
    };
    let Some(allow) = first.get("allow").and_then(|value| value.as_array()) else {
        return false;
    };
    if allow.is_empty() {
        return false;
    }
    let Some(name) = reported_name(node) else {
        return false;
    };
    allow.iter().any(|entry| match entry {
        serde_json::Value::String(text) => text == &name,
        serde_json::Value::Object(map) => {
            map.get("name").and_then(|v| v.as_str()) == Some(name.as_str())
        }
        _ => false,
    })
}

/// Returns the name to compare against the `allow` option, mirroring
/// `getReportedNodeName`.
fn reported_name(node: &LintNode) -> Option<String> {
    if node.kind == "Super" {
        return Some("super".to_owned());
    }
    if node.kind == "PrivateIdentifier" {
        let name = node
            .field_stringish("name")
            .or_else(|| node.text.clone())
            .unwrap_or_default();
        return Some(format!("#{name}"));
    }
    node.field_stringish("name").or_else(|| node.text.clone())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext};
    use super::NoDeprecatedRule;
    use crate::lint::RustLintRule;

    fn run(node: &LintNode) -> Vec<(String, super::super::super::TextRange)> {
        let rule = NoDeprecatedRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| (diagnostic.message_id, diagnostic.range))
            .collect()
    }

    fn node(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    #[test]
    fn reports_deprecated_identifier_use() {
        // `/** @deprecated */ var a = undefined; a;` -> use of `a`.
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 40, "end": 41 },
            "fields": {
                "name": "a",
                "__deprecated": true,
                "__parentKind": "ExpressionStatement"
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "deprecated");
        assert_eq!(diagnostics[0].1.start, 40);
    }

    #[test]
    fn reports_deprecated_with_reason() {
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 10, "end": 13 },
            "fields": {
                "name": "foo",
                "__deprecated": true,
                "__deprecationReason": "  Use bar instead.  ",
                "__parentKind": "CallExpression"
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "deprecatedWithReason");
    }

    #[test]
    fn does_not_report_non_deprecated() {
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 3 },
            "fields": {
                "name": "bar",
                "__parentKind": "CallExpression"
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_inside_import() {
        // `import { deprecatedFn } from './deprecated';` -> the import binding.
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 9, "end": 21 },
            "fields": {
                "name": "deprecatedFn",
                "__deprecated": true,
                "__parentKind": "ImportSpecifier"
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_jsx_closing_element() {
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 50, "end": 51 },
            "fields": {
                "name": "A",
                "__deprecated": true,
                "__parentKind": "JSXClosingElement"
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_declaration_name() {
        // `/** @deprecated */ const a = 1;` -> the binding name `a`.
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 25, "end": 26 },
            "fields": {
                "name": "a",
                "__deprecated": true,
                "__parentKind": "VariableDeclarator",
                "__declarationName": true
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn honours_allow_option() {
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 1 },
            "fields": {
                "name": "a",
                "__deprecated": true,
                "__parentKind": "CallExpression",
                "__ruleOptions": [{ "allow": ["a"] }]
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_deprecated_reexport_specifier() {
        // `export { deprecatedFunction } from './deprecated';` where the host
        // resolved the re-exported symbol as deprecated. Go reports here
        // (deprecatedWithReason in the upstream fixtures), so the export
        // specifier must NOT be blanket-skipped.
        let node = node(json!({
            "kind": "Identifier",
            "range": { "start": 9, "end": 27 },
            "fields": {
                "name": "deprecatedFunction",
                "__deprecated": true,
                "__deprecationReason": "Use replacement.",
                "__parentKind": "ExportSpecifier"
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "deprecatedWithReason");
    }

    #[test]
    fn reports_private_identifier() {
        let node = node(json!({
            "kind": "PrivateIdentifier",
            "range": { "start": 5, "end": 12 },
            "fields": {
                "name": "secret",
                "__deprecated": true,
                "__parentKind": "MemberExpression"
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "deprecated");
    }
}

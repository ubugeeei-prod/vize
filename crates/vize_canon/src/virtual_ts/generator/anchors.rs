//! Setup binding anchor emission for template-used names.

use super::imports::{IdentifierUsage, collect_setup_binding_anchor_names};
use std::ops::Range;
use vize_carton::{FxHashSet, String, append, profile};
use vize_croquis::{BindingType, Croquis};

/// Header comment for the anchor list; the wording differs when unused-local
/// diagnostics are preserved because the list is then narrowed to
/// template-referenced names.
pub(super) fn setup_binding_anchor_comment(preserve_unused_diagnostics: bool) -> &'static str {
    if preserve_unused_diagnostics {
        "Reference setup bindings used by template generation"
    } else {
        "Reference setup bindings (used in template/CSS v-bind)"
    }
}

/// Anchor a setup binding named `props` at setup scope, before the template
/// closure opens.
///
/// The closure's synthetic `const props` shadows a setup binding of the same
/// name, so the in-closure anchor list can no longer mark it read — a
/// `const props = defineProps(...)` used only through the template
/// (`v-bind="props"`) reported a false TS6133.
///
/// This includes a reactive `defineProps` destructure whose local or rest
/// binding is named `props`: Croquis classifies those as `BindingType::Props`,
/// but they are still authored setup locals shadowed by the template closure.
///
/// Only a template that spells `props` itself counts as a read, so a component
/// whose template reads prop names directly (`{{ label }}`) keeps its genuine
/// unused-`props` report. Unlike the in-closure anchor list this narrowing is
/// unconditional: the LSP generates with unused-local diagnostics unpreserved
/// yet still surfaces TS6133 as a hint, so anchoring on binding kind alone
/// would suppress true positives there.
pub(super) fn emit_props_shadow_anchor(
    ts: &mut String,
    summary: &Croquis,
    template_referenced_names: &FxHashSet<String>,
) -> Option<Range<usize>> {
    if is_setup_props_binding(summary) && template_referenced_names.contains("props") {
        ts.push_str("  // Anchor before the template scope shadows `props`\n");
        let start = ts.len() + "  void ".len();
        ts.push_str("  void props;\n");
        return Some(start..start + "props".len());
    }
    None
}

fn is_setup_props_binding(summary: &Croquis) -> bool {
    crate::virtual_ts::script_facts::binding_type(summary, "props")
        .as_ref()
        .is_some_and(is_setup_variable)
        || summary
            .macros
            .props_destructure()
            .is_some_and(|destructure| {
                destructure
                    .bindings
                    .values()
                    .any(|binding| binding.local.as_str() == "props")
                    || destructure.rest_id.as_deref() == Some("props")
            })
}

/// True for bindings declared as variables in `<script setup>` scope
/// (let/const/ref/reactive), as opposed to props, Options API members,
/// globals, or imports.
fn is_setup_variable(kind: &BindingType) -> bool {
    matches!(
        kind,
        BindingType::SetupLet
            | BindingType::SetupMaybeRef
            | BindingType::SetupRef
            | BindingType::SetupReactiveConst
            | BindingType::SetupConst
    )
}

pub(super) struct SetupBindingAnchors<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) script_content: Option<&'a str>,
    pub(super) usage: &'a IdentifierUsage,
    pub(super) template_usage_names: &'a FxHashSet<String>,
    pub(super) preserve_unused_diagnostics: bool,
}

impl SetupBindingAnchors<'_> {
    pub(super) fn emit(&self, ts: &mut String, label: &'static str) {
        profile!(
            label,
            emit_setup_binding_anchors(
                ts,
                self.summary,
                self.script_content,
                self.usage,
                self.preserve_unused_diagnostics
                    .then_some(self.template_usage_names),
                setup_binding_anchor_comment(self.preserve_unused_diagnostics),
            )
        );
    }
}

fn emit_setup_binding_anchors(
    ts: &mut String,
    summary: &Croquis,
    script_content: Option<&str>,
    usage: &IdentifierUsage,
    template_referenced_names: Option<&FxHashSet<String>>,
    comment: &str,
) {
    if !crate::virtual_ts::script_facts::has_typed_bindings(summary) {
        return;
    }

    let binding_names = collect_setup_binding_anchor_names(
        summary,
        script_content,
        usage,
        template_referenced_names,
    );
    let mut first = true;
    for name in binding_names {
        if is_reserved_anchor_name(&name) {
            continue;
        }
        if first {
            append!(*ts, "\n  // {comment}\n  ");
        } else {
            ts.push(' ');
        }
        append!(*ts, "void {name};");
        first = false;
    }
    if !first {
        ts.push('\n');
    }
}

fn is_reserved_anchor_name(name: &str) -> bool {
    matches!(
        name,
        "default"
            | "class"
            | "new"
            | "delete"
            | "void"
            | "typeof"
            | "in"
            | "instanceof"
            | "return"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "with"
            | "var"
            | "let"
            | "const"
            | "function"
            | "this"
            | "super"
            | "import"
            | "export"
            | "yield"
            | "await"
            | "async"
            | "static"
            | "enum"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
    )
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
    use vize_carton::String;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
    use crate::virtual_ts::VirtualTsOptions;

    const CORPUS: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/quoted-prop-setup-anchors/"
    );

    fn parse_errors(source: &str) -> usize {
        Parser::new(&Allocator::default(), source, SourceType::ts())
            .parse()
            .diagnostics
            .len()
    }

    #[test]
    fn complete_original_native_projections_retain_the_syntax_regression_control() {
        for name in ["popover", "tooltip"] {
            let before =
                std::fs::read_to_string(format!("{CORPUS}{name}.original-h.vue.ts.txt")).unwrap();
            assert!(
                parse_errors(&before) > 0,
                "literal original-H projection is the regression control"
            );
        }
        // The fresh full ElementPlus projections run through their original pinned
        // import graph in the required native integration lane, after App E2E has
        // hydrated it. A hand-built Croquis summary cannot stand in for that graph.
    }

    #[test]
    fn quoted_prop_keys_keep_valid_unicode_and_dollar_identifiers() {
        let source =
            std::fs::read_to_string(format!("{CORPUS}quoted-properties-and-unicode.vue.txt"))
                .unwrap();
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
        let script = descriptor.script_setup.as_ref().unwrap().content.as_ref();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let usage = super::super::imports::collect_identifier_usage(&parsed.program);
        let mut anchors = String::default();
        super::emit_setup_binding_anchors(
            &mut anchors,
            &summary,
            Some(script),
            &usage,
            None,
            "Reference setup bindings (used in template/CSS v-bind)",
        );
        assert_eq!(
            anchors.as_str(),
            "\n  // Reference setup bindings (used in template/CSS v-bind)\n  void $setup; void $value; void _private; void café; void props; void résumé;\n"
        );
        assert_eq!(
            summary
                .macros
                .props()
                .iter()
                .map(|prop| prop.name.as_str())
                .collect::<Vec<_>>(),
            [
                "onUpdate:visible",
                "data-id",
                "class",
                "default",
                "café",
                "$value",
                "_private"
            ]
        );
        let output = generate_vue_document_virtual_ts(
            std::path::Path::new("Identifiers.vue"),
            &source,
            &VirtualTsOptions::default(),
            &ImportRewriter::new(),
            false,
        )
        .unwrap();
        assert_eq!(parse_errors(&output.pre_rewrite_code), 0);
    }
}

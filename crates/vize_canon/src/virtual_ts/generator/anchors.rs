//! Setup binding anchor emission for template-used names.

use super::global_components::GlobalComponentPlan;
use super::imports::{IdentifierUsage, ValueImportNames, collect_setup_binding_anchor_names};
use crate::virtual_ts::types::VirtualTsOptions;
use std::ops::Range;
use vize_carton::{CompactString, FxHashSet, String, append, profile};
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
    ts: &mut crate::virtual_ts::TextBuf,
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
    pub(super) imports: &'a ValueImportNames,
    pub(super) usage: &'a IdentifierUsage,
    pub(super) template_usage_names: &'a FxHashSet<String>,
    pub(super) template_value_usage_names: &'a FxHashSet<String>,
    pub(super) preserve_unused_diagnostics: bool,
}

impl SetupBindingAnchors<'_> {
    pub(super) fn emit_with_components(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        options: &VirtualTsOptions,
        global_components: &GlobalComponentPlan<'_>,
        type_only_imports: &FxHashSet<CompactString>,
    ) {
        let value_reads = super::unresolved_components::emit_unresolved_components(
            ts,
            self.summary,
            options,
            global_components,
            type_only_imports,
        );
        self.emit_with_value_reads(
            ts,
            "canon.virtual_ts.emit_setup_binding_anchors",
            &value_reads,
        );
    }

    pub(super) fn emit(&self, ts: &mut crate::virtual_ts::TextBuf, label: &'static str) {
        self.emit_with_value_reads(ts, label, &[]);
    }

    fn emit_with_value_reads(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        label: &'static str,
        value_reads: &[CompactString],
    ) {
        profile!(
            label,
            emit_setup_binding_anchors(
                ts,
                self.summary,
                self.imports,
                self.usage,
                self.preserve_unused_diagnostics
                    .then_some(self.template_usage_names),
                self.template_value_usage_names,
                (
                    setup_binding_anchor_comment(self.preserve_unused_diagnostics),
                    value_reads
                ),
            )
        );
    }
}

fn emit_setup_binding_anchors(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    imports: &ValueImportNames,
    usage: &IdentifierUsage,
    template_referenced_names: Option<&FxHashSet<String>>,
    template_value_names: &FxHashSet<String>,
    references: (&str, &[CompactString]),
) {
    let (comment, component_value_reads) = references;
    if !crate::virtual_ts::script_facts::has_typed_bindings(summary) {
        return;
    }

    let binding_names = collect_setup_binding_anchor_names(
        summary,
        imports,
        usage,
        template_referenced_names,
        template_value_names,
    );
    if binding_names.is_empty() {
        return;
    }
    // One immutable fact table serves every anchor; preparing it per name
    // rebuilds the full binding union and makes emission quadratic.
    crate::virtual_ts::script_facts::with_bindings(summary, |bindings| {
        let mut first = true;
        for name in binding_names {
            let value_import = imports.bindings.contains(name.as_str());
            let kind = bindings.binding_type(name.as_str());
            let name = crate::virtual_ts::props::setup_anchor_identifier(summary, name, kind);
            // The exact same lexical scope already read these component values.
            // Namespace members do not establish a read of a different anchor.
            // Imports without a real value read keep their const-enum-safe query.
            if is_reserved_anchor_name(&name)
                || (value_import
                    && component_value_reads
                        .binary_search_by(|read| read.as_str().cmp(name.as_str()))
                        .is_ok())
            {
                continue;
            }
            if first {
                append!(*ts, "\n  // {comment}\n  ");
            } else {
                ts.push(' ');
            }
            if value_import {
                // A direct value read is illegal for imported `const enum`s (TS2475).
                // `keyof typeof` consumes the permitted type query without asserting
                // directly to the enum object type, which TypeScript also rejects.
                append!(*ts, "void (null as unknown as keyof typeof {name});");
            } else if kind == Some(BindingType::SetupLet) {
                // The template IIFE shares setup's flow: a value read of a
                // `let` assigned only in callbacks is TS2454. A type query
                // still marks it used without asserting definite assignment.
                append!(*ts, "void (null as unknown as typeof {name});");
            } else {
                append!(*ts, "void {name};");
            }
            first = false;
        }
        if !first {
            ts.push('\n');
        }
    });
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
mod contextual_names;

#[cfg(test)]
mod value_import_names;

#[cfg(test)]
mod component_value_reads;

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
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
        let imports =
            super::super::imports::ImportNames::collect(&summary, Some(script), &usage).values;
        let mut anchors = crate::virtual_ts::TextBuf::default();
        super::emit_setup_binding_anchors(
            &mut anchors,
            &summary,
            &imports,
            &usage,
            None,
            &Default::default(),
            (
                "Reference setup bindings (used in template/CSS v-bind)",
                &[],
            ),
        );
        assert_eq!(
            anchors.as_str(),
            "\n  // Reference setup bindings (used in template/CSS v-bind)\n  void $setup; void $value; void _private; void café; void _class; void _default; void props; void résumé;\n"
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

    #[test]
    fn imported_value_anchors_use_const_enum_safe_type_queries() {
        let script = r#"import { ref } from 'vue';
import { State } from './state';
const state = ref(State.Idle);"#;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let usage = super::super::imports::collect_identifier_usage(&parsed.program);
        let imports =
            super::super::imports::ImportNames::collect(&summary, Some(script), &usage).values;
        let mut anchors = crate::virtual_ts::TextBuf::default();
        super::emit_setup_binding_anchors(
            &mut anchors,
            &summary,
            &imports,
            &usage,
            None,
            &Default::default(),
            (
                "Reference setup bindings (used in template/CSS v-bind)",
                &[],
            ),
        );

        assert_eq!(
            anchors.as_str(),
            "\n  // Reference setup bindings (used in template/CSS v-bind)\n  void (null as unknown as keyof typeof State); void (null as unknown as keyof typeof ref); void state;\n"
        );
    }

    #[test]
    fn let_anchors_use_type_queries_without_asserting_definite_assignment() {
        // misskey MkMediaAudio: `let stop: () => void` is assigned only in a
        // callback, so a value read in the template IIFE is TS2454.
        let script = r#"import { onMounted } from 'vue';
let stop: () => void;
let count = 0;
const label = 'a';
onMounted(() => { stop = () => {}; count++; });"#;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let usage = super::super::imports::collect_identifier_usage(&parsed.program);
        let imports =
            super::super::imports::ImportNames::collect(&summary, Some(script), &usage).values;
        let mut anchors = crate::virtual_ts::TextBuf::default();
        super::emit_setup_binding_anchors(
            &mut anchors,
            &summary,
            &imports,
            &usage,
            None,
            &Default::default(),
            (
                "Reference setup bindings (used in template/CSS v-bind)",
                &[],
            ),
        );

        assert_eq!(
            anchors.as_str(),
            "\n  // Reference setup bindings (used in template/CSS v-bind)\n  void (null as unknown as typeof count); void label; void (null as unknown as keyof typeof onMounted); void (null as unknown as typeof stop);\n"
        );
    }
}

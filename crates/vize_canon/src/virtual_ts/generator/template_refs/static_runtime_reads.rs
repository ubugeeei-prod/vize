//! Raw setup reads performed by inline static template refs.
//!
//! These reads must precede the synthetic template scope: adding their names
//! to ordinary template usage would unwrap or shadow the original refs.

use oxc_syntax::identifier::is_identifier_name;
use vize_croquis::{BindingType, Croquis, ScopeData, ScopeKind};
use vize_l0::{FxHashSet, String, append};
use vize_relief::{PropNode, RootNode, TemplateChildNode};

use super::super::super::types::VirtualTsGenerationOptions;
use super::super::legacy_vue2::needs_legacy_vue2_helpers;
use crate::virtual_ts::script_facts::{ScriptBindings, with_bindings};

pub(super) fn emit(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    root: Option<&RootNode<'_>>,
    generation: VirtualTsGenerationOptions<'_>,
) {
    if needs_legacy_vue2_helpers(generation.legacy_vue2, generation.dialect) {
        return;
    }
    let Some(root) = root else { return };
    // A raw static ref always spells the attribute name `ref`. Keep ordinary
    // components out of the additional AST walk; empty source remains unknown.
    if !root.source.is_empty() && !root.source.contains("ref") {
        return;
    }
    let mut names = FxHashSet::default();
    with_bindings(summary, |bindings| {
        if bindings.is_script_setup() {
            for child in &root.children {
                collect(child, summary, bindings, &mut names);
            }
        }
    });
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort_unstable();
    if !names.is_empty() {
        ts.push_str("  // Runtime reads for static template refs\n");
        for name in names {
            // This is a usage witness, not an authored read. Even a deferred
            // closure reports TS2454 when a mutable local is never initialized.
            append!(*ts, "  void (() => {name}!);\n");
        }
        ts.push('\n');
    }
}

fn collect(
    child: &TemplateChildNode<'_>,
    summary: &Croquis,
    bindings: ScriptBindings<'_>,
    names: &mut FxHashSet<String>,
) {
    match child {
        TemplateChildNode::Element(element) => {
            for prop in &element.props {
                let PropNode::Attribute(attribute) = prop else {
                    continue;
                };
                if attribute.name != "ref" {
                    continue;
                }
                let Some(value) = &attribute.value else {
                    continue;
                };
                let name = value.content;
                if is_identifier_name(name)
                    && runtime_binding(summary, bindings, name)
                    && !template_shadow(summary, name, element.loc.span.start)
                {
                    names.insert(name.into());
                }
            }
            for child in &element.children {
                collect(child, summary, bindings, names);
            }
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                for child in &branch.children {
                    collect(child, summary, bindings, names);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in &branch.children {
                collect(child, summary, bindings, names);
            }
        }
        TemplateChildNode::For(node) => {
            for child in &node.children {
                collect(child, summary, bindings, names);
            }
        }
        _ => {}
    }
}

fn runtime_binding(summary: &Croquis, bindings: ScriptBindings<'_>, name: &str) -> bool {
    let span = bindings.span(name);
    if span.is_some_and(|(start, _)| {
        summary
            .import_statements
            .iter()
            .any(|import| import.start <= start && start < import.end)
    }) {
        // Import bindings live in their retained external-module scope,
        // not directly in ScriptSetup. Follow Vue's import metadata law:
        // namespace/type-only/Vue imports and default SFC imports are const.
        return summary.scopes.iter().any(|scope| {
            let ScopeData::ExternalModule(module) = scope.data() else {
                return false;
            };
            if module.is_type_only
                || module.source == "vue"
                || !scope
                    .parent()
                    .and_then(|id| summary.scopes.get_scope(id))
                    .is_some_and(|parent| parent.kind == ScopeKind::ScriptSetup)
            {
                return false;
            }
            module.exports.iter().any(|export| {
                export.local_name == name
                    && !(export.export_name == "default" && module.source.ends_with(".vue"))
            })
        });
    }
    if !matches!(
        bindings.binding_type(name),
        Some(BindingType::SetupLet | BindingType::SetupRef | BindingType::SetupMaybeRef)
    ) {
        return false;
    }
    // Definition spans establish the owning script, including destructures.
    // Keep spanless summaries conservative when multiple scripts are present;
    // nested locals live only in Scope, not in the top-level Bindings fact.
    summary.scopes.iter().any(|scope| {
        scope.kind == ScopeKind::ScriptSetup
            && span.is_none_or(|(start, _)| scope.span.start <= start && start < scope.span.end)
    }) && (span.is_some()
        || !summary
            .scopes
            .iter()
            .any(|scope| scope.kind == ScopeKind::NonScriptSetup))
}

fn template_shadow(summary: &Croquis, name: &str, offset: u32) -> bool {
    // Only template-coordinate scopes participate. Comparing this offset with
    // script functions/blocks would conflate two independent source spaces.
    summary.scopes.iter().any(|scope| {
        matches!(
            scope.kind,
            ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
        ) && scope.span.start <= offset
            && offset < scope.span.end
            && scope.get_binding(name).is_some()
    })
}

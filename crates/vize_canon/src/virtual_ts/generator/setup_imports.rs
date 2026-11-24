//! Specialize authored Vue context imports once in their setup scope.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{String, append};
use vize_croquis::script_parser::parse_program_for_analysis;
use vize_l0::ts;

use super::component_public_types::TemplateSlotsType;

use crate::virtual_ts::{
    VizeMapping, VizeSemanticLink, VizeSemanticLinkKind,
    types::{CSS_MODULE_GLOBAL_MARKER, VirtualTsOptions},
};

#[derive(Default)]
pub(super) struct SetupImportPlan {
    imports: Vec<ImportSpec>,
    slots_type: Option<TemplateSlotsType>,
    attrs_type: Option<String>,
    /// `<script setup>` binds `$slots` itself, which the template then reads.
    authored_dollar_slots: bool,
    /// Script consumers can read slots which have no template outlet.
    untyped_script_slots: bool,
    inferred_slots: bool,
}

struct ImportSpec {
    name: String,
    span: Range<usize>,
    namespace: bool,
    css: bool,
    slots: bool,
    attrs: bool,
}

impl SetupImportPlan {
    pub(super) fn new(
        script: Option<&str>,
        summary: &vize_croquis::Croquis,
        template: Option<&vize_relief::RootNode<'_>>,
        checks: crate::virtual_ts::types::VirtualTsCheckOptions,
    ) -> Self {
        let inferred_slots = super::component_public_types::infers_slots(summary, template, checks);
        let slots_type = checks.infer_template_dollar_slots.then(|| {
            super::component_public_types::template_slots_type(summary, inferred_slots, script)
        });
        let attrs_type = checks.infer_template_dollar_attrs.then(|| {
            let mut ty = String::from("import('vue').ComponentPublicInstance['$attrs']");
            if checks.fallthrough_attributes
                && let Some(inherited) =
                    super::fallthrough::fallthrough_attrs_type_ref(summary, template)
            {
                append!(ty, " & {inherited}");
            }
            ty
        });
        let authored_dollar_slots =
            crate::virtual_ts::script_facts::with_bindings(summary, |bindings| {
                bindings.is_script_setup() && bindings.contains_binding("$slots")
            });
        let mut plan = Self {
            slots_type,
            attrs_type,
            authored_dollar_slots,
            inferred_slots,
            ..Self::default()
        };
        let Some(script) = script else {
            return plan;
        };
        let css = script.contains("useCssModule");
        let slots = plan.slots_type.is_some() && script.contains("useSlots");
        // `inferTemplateDollarSlots` retypes `useSlots()` to the exact template map, so a script
        // consumer cannot read an undeclared slot and the public map stays closed.
        let public_slots = plan.slots_type.is_none()
            && summary.macros.define_slots().is_none()
            && script.contains("useSlots");
        let attrs = plan.attrs_type.is_some() && script.contains("useAttrs");
        if !css && !slots && !attrs && !public_slots {
            return plan;
        }
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, script, SourceType::tsx());
        if parsed.panicked {
            return plan;
        }
        let built = SemanticBuilder::new()
            .with_build_nodes(public_slots)
            .build(&parsed.program);
        for statement in &parsed.program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.source.value != "vue" || import.import_kind == ImportOrExportKind::Type {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                let (local, namespace, css, slots, attrs) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if specifier.imported.name() == "useCssModule"
                            && specifier.import_kind != ImportOrExportKind::Type =>
                    {
                        (&specifier.local, false, true, false, false)
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if (slots || public_slots)
                            && specifier.imported.name() == "useSlots"
                            && specifier.import_kind != ImportOrExportKind::Type =>
                    {
                        (&specifier.local, false, false, slots, false)
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if attrs
                            && specifier.imported.name() == "useAttrs"
                            && specifier.import_kind != ImportOrExportKind::Type =>
                    {
                        (&specifier.local, false, false, false, true)
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        (&specifier.local, true, css, slots, attrs)
                    }
                    _ => continue,
                };
                let Some(symbol) = local.symbol_id.get() else {
                    continue;
                };
                // An unused import must retain its authored unused diagnostic.
                if built.semantic.symbol_references(symbol).next().is_none() {
                    continue;
                }
                if public_slots && (namespace || (!css && !attrs)) {
                    plan.untyped_script_slots |= built.semantic.symbol_references(symbol).any(|reference| {
                            if !reference.is_read() || reference.flags().is_value_as_type() {
                                return false;
                            }
                            if !namespace {
                                return true;
                            }
                            match built.semantic.nodes().parent_kind(reference.node_id()) {
                                AstKind::StaticMemberExpression(member) => member.property.name == "useSlots",
                                AstKind::ComputedMemberExpression(member) => matches!(&member.expression, oxc_ast::ast::Expression::StringLiteral(name) if name.value == "useSlots"),
                                _ => false,
                            }
                        });
                }
                if !css && !slots && !attrs {
                    continue;
                }
                let span = local.span;
                plan.imports.push(ImportSpec {
                    name: local.name.as_str().into(),
                    span: span.start as usize..span.end as usize,
                    namespace,
                    css,
                    slots,
                    attrs,
                });
            }
        }
        plan
    }

    pub(super) fn emit_import_anchors(&self, ts: &mut crate::virtual_ts::TextBuf) {
        for (index, import) in self.imports.iter().enumerate() {
            append!(
                *ts,
                "const __vize_setup_import_{index} = {};\n",
                import.name
            );
        }
    }

    pub(super) fn read_template_slots_from(&mut self, value: &str) {
        if let Some(slots) = &mut self.slots_type {
            slots.read_template_value(value);
        }
    }

    pub(super) fn has_own_slots(&self) -> bool {
        self.slots_type.is_some()
    }

    pub(super) fn has_untyped_script_slots(&self) -> bool {
        self.untyped_script_slots
    }

    pub(super) fn infers_slots(&self) -> bool {
        self.inferred_slots
    }

    pub(super) fn attrs_type(&self) -> Option<&str> {
        self.attrs_type.as_deref()
    }

    pub(super) fn emit_template_slots(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        mappings: &mut Vec<VizeMapping>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        if let Some(slots) = self
            .slots_type
            .as_ref()
            .filter(|_| !self.authored_dollar_slots)
        {
            ts.push_str("    const $slots: ");
            slots.emit(ts, mappings, source_offset);
            ts.push_str(" = undefined as any;\n");
        }
    }

    pub(super) fn emit_setup(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        options: &VirtualTsOptions,
        mappings: &mut Vec<VizeMapping>,
        links: &mut Vec<VizeSemanticLink>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        if self.imports.is_empty() {
            return;
        }
        if self.imports.iter().any(|import| import.css) {
            emit_css_module_type(ts, options);
        }
        if self.imports.iter().any(|import| import.slots)
            && let Some(slots) = &self.slots_type
        {
            ts.push_str("  type $UseSlots = () => ");
            slots.emit(ts, mappings, source_offset);
            ts.push_str(";\n");
        }
        if self.imports.iter().any(|import| import.attrs)
            && let Some(attrs) = &self.attrs_type
        {
            append!(*ts, "  type $UseAttrs = () => {attrs};\n");
        }
        for (index, import) in self.imports.iter().enumerate() {
            let ImportSpec {
                name,
                span,
                namespace,
                css,
                slots,
                attrs,
            } = import;
            ts.push_str("  const ");
            let start = ts.len();
            ts.push_str(name);
            let original = source_offset(span.start)..source_offset(span.end);
            if let Some(source_range) =
                super::script_module::mapped_binding_range(mappings, &original)
            {
                links.push(VizeSemanticLink {
                    source_range,
                    target_range: start..ts.len(),
                    kind: VizeSemanticLinkKind::VueSetupImportSpecialization,
                });
            }
            mappings.push(VizeMapping {
                gen_range: start..ts.len(),
                src_range: original,
                sub_spans: Vec::new(),
            });
            append!(*ts, " = __vize_setup_import_{index} as unknown as ");
            if *namespace {
                let mut fields = Vec::new();
                let mut keys = Vec::new();
                if *css {
                    keys.push("'useCssModule'");
                    fields.push("useCssModule: $UseCssModule");
                }
                if *slots {
                    keys.push("'useSlots'");
                    fields.push("useSlots: $UseSlots");
                }
                if *attrs {
                    keys.push("'useAttrs'");
                    fields.push("useAttrs: $UseAttrs");
                }
                append!(
                    *ts,
                    "{}<typeof __vize_setup_import_{index}, {}> & {{ {} }};\n",
                    crate::virtual_ts::generated_names::lib("Omit"),
                    keys.join(" | "),
                    fields.join("; ")
                );
            } else if *css {
                ts.push_str("$UseCssModule;\n");
            } else if *attrs {
                ts.push_str("$UseAttrs;\n");
            } else {
                ts.push_str("$UseSlots;\n");
            }
        }
    }
}

fn emit_css_module_type(ts: &mut crate::virtual_ts::TextBuf, options: &VirtualTsOptions) {
    let mut has_default = false;
    ts! { &mut *ts =>
        #("  type $StyleModules = ")
        {
            #("\n")
            #{
                for global in options
                    .template_globals
                    .iter()
                    .filter(|global| global.default_value == CSS_MODULE_GLOBAL_MARKER)
                {
                    let name = serde_json::to_string(global.name.as_str()).unwrap_or_default();
                    append!(*__ts, "    {name}: ");
                    crate::virtual_ts::generated_names::push_lib_text(
                        __ts,
                        global.type_annotation.as_str(),
                    );
                    __ts.push_str(";\n");
                    has_default |= global.name == "$style";
                }
                let record = crate::virtual_ts::generated_names::lib("Record");
                for name in &options.css_modules {
                    let quoted = serde_json::to_string(name.as_str()).unwrap_or_default();
                    append!(*__ts, "    {quoted}: {record}<string, string>;\n");
                    has_default |= name == "$style";
                }
            }
            #("  ")
        }
        #(";\n  type $UseCssModule = ")
        {
            #("\n")
            #{
                if has_default {
                    __ts.push_str("    (): $StyleModules['$style'];\n");
                }
                append!(
                    *__ts,
                    "    <K extends {}<keyof $StyleModules, '$style'>>(name: K): $StyleModules[K];\n  ",
                    crate::virtual_ts::generated_names::lib("Exclude")
                );
            }
        }
        ;
    }
}

#[cfg(test)]
mod tests;

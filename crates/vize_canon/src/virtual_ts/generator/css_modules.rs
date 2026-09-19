//! Specialize authored Vue CSS-module imports once in their setup scope.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{String, append};

use crate::virtual_ts::{
    VizeMapping, VizeSemanticLink, VizeSemanticLinkKind,
    types::{CSS_MODULE_GLOBAL_MARKER, VirtualTsOptions},
};

#[derive(Default)]
pub(super) struct CssModulePlan {
    imports: Vec<(String, Range<usize>, bool)>,
}

impl CssModulePlan {
    pub(super) fn new(script: Option<&str>) -> Self {
        let mut plan = Self::default();
        let Some(script) = script.filter(|source| source.contains("useCssModule")) else {
            return plan;
        };
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::tsx()).parse();
        if parsed.panicked {
            return plan;
        }
        let built = SemanticBuilder::new().build(&parsed.program);
        for statement in &parsed.program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.source.value != "vue" || import.import_kind == ImportOrExportKind::Type {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                let (local, namespace) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if specifier.imported.name() == "useCssModule"
                            && specifier.import_kind != ImportOrExportKind::Type =>
                    {
                        (&specifier.local, false)
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        (&specifier.local, true)
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
                let span = local.span;
                plan.imports.push((
                    local.name.as_str().into(),
                    span.start as usize..span.end as usize,
                    namespace,
                ));
            }
        }
        plan
    }

    pub(super) fn emit_import_anchors(&self, ts: &mut String) {
        for (index, (name, _, _)) in self.imports.iter().enumerate() {
            append!(*ts, "const __vize_css_module_{index} = {name};\n");
        }
    }

    pub(super) fn emit_setup(
        &self,
        ts: &mut String,
        options: &VirtualTsOptions,
        mappings: &mut Vec<VizeMapping>,
        links: &mut Vec<VizeSemanticLink>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        if self.imports.is_empty() {
            return;
        }
        ts.push_str("  type __VizeStyleModules = {\n");
        let mut has_default = false;
        for global in options
            .template_globals
            .iter()
            .filter(|global| global.default_value == CSS_MODULE_GLOBAL_MARKER)
        {
            let name =
                serde_json::to_string(global.name.as_str()).expect("CSS module name serializes");
            append!(*ts, "    {name}: {};\n", global.type_annotation);
            has_default |= global.name == "$style";
        }
        for name in &options.css_modules {
            let quoted = serde_json::to_string(name.as_str()).expect("CSS module name serializes");
            append!(*ts, "    {quoted}: Record<string, string>;\n");
            has_default |= name == "$style";
        }
        ts.push_str("  };\n  type __VizeUseCssModule = {\n");
        if has_default {
            ts.push_str("    (): __VizeStyleModules['$style'];\n");
        }
        ts.push_str("    <K extends Exclude<keyof __VizeStyleModules, '$style'>>(name: K): __VizeStyleModules[K];\n  };\n");
        for (index, (name, span, namespace)) in self.imports.iter().enumerate() {
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
            append!(*ts, " = __vize_css_module_{index} as unknown as ");
            if *namespace {
                append!(
                    *ts,
                    "Omit<typeof __vize_css_module_{index}, 'useCssModule'> & {{ useCssModule: __VizeUseCssModule }};\n"
                );
            } else {
                ts.push_str("__VizeUseCssModule;\n");
            }
        }
    }
}

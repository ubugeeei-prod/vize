//! Emit helpers by lexical ownership; never rewrite a generated block afterward.

use crate::virtual_ts::helpers::setup_macros::{
    SETUP_MACRO_HELPERS, VUE_SETUP_HELPERS, VUE_SETUP_HELPERS_HOISTED,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, append};
use vize_croquis::Croquis;

pub(crate) struct SetupHelperPlan {
    shadowed: FxHashSet<CompactString>,
    vue_template_ref_import: bool,
    pub(crate) macro_results: Vec<CompactString>,
}

impl SetupHelperPlan {
    pub(crate) fn collect(summary: &Croquis, source: Option<&str>) -> Self {
        shadowed_helpers(summary, source)
    }

    pub(crate) fn emit_import_anchors(&self, ts: &mut String) {
        if self.vue_template_ref_import {
            ts.push_str("// Keep the authored Vue helper import used by its specialized setup signature\nvoid useTemplateRef;\n");
        }
    }
}

pub(super) fn emit(
    ts: &mut String,
    plan: &SetupHelperPlan,
    hoisted: bool,
    generic_props: bool,
    typed_refs: bool,
) {
    let shadowed = &plan.shadowed;
    if shadowed.is_empty() && !generic_props && !typed_refs {
        ts.push_str(if hoisted {
            VUE_SETUP_HELPERS_HOISTED
        } else {
            VUE_SETUP_HELPERS
        });
        return;
    }
    ts.push_str(if hoisted {
        "  // Compiler macros (setup-scope only; signatures hoisted to the shared helpers file)\n"
    } else {
        "  // Compiler macros (only valid in setup scope, not global)\n"
    });
    for helper in SETUP_MACRO_HELPERS {
        if shadowed.contains(helper.name) {
            continue;
        }
        if helper.name == "defineProps" && generic_props {
            ts.push_str(if hoisted {
                GENERIC_PROPS_HOISTED
            } else {
                GENERIC_PROPS_EMBEDDED
            });
        } else if helper.name == "useTemplateRef" && typed_refs {
            ts.push_str(if hoisted {
                "  const useTemplateRef = __vize_useTemplateRef as unknown as __VizeUseTemplateRef;\n"
            } else {
                "  const useTemplateRef = (undefined as unknown as __VizeUseTemplateRef); void ((_key: string) => useTemplateRef(_key));\n"
            });
        } else {
            ts.push_str(if hoisted {
                helper.hoisted
            } else {
                helper.embedded
            });
        }
    }
    ts.push_str("  // Mark compiler macros as used\n  ");
    let mut first = true;
    for helper in SETUP_MACRO_HELPERS {
        if shadowed.contains(helper.name) {
            continue;
        }
        if !first {
            ts.push(' ');
        }
        append!(*ts, "void {};", helper.name);
        first = false;
    }
}

fn shadowed_helpers(summary: &Croquis, source: Option<&str>) -> SetupHelperPlan {
    let mut plan = SetupHelperPlan {
        shadowed: FxHashSet::default(),
        vue_template_ref_import: false,
        macro_results: Vec::new(),
    };
    if !SETUP_MACRO_HELPERS
        .iter()
        .any(|helper| summary.bindings.bindings.contains_key(helper.name))
        && summary.macros.define_props().is_none()
        && summary.macros.define_emits().is_none()
    {
        return plan;
    }
    let Some(source) = source else {
        return plan;
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let parsed = if parsed.panicked || !parsed.diagnostics.is_empty() {
        Parser::new(&allocator, source, SourceType::tsx()).parse()
    } else {
        parsed
    };
    if parsed.panicked {
        return plan;
    }
    let built = SemanticBuilder::new().build(&parsed.program);
    let scoping = built.semantic.scoping();
    for symbol in scoping.symbol_ids() {
        if scoping.symbol_scope_id(symbol) == scoping.root_scope_id()
            && scoping.symbol_flags(symbol).is_value()
            && SETUP_MACRO_HELPERS
                .iter()
                .any(|helper| helper.name == scoping.symbol_name(symbol))
        {
            plan.shadowed
                .insert(CompactString::from(scoping.symbol_name(symbol)));
        }
    }
    // Vue's runtime helper is intentionally specialized from the template ref
    // registry. Other authored values (including imports from other modules)
    // retain their actual declarations and signatures.
    for statement in &parsed.program.body {
        if let Statement::ImportDeclaration(import) = statement
            && import.source.value == "vue"
            && import.import_kind != ImportOrExportKind::Type
            && let Some(specifiers) = &import.specifiers
        {
            for specifier in specifiers {
                if let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier
                    && named.import_kind != ImportOrExportKind::Type
                    && named.local.name == "useTemplateRef"
                    && named.imported.name() == "useTemplateRef"
                {
                    plan.shadowed.remove("useTemplateRef");
                    plan.vue_template_ref_import = true;
                }
            }
        }
    }
    plan.macro_results = super::macro_results::collect(&parsed.program, &plan.shadowed, summary);
    plan
}

const GENERIC_PROPS_HOISTED: &str = r#"  const defineProps = __vize_defineProps as {
    <_T = unknown>(): __DefineProps<__LooseRequired<_T>, Extract<__VizeDefinePropsBooleanKeys<_T>, keyof __LooseRequired<_T>>>;
    <const _T extends readonly string[]>(_props: _T): { [K in _T[number]]?: any };
    <const _T extends Record<string, any>>(_props: _T): __RuntimePropShape<_T>;
  };
"#;
const GENERIC_PROPS_EMBEDDED: &str = r#"  function defineProps<_T = unknown>(): __DefineProps<__LooseRequired<_T>, Extract<__VizeDefinePropsBooleanKeys<_T>, keyof __LooseRequired<_T>>>;
  function defineProps<const _T extends readonly string[]>(_props: _T): { [K in _T[number]]?: any };
  function defineProps<const _T extends Record<string, any>>(_props: _T): __RuntimePropShape<_T>;
  function defineProps(_props?: any) { void _props; return undefined as any; }
"#;

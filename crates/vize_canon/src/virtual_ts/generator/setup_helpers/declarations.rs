//! Emit helpers by lexical ownership; never rewrite a generated block afterward.

use super::super::template_ref_keys::TemplateRefUsage;
use crate::virtual_ts::helpers::setup_macros::{
    SETUP_MACRO_HELPERS, VUE_SETUP_HELPERS, VUE_SETUP_HELPERS_HOISTED,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, append};
use vize_croquis::Croquis;

pub(crate) struct SetupHelperPlan {
    shadowed: FxHashSet<CompactString>,
    pub(crate) macro_results: Vec<CompactString>,
    pub(super) emit_calls: Vec<super::emit_navigation::OwnedEmitCall>,
    pub(in crate::virtual_ts::generator) template_refs: TemplateRefUsage,
    active_template_ref: bool,
}

impl SetupHelperPlan {
    pub(crate) fn collect(
        summary: &Croquis,
        source: Option<&str>,
        preserve_event_navigation: bool,
    ) -> Self {
        shadowed_helpers(summary, source, preserve_event_navigation)
    }

    #[cfg(feature = "native")]
    pub(in crate::virtual_ts::generator) fn emit_error_recovery(
        &self,
        ts: &mut String,
        hoisted: bool,
    ) {
        emit(ts, self, hoisted, false, false);
    }

    pub(crate) fn emit_import_anchors(&self, ts: &mut String) {
        self.template_refs.emit_import_anchors(ts);
    }

    pub(crate) fn template_emit_initializer(
        &self,
        ts: &mut String,
        summary: &Croquis,
        generic_parameters: Option<&crate::virtual_ts::generic_parameters::GenericParameters>,
    ) -> Option<String> {
        let emits = summary
            .macros
            .define_emits()
            .filter(|_| !self.shadowed.contains("defineEmits"));
        let base = emits.and_then(|emits| {
            if let Some(type_args) = emits.type_args.as_deref() {
                let inner = super::super::emits::inner_type_of(type_args);
                Some(vize_carton::cstr!("__EmitFn<{inner}>"))
            } else if let Some(args) = emits.runtime_args.as_ref() {
                append!(
                    *ts,
                    "    const __vize_template_emit = defineEmits({args});\n"
                );
                Some(String::from("typeof __vize_template_emit"))
            } else {
                None
            }
        });
        let models = (!self.shadowed.contains("defineModel"))
            .then(|| super::super::emits::template_model_emit_type(summary, generic_parameters))
            .flatten();
        let ty = match (base, models) {
            (Some(base), Some(models)) => vize_carton::cstr!("{base} & {models}"),
            (Some(ty), None) | (None, Some(ty)) => ty,
            (None, None) => return None,
        };
        Some(vize_carton::cstr!("(void 0) as unknown as {ty}"))
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
    if shadowed.is_empty() && !generic_props && !typed_refs && !plan.active_template_ref {
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
    if typed_refs {
        plan.template_refs.emit_inference_helpers(ts);
    }
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

fn shadowed_helpers(
    summary: &Croquis,
    source: Option<&str>,
    preserve_event_navigation: bool,
) -> SetupHelperPlan {
    let mut plan = SetupHelperPlan {
        shadowed: FxHashSet::default(),
        macro_results: Vec::new(),
        emit_calls: Vec::new(),
        template_refs: TemplateRefUsage::default(),
        // Escapes may spell the identifier or imported name without its literal
        // bytes. Keep this conservative even when recovery cannot build an AST.
        active_template_ref: source.is_some_and(|source| {
            source.contains("useTemplateRef") || source.as_bytes().contains(&b'\\')
        }),
    };
    if !SETUP_MACRO_HELPERS
        .iter()
        .any(|helper| crate::virtual_ts::script_facts::contains_binding(summary, helper.name))
        && summary.macros.define_props().is_none()
        && summary.macros.define_emits().is_none()
        && !plan.active_template_ref
    {
        return plan;
    }
    let Some(source) = source else {
        return plan;
    };
    // This is a runtime composable, not a compiler macro. Never invent or
    // replace its ambient contract: custom globals and missing imports must
    // keep their authored types and diagnostics, including parser recovery.
    if plan.active_template_ref {
        plan.shadowed.insert(CompactString::from("useTemplateRef"));
    }
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
    if plan.active_template_ref {
        plan.template_refs = TemplateRefUsage::collect(&parsed.program, scoping, source);
    }
    plan.macro_results = super::macro_results::collect(&parsed.program, &plan.shadowed, summary);
    if preserve_event_navigation && parsed.diagnostics.is_empty() && built.diagnostics.is_empty() {
        plan.emit_calls = super::emit_navigation::collect(
            &parsed.program,
            scoping,
            source,
            &plan.shadowed,
            summary,
        );
    }
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

#[cfg(test)]
mod template_ref_fallback_tests {
    use super::{
        SETUP_MACRO_HELPERS, SetupHelperPlan, VUE_SETUP_HELPERS, VUE_SETUP_HELPERS_HOISTED, emit,
    };
    use vize_carton::String;

    #[test]
    fn possible_global_ref_calls_never_receive_an_invented_runtime_contract() {
        for source in [
            "function nested() { useTemplateRef<unknown>('missing'); }",
            "function nested() { useTemplateRef<never>('missing'); }",
            "function nested() { useTemplateRe\\u0066<unknown>('missing'); }",
            "const target = useTemplateRef<unknown>(",
        ] {
            let plan =
                SetupHelperPlan::collect(&vize_croquis::Croquis::default(), Some(source), false);
            for hoisted in [false, true] {
                let mut generated = String::default();
                emit(&mut generated, &plan, hoisted, false, false);
                let helper = SETUP_MACRO_HELPERS
                    .iter()
                    .find(|helper| helper.name == "useTemplateRef")
                    .unwrap();
                let expected = if hoisted {
                    VUE_SETUP_HELPERS_HOISTED
                } else {
                    VUE_SETUP_HELPERS
                }
                .replace(
                    if hoisted {
                        helper.hoisted
                    } else {
                        helper.embedded
                    },
                    "",
                )
                .replace(" void useTemplateRef;", "");
                assert_eq!(generated, expected);
            }
        }
    }

    #[test]
    fn unrelated_document_helper_packets_remain_byte_identical() {
        let plan = SetupHelperPlan::collect(
            &vize_croquis::Croquis::default(),
            Some("const count = 1;"),
            false,
        );
        let mut generated = String::default();
        emit(&mut generated, &plan, false, false, false);
        assert_eq!(generated, VUE_SETUP_HELPERS);
    }
}

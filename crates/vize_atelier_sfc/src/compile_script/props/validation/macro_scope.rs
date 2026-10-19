//! Scope validation for runtime arguments of hoisted script-setup macros.

use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::symbol::SymbolFlags;
use vize_carton::cstr;

use crate::script::{ScriptCompileContext, define_model_prop_option_spans};
use crate::types::{BindingType, SfcDescriptor, SfcError};

use super::block_location_for_span;

/// Cheap strict-superset filter for the script-setup semantic validators.
/// Callers may skip parsing only when this returns false.
pub fn script_setup_has_semantic_validator_candidates(content: &str) -> bool {
    let has_local_declaration = [
        "const",
        "let",
        "var",
        "using",
        "function",
        "class",
        "enum",
        "namespace",
        "module",
    ]
    .iter()
    .any(|keyword| content.contains(keyword));
    let has_hoisted_macro = [
        "withDefaults",
        "defineProps",
        "defineEmits",
        "defineOptions",
        "defineSlots",
        "defineModel",
    ]
    .iter()
    .any(|macro_name| content.contains(macro_name));
    has_local_declaration && has_hoisted_macro
}

#[derive(Clone, Copy)]
struct HoistedMacroSpan {
    name: &'static str,
    start: usize,
    end: usize,
}

fn hoisted_macro_spans(ctx: &ScriptCompileContext) -> Vec<HoistedMacroSpan> {
    let mut spans = Vec::with_capacity(6 + ctx.macros.define_models.len());
    if let Some(call) = &ctx.macros.with_defaults {
        spans.push(HoistedMacroSpan {
            name: "withDefaults",
            start: call.start,
            end: call.end,
        });
    }
    for (name, call) in [
        ("defineProps", ctx.macros.define_props.as_ref()),
        ("defineEmits", ctx.macros.define_emits.as_ref()),
        ("defineOptions", ctx.macros.define_options.as_ref()),
        ("defineSlots", ctx.macros.define_slots.as_ref()),
    ] {
        if let Some(call) = call {
            spans.push(HoistedMacroSpan {
                name,
                start: call.start,
                end: call.end,
            });
        }
    }
    spans.extend(ctx.macros.define_models.iter().flat_map(|call| {
        define_model_prop_option_spans(ctx.source.as_str(), call)
            .into_iter()
            .map(|(start, end)| HoistedMacroSpan {
                name: "defineModel",
                start,
                end,
            })
    }));
    spans
}

/// Only runtime arguments are hoisted. The macro callee remains a compiler
/// token, and withDefaults' first argument is the nested defineProps call.
fn runtime_argument_spans(
    program: &Program<'_>,
    candidates: &[HoistedMacroSpan],
) -> Vec<HoistedMacroSpan> {
    struct Arguments<'a> {
        candidates: &'a [HoistedMacroSpan],
        spans: Vec<HoistedMacroSpan>,
    }
    impl<'a> Visit<'a> for Arguments<'_> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            for candidate in self.candidates {
                if candidate.start != call.span.start as usize
                    || candidate.end != call.span.end as usize
                {
                    continue;
                }
                for argument in call
                    .arguments
                    .iter()
                    .skip(usize::from(candidate.name == "withDefaults"))
                {
                    let span = argument.span();
                    self.spans.push(HoistedMacroSpan {
                        name: candidate.name,
                        start: span.start as usize,
                        end: span.end as usize,
                    });
                }
            }
            walk_call_expression(self, call);
        }
    }
    let mut visitor = Arguments {
        candidates,
        spans: candidates
            .iter()
            .copied()
            .filter(|span| span.name == "defineModel")
            .collect(),
    };
    visitor.visit_program(program);
    visitor.spans
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

fn contains_identifier(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let bytes = source.as_bytes();
        let end = start + name.len();
        let before = start.checked_sub(1).and_then(|prev| bytes.get(prev));
        before.is_none_or(|&byte| !is_identifier_byte(byte))
            && bytes.get(end).is_none_or(|&byte| !is_identifier_byte(byte))
    })
}

/// Reject setup-local runtime values used by macros whose arguments are
/// hoisted to module scope. Imports and literal constants remain valid.
pub(crate) fn validate_macro_scope_references(
    ctx: &ScriptCompileContext,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    let spans = hoisted_macro_spans(ctx);
    if !has_possible_local_macro_reference(ctx, &spans) {
        return Ok(());
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &ctx.source, SourceType::ts()).parse();
    validate_macro_scope_references_in_program(ctx, &parsed.program, block_start, sfc_source)
}

pub(crate) fn validate_macro_scope_and_props(
    ctx: &ScriptCompileContext,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    validate_macro_scope_references(ctx, block_start, sfc_source)?;
    super::validate_props_destructure_default_types(ctx, block_start, sfc_source)
}

pub(crate) fn validate_macro_scope_for_descriptor(
    ctx: &ScriptCompileContext,
    program: Option<&Program<'_>>,
    descriptor: &SfcDescriptor<'_>,
) -> Result<(), SfcError> {
    // Without a `<script setup>` block there are no setup-local values to
    // reject, so there is nothing to validate.
    let Some(setup) = descriptor.script_setup.as_ref() else {
        return Ok(());
    };
    match program {
        Some(program) => validate_macro_scope_references_in_program(
            ctx,
            program,
            setup.loc.start,
            &descriptor.source,
        ),
        None => validate_macro_scope_references(ctx, setup.loc.start, &descriptor.source),
    }
}

fn has_possible_local_macro_reference(
    ctx: &ScriptCompileContext,
    spans: &[HoistedMacroSpan],
) -> bool {
    // The legacy classifier omits enum/namespace and `using`. A false positive
    // here only costs one semantic pass and can never suppress a diagnostic.
    if ["enum", "namespace", "module", "using"]
        .iter()
        .any(|keyword| ctx.source.contains(keyword))
    {
        return true;
    }
    ctx.bindings.bindings.iter().any(|(name, binding_type)| {
        !matches!(binding_type, BindingType::LiteralConst)
            && spans.iter().any(|span| {
                ctx.source
                    .get(span.start..span.end)
                    .is_some_and(|source| contains_identifier(source, name))
            })
    })
}

pub(crate) fn validate_macro_scope_references_in_program(
    ctx: &ScriptCompileContext,
    program: &Program<'_>,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    let spans = hoisted_macro_spans(ctx);
    if !has_possible_local_macro_reference(ctx, &spans) {
        return Ok(());
    }
    let spans = runtime_argument_spans(program, &spans);
    if spans.is_empty() {
        return Ok(());
    }
    // `Semantic::reference_span` reads `Semantic::nodes`, and OXC 0.142 made the
    // node store opt-in (empty by default so the compiler pipeline does not pay
    // for random access it never uses). Without this the lookup below indexes an
    // empty `IndexVec` and panics.
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
        .semantic;
    let scoping = semantic.scoping();
    let root_bindings = scoping.get_bindings(scoping.root_scope_id());
    let mut invalid = None;

    for (name, symbol_id) in root_bindings {
        let flags = scoping.symbol_flags(*symbol_id);
        if flags.intersects(SymbolFlags::Import | SymbolFlags::TypeImport | SymbolFlags::Ambient)
            || (flags.is_type() && !flags.intersects(SymbolFlags::Value))
            || matches!(
                ctx.bindings.bindings.get(name.as_str()),
                Some(BindingType::LiteralConst)
            )
        {
            continue;
        }
        for reference in semantic.symbol_references(*symbol_id) {
            let reference_flags = reference.flags();
            if reference_flags.is_type() || reference_flags.is_value_as_type() {
                continue;
            }
            let reference_span = semantic.reference_span(reference);
            for macro_span in &spans {
                if reference_span.start as usize >= macro_span.start
                    && reference_span.end as usize <= macro_span.end
                {
                    let candidate = (
                        reference_span.start as usize,
                        reference_span.end as usize,
                        name.as_str(),
                        macro_span.name,
                    );
                    if invalid
                        .is_none_or(|current: (usize, usize, &str, &str)| candidate.0 < current.0)
                    {
                        invalid = Some(candidate);
                    }
                }
            }
        }
    }

    let Some((start, end, name, macro_name)) = invalid else {
        return Ok(());
    };
    Err(SfcError {
        message: cstr!(
            "`{macro_name}()` in <script setup> cannot reference locally declared variable `{name}` because its arguments are hoisted outside setup(). Move the initialization to a normal <script> block or import it from another module."
        ),
        code: Some("SCRIPT_SETUP_MACRO_SCOPE".into()),
        loc: Some(block_location_for_span(
            sfc_source,
            block_start + start,
            block_start + end,
        )),
    })
}

#[cfg(test)]
mod tests {
    #![expect(clippy::string_slice, reason = "tests assert by panicking")]
    use super::super::validate_script_setup_semantics;
    use super::script_setup_has_semantic_validator_candidates;

    #[test]
    fn semantic_validator_prefilter_is_a_strict_superset() {
        for source in [
            "const local = makeDefault()\nwithDefaults(defineProps(), { value: local })",
            "const\tlocal = makeDefault()\nwithDefaults(defineProps(), { value: local })",
            "function local() {}\ndefineOptions({ setup: local })",
            "const { value = 0 } = defineProps<{ value?: string }>()",
        ] {
            assert!(
                script_setup_has_semantic_validator_candidates(source),
                "validator candidate must not be skipped: {source}"
            );
        }
        for source in [
            "const value = compute()",
            "defineProps<{ value: string }>()",
            "import { fallback } from './defaults'\nwithDefaults(defineProps(), { value: fallback })",
        ] {
            assert!(
                !script_setup_has_semantic_validator_candidates(source),
                "obviously irrelevant source should keep the fast path: {source}"
            );
        }
    }

    #[test]
    fn rejects_setup_local_values_in_every_hoisted_runtime_macro() {
        let cases = [
            (
                "withDefaults direct default",
                "const local = makeDefault()\nwithDefaults(defineProps<{ value?: string }>(), { value: local })",
                "withDefaults",
            ),
            (
                "withDefaults factory closure",
                "const local = makeDefault()\nwithDefaults(defineProps<{ value?: string }>(), { value: () => local })",
                "withDefaults",
            ),
            (
                "defineProps runtime validator",
                "const local = makeValidator()\ndefineProps({ value: { validator: local } })",
                "defineProps",
            ),
            (
                "defineProps runtime constructor",
                "class local {}\ndefineProps({ value: { type: local } })",
                "defineProps",
            ),
            (
                "defineEmits runtime validator",
                "const local = makeValidator()\ndefineEmits({ change: local })",
                "defineEmits",
            ),
            (
                "defineOptions component option",
                "const local = makeName()\ndefineOptions({ name: local })",
                "defineOptions",
            ),
            (
                "function declaration",
                "function local() {}\ndefineOptions({ setup: local })",
                "defineOptions",
            ),
            (
                "runtime-dependent enum value",
                "const seed = runtime()\nenum local { Ready = seed }\ndefineOptions({ ready: local.Ready })",
                "defineOptions",
            ),
            (
                "namespace runtime value",
                "namespace local { export const value = 1 }\ndefineOptions({ value: local.value })",
                "defineOptions",
            ),
            (
                "defineModel runtime option",
                "const local = makeValidator()\ndefineModel<string>({ validator: local })",
                "defineModel",
            ),
            (
                "defineModel prop option beside runtime accessor",
                "const local = makeValidator()\ndefineModel<string>({ validator: local, get: value => value })",
                "defineModel",
            ),
        ];

        for (label, source, macro_name) in cases {
            let Err(error) = validate_script_setup_semantics(source) else {
                panic!("{label} should reject a setup-local reference");
            };
            assert_eq!(
                error.code.as_deref(),
                Some("SCRIPT_SETUP_MACRO_SCOPE"),
                "{label}"
            );
            assert_eq!(
                error.message,
                vize_carton::cstr!(
                    "`{macro_name}()` in <script setup> cannot reference locally declared variable \
                     `local` because its arguments are hoisted outside setup(). Move the \
                     initialization to a normal <script> block or import it from another module."
                ),
                "{label}"
            );
            let loc = error.loc.expect("invalid reference should have a location");
            assert_eq!(
                &source[loc.start..loc.end],
                "local",
                "{label} should highlight only the invalid identifier"
            );
        }
    }

    #[test]
    fn allows_values_that_remain_valid_after_macro_hoisting() {
        let cases = [
            (
                "macro callee is not a hoisted argument",
                "const defineProps = () => ({ value: 1 }); const result = defineProps();",
            ),
            (
                "nested macro callee is not a withDefaults default",
                "const defineProps = () => ({}); const result = withDefaults(defineProps<{ value?: number }>(), { value: 1 });",
            ),
            (
                "imported binding",
                "import { fallback } from './defaults'\nwithDefaults(defineProps<{ value?: string }>(), { value: fallback })",
            ),
            (
                "literal setup constant",
                "const fallback = 'ready'\nwithDefaults(defineProps<{ value?: string }>(), { value: fallback })",
            ),
            (
                "static enum runtime value",
                "enum Step { Name, General = 1 + 1 }\nwithDefaults(defineProps<{ step?: Step }>(), { step: Step.Name })",
            ),
            (
                "global binding",
                "defineProps({ value: { default: Math.random() } })",
            ),
            (
                "shadowed callback parameter",
                "const fallback = makeDefault()\nwithDefaults(defineProps<{ value?: string }>(), { value: (fallback) => fallback })",
            ),
            (
                "defineExpose setup local",
                "const exposed = makeValue()\ndefineExpose({ exposed })",
            ),
            (
                "local enum in a type argument",
                "enum LocalEnum { Ready }\ndefineProps<{ value?: LocalEnum }>()",
            ),
            (
                "local class in a type argument",
                "class LocalClass {}\ndefineProps<{ value?: LocalClass }>()",
            ),
            (
                "local value in a type query",
                "const runtime = { ready: true }\ndefineProps<{ value?: typeof runtime }>()",
            ),
            (
                "defineModel runtime accessors",
                "const local = computed(() => 1)\ndefineModel<number>({ get(value) { return value ?? local.value }, set(value) { return Math.max(local.value, value) } })",
            ),
            (
                "defineModel modifiers binding in setter",
                "const [model, modifiers] = defineModel<string>({ set(value) { return modifiers.trim ? value.trim() : value } })",
            ),
        ];

        for (label, source) in cases {
            validate_script_setup_semantics(source)
                .unwrap_or_else(|error| panic!("{label} should remain valid: {error:?}"));
        }
    }

    #[test]
    fn reports_the_exact_authored_identifier_range() {
        let source =
            "const items = []\n\nwithDefaults(defineProps<{\n  items?: string[]\n}>(), { items })";
        let error = validate_script_setup_semantics(source).expect_err("local default must fail");
        let loc = error.loc.expect("scope error should be located");

        assert_eq!(&source[loc.start..loc.end], "items");
        assert_eq!((loc.start_line, loc.start_column), (5, 9));
        assert_eq!((loc.end_line, loc.end_column), (5, 14));
    }
}

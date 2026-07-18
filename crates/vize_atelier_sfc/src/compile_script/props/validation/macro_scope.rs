//! Scope validation for runtime arguments of hoisted script-setup macros.

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolFlags;
use vize_carton::cstr;

use crate::script::ScriptCompileContext;
use crate::types::{BindingType, SfcDescriptor, SfcError};

use super::block_location_for_span;

#[cfg(test)]
mod tests;

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
    spans.extend(
        ctx.macros
            .define_models
            .iter()
            .map(|call| HoistedMacroSpan {
                name: "defineModel",
                start: call.start,
                end: call.end,
            }),
    );
    spans
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

fn contains_identifier(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let bytes = source.as_bytes();
        let end = start + name.len();
        (start == 0 || !is_identifier_byte(bytes[start - 1]))
            && (end == bytes.len() || !is_identifier_byte(bytes[end]))
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
    let setup = descriptor
        .script_setup
        .as_ref()
        .expect("script setup exists while compiling its context");
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
    let semantic = SemanticBuilder::new().build(program).semantic;
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

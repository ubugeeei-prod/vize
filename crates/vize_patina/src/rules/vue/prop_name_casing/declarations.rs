//! Prop declarations and the SFC-absolute range each one occupies.
//!
//! Croquis reports macro ranges against the *analysis* view of the script,
//! which concatenates a sibling plain `<script>` in front of `<script setup>`.
//! A diagnostic has to address the file, so each range is shifted back out of
//! that combined view and then into the declaring block's position in the SFC —
//! the same normalisation [`super::super::no_unused_properties`] performs.

use crate::context::LintContext;
use crate::rules::script::{
    NoDupeKeysModuleScript, collect_options_props, no_dupe_keys, script_source_type,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use super::super::macro_type_argument::macro_type_argument_span;

/// A declared prop and where it is written.
pub(super) struct Declaration {
    pub(super) name: vize_l0::CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Every prop `defineProps` declares, with SFC-absolute ranges.
pub(super) fn declarations(ctx: &LintContext<'_>) -> Vec<Declaration> {
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return Vec::new();
    };
    let mut declarations = Vec::new();
    if let Some(script) = descriptor
        .script
        .as_ref()
        .filter(|block| block.src.is_none())
    {
        let allocator = Allocator::default();
        let source_type = match script.lang.as_deref().map(str::trim) {
            Some("tsx") => SourceType::tsx(),
            Some("jsx") => SourceType::jsx(),
            _ => script_source_type(),
        };
        let parsed = Parser::new(&allocator, script.content.as_ref(), source_type).parse();
        if !parsed.panicked
            && parsed.diagnostics.is_empty()
            && let Some(props) = collect_options_props(&parsed.program)
        {
            let offset = script.loc.start as u32;
            for prop in props.object_props() {
                declarations.push(Declaration {
                    name: prop.name.into(),
                    start: offset + prop.span.start,
                    end: offset + prop.span.end,
                });
            }
            for name in props.array_names() {
                declarations.push(Declaration {
                    name: name.value.as_str().into(),
                    start: offset + name.span.start,
                    end: offset + name.span.end,
                });
            }
        }
    }
    let script_setup = descriptor.script_setup.as_ref();
    let plain_script = descriptor.script.as_ref();
    let Some(declaring_block) = script_setup.or(plain_script) else {
        return Vec::new();
    };
    let declaring_offset = declaring_block.loc.start as u32;
    let setup_shift = match (plain_script, script_setup) {
        (Some(plain), Some(_)) => plain.content.len() as u32 + 1,
        _ => 0,
    };

    if let Some(analysis) = ctx.analysis() {
        append_analyzed_props(
            &mut declarations,
            analysis,
            declaring_block.content.as_ref(),
            declaring_offset,
            setup_shift,
        );
        return declarations;
    }
    // A template-less SFC never receives croquis on the SFC pass. Build the
    // same filename-aware summary the template pass would, so `import type`
    // props resolve. A local croquis cannot be stored on the context: its
    // analysis borrow is tied to the source lifetime.
    if script_setup.is_some() {
        let allocator = Allocator::default();
        let scripts = crate::linter::sfc_scripts::lint_scripts(&allocator, descriptor);
        let owned = crate::linter::analyze_descriptor_for_lint(
            &scripts,
            None,
            Some(ctx.filename),
            false,
            false,
        );
        if append_analyzed_props(
            &mut declarations,
            &owned,
            declaring_block.content.as_ref(),
            declaring_offset,
            setup_shift,
        ) {
            return declarations;
        }
    }
    if let (Some(block), Some(scripts)) = (script_setup, ctx.sfc_scripts())
        && let Some(parsed) = scripts.script_setup()
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        let built = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(&parsed.program);
        if built.diagnostics.is_empty()
            && !no_dupe_keys::setup::macro_shadowed(&parsed.program, &built.semantic, "defineProps")
        {
            for candidate in no_dupe_keys::setup::candidates(&parsed.program) {
                for name in no_dupe_keys::setup::props::names(
                    candidate.call,
                    &parsed.program,
                    &built.semantic,
                    NoDupeKeysModuleScript::Absent,
                    None,
                    block.content.as_ref(),
                    None,
                ) {
                    let span = no_dupe_keys::setup::props::declaration_span(candidate.call, &name)
                        .unwrap_or(candidate.call.span);
                    declarations.push(Declaration {
                        name,
                        start: block.loc.start as u32 + span.start,
                        end: block.loc.start as u32 + span.end,
                    });
                }
            }
        }
    }
    declarations
}

/// Record props from a resolved `defineProps` call. Returns false when that
/// call is absent, so the same-file walk can still see a parse the summary missed.
fn append_analyzed_props(
    declarations: &mut Vec<Declaration>,
    analysis: &vize_croquis::Croquis,
    declaring_content: &str,
    declaring_offset: u32,
    setup_shift: u32,
) -> bool {
    let Some(call) = analysis.macros.define_props() else {
        return false;
    };
    let type_argument_span = macro_type_argument_span(declaring_content, call, setup_shift);
    declarations.extend(analysis.macros.props().iter().map(|prop| {
        let (start, end) = analysis
            .macros
            .prop_declaration(prop.name.as_str())
            .or(type_argument_span)
            .unwrap_or((call.start, call.end));
        Declaration {
            name: prop.name.clone(),
            start: declaring_offset + start.saturating_sub(setup_shift),
            end: declaring_offset + end.saturating_sub(setup_shift),
        }
    }));
    true
}

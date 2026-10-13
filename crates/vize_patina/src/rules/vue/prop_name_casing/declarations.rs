//! Prop declarations and the SFC-absolute range each one occupies.
//!
//! Croquis reports macro ranges against the *analysis* view of the script,
//! which concatenates a sibling plain `<script>` in front of `<script setup>`.
//! A diagnostic has to address the file, so each range is shifted back out of
//! that combined view and then into the declaring block's position in the SFC —
//! the same normalisation [`super::super::no_unused_properties`] performs.

use crate::context::LintContext;
use crate::rules::script::{collect_options_props, script_source_type};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

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

    let Some(analysis) = ctx.analysis() else {
        return declarations;
    };
    let Some(call) = analysis.macros.define_props() else {
        return declarations;
    };
    declarations.extend(analysis.macros.props().iter().map(|prop| {
        let (start, end) = analysis
            .macros
            .prop_declaration(prop.name.as_str())
            .unwrap_or((call.start, call.end));
        Declaration {
            name: prop.name.clone(),
            start: declaring_offset + start.saturating_sub(setup_shift),
            end: declaring_offset + end.saturating_sub(setup_shift),
        }
    }));
    declarations
}

//! Preserve the authored SFC header for the existing template compiler lanes.

use super::SfcScriptOutputMode;
use crate::types::{
    BlockLocation, SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult,
    SfcDescriptor, SfcError,
};
use std::borrow::Cow;
use vize_atelier_core::{
    Allocator, ParserOptions, PropNode, TemplateChildNode,
    parser::parse_with_options_and_template_syntax,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_s0::String;

pub(super) fn compile_sfc_inner(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
) -> Result<SfcCompileResult, SfcError> {
    let descriptor = prepare_root_patterned_template(
        descriptor,
        options
            .template
            .compiler_options
            .as_ref()
            .is_some_and(|options| options.experimental_patterned_template),
        options
            .template
            .compiler_options
            .as_ref()
            .is_some_and(|options| options.experimental_in_tag_comments),
        template_syntax,
    )?;
    let descriptor = super::pug::prepare_pug_template(&descriptor)?;
    super::super::compile_sfc_inner(
        &descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
    )
}

/// Prepare an offset-preserving compiler view without changing the parsed SFC.
#[doc(hidden)]
pub fn prepare_root_patterned_template<'d, 's>(
    descriptor: &'d SfcDescriptor<'s>,
    enabled: bool,
    in_tag_comments: bool,
    template_syntax: TemplateSyntaxMode,
) -> Result<Cow<'d, SfcDescriptor<'s>>, SfcError> {
    let Some(template) = descriptor.template.as_ref().filter(|t| t.has_root_match()) else {
        return Ok(Cow::Borrowed(descriptor));
    };
    let error = |message: &str, start, end| SfcError {
        message: message.into(),
        code: Some("V_MATCH_SYNTAX".into()),
        loc: descriptor
            .source
            .get(..start)
            .zip(descriptor.source.get(..end))
            .map(|_| location(&descriptor.source, start, end)),
    };
    if template.src.is_some() || template.lang.as_deref().is_some_and(|lang| lang != "html") {
        return Err(error(
            "Root v-match requires an inline HTML template.",
            template.loc.tag_start,
            template.loc.start,
        ));
    }
    let source = descriptor.source.as_ref();
    let Some(original) = source
        .get(template.loc.tag_start..template.loc.tag_end)
        .filter(|_| {
            template.loc.tag_start <= template.loc.start
                && template.loc.end <= template.loc.tag_end
                && source.get(template.loc.start..template.loc.end)
                    == Some(template.content.as_ref())
        })
    else {
        return Err(error(
            "Root v-match requires preserved SFC source metadata.",
            template.loc.tag_start,
            template.loc.start,
        ));
    };
    let allocator = Allocator::default();
    let (root, diagnostics) = parse_with_options_and_template_syntax(
        &allocator,
        original,
        ParserOptions {
            experimental_in_tag_comments: in_tag_comments,
            ..Default::default()
        },
        template_syntax,
    );
    let Some(TemplateChildNode::Element(element)) = root.children.first() else {
        return Err(error(
            "Root v-match requires a template element.",
            template.loc.tag_start,
            template.loc.start,
        ));
    };
    let mut retained = None;
    for prop in &element.props {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name != "match" {
            continue;
        }
        let start = template.loc.tag_start + dir.loc.span.start as usize;
        let end = template.loc.tag_start + dir.loc.span.end as usize;
        if !enabled {
            return Err(error(
                "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`.",
                start,
                end,
            ));
        }
        if retained.is_some()
            || dir.arg.is_some()
            || !dir.modifiers.is_empty()
            || template
                .attrs
                .get("v-match")
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(error(
                "v-match requires one subject expression without arguments or modifiers.",
                start,
                end,
            ));
        }
        retained = Some(dir.loc.span);
    }
    let Some(retained) = retained else {
        return Err(error(
            "Root v-match requires a parsed subject directive.",
            template.loc.tag_start,
            template.loc.start,
        ));
    };
    // Header metadata is masked below; body diagnostics still belong to the
    // configured compiler lane, including its custom-element parser options.
    let header_end = template.loc.start - template.loc.tag_start;
    if let Some(fatal) = diagnostics.iter().find(|diagnostic| {
        !diagnostic.is_recoverable()
            && diagnostic
                .loc
                .as_ref()
                .is_none_or(|loc| (loc.span.start as usize) < header_end)
    }) {
        let (start, end) = fatal.loc.as_ref().map_or((0, header_end), |loc| {
            (loc.span.start as usize, loc.span.end as usize)
        });
        let mut fatal = error(
            &fatal.message,
            template.loc.tag_start + start,
            template.loc.tag_start + end,
        );
        fatal.code = Some("TEMPLATE_ERROR".into());
        return Err(fatal);
    }
    // SFC metadata is not a template directive. Blank those parsed attribute
    // spans, retaining every byte offset and line break for diagnostics.
    let mut content = String::with_capacity(original.len());
    let mut cursor = 0;
    for prop in &element.props {
        let span = match prop {
            PropNode::Attribute(attr) => attr.loc.span,
            PropNode::Directive(dir) => dir.loc.span,
        };
        if span == retained {
            continue;
        }
        content.push_str(
            original
                .get(cursor..span.start as usize)
                .unwrap_or_default(),
        );
        let masked = original.get(span.start as usize..span.end as usize);
        for byte in masked.unwrap_or_default().bytes() {
            content.push(if matches!(byte, b'\r' | b'\n') {
                char::from(byte)
            } else {
                ' '
            });
        }
        cursor = span.end as usize;
    }
    content.push_str(original.get(cursor..).unwrap_or_default());
    let mut descriptor = descriptor.clone();
    if let Some(template) = descriptor.template.as_mut() {
        template.content = Cow::Owned(content.into());
        template.loc = location(source, template.loc.tag_start, template.loc.tag_end);
    }
    Ok(Cow::Owned(descriptor))
}

fn location(source: &str, start: usize, end: usize) -> BlockLocation {
    let position = |offset| {
        let prefix = source.get(..offset).unwrap_or_default();
        (
            prefix.bytes().filter(|&b| b == b'\n').count() + 1,
            prefix.rfind('\n').map_or(offset + 1, |line| offset - line),
        )
    };
    let (start_line, start_column) = position(start);
    let (end_line, end_column) = position(end);
    BlockLocation {
        start,
        end,
        tag_start: start,
        tag_end: end,
        start_line,
        start_column,
        end_line,
        end_column,
    }
}

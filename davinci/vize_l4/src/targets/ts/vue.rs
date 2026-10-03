//! Original setup Modules and resolved Vue interpolation reads, without a context shim.
//!
//! This target consumes the actual native SFC custody receipt. Its selected
//! Programs, Component and expression observations stay borrowed throughout
//! emission and diagnostic mapping. It does not parse or walk an AST again.

use alloc::vec::Vec;
use vize_l0::{
    Span,
    config::{VueDialect, VueVersion},
    line_index::utf16_offset,
};
use vize_l1_to_l2::{native::NativeEmbed, native_file::NativeSfc};
use vize_l2::{
    expr::ExprRef, file::FileArtifact, lang::js::VueSetup, op::Op, resolution::Usage, walk::NodeRef,
};

use super::{MappingError, SourceKind};
use crate::write::{EmitDocument, LinkSink, NoLinks, Recorded, Writer};

/// Refuse unfinished source families instead of producing a partial checker document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueProjectionError {
    UnsupportedDialect,
    UnsupportedScripts,
    UnsupportedStyles,
    UnsupportedTemplate,
    UnsupportedTemplateBindings,
    Custody,
    SourceTooLarge,
}

struct ExpressionLink<'o, 'a> {
    generated: Span,
    original: &'o NativeEmbed<'a>,
}

/// One derived checker document, bound to all original native owners.
///
/// ```compile_fail
/// use vize_l4::targets::ts::vue::VueProjection;
/// use vize_l4::write::EmitDocument;
/// fn forge(document: EmitDocument) { let _ = VueProjection { document }; }
/// ```
pub struct VueProjection<'o, 'a> {
    original: NativeSfc<'o, 'a>,
    kind: SourceKind,
    script: Option<Span>,
    expressions: Vec<ExpressionLink<'o, 'a>>,
    document: EmitDocument,
}

impl<'o, 'a> VueProjection<'o, 'a> {
    #[must_use]
    pub fn original(&self) -> &NativeSfc<'o, 'a> {
        &self.original
    }

    #[must_use]
    pub fn file(&self) -> &'o FileArtifact<'a> {
        self.original.file().file()
    }

    #[must_use]
    pub const fn source_kind(&self) -> SourceKind {
        self.kind
    }

    #[must_use]
    pub fn document(&self) -> &EmitDocument {
        &self.document
    }

    /// Map a complete diagnostic range. Entity interiors conservatively cover
    /// their entire original spelling; this mapping is not an edit selector.
    pub fn map_span(&self, generated: Span) -> Result<Span, MappingError> {
        if !self.document.is_recording() {
            return Err(MappingError::Unrecorded);
        }
        if generated.start > generated.end
            || self
                .document
                .as_str()
                .get(generated.start as usize..generated.end as usize)
                .is_none()
        {
            return Err(MappingError::InvalidRange);
        }
        if let Some(script) = self.script {
            let length = script.end - script.start;
            if generated.start <= length {
                if generated.end > length {
                    return Err(MappingError::CrossesBoundary);
                }
                return Ok(Span::new(
                    script.start + generated.start,
                    script.start + generated.end,
                ));
            }
        }
        for link in &self.expressions {
            if generated.start >= link.generated.start && generated.start <= link.generated.end {
                if generated.end > link.generated.end {
                    return Err(MappingError::CrossesBoundary);
                }
                return link
                    .original
                    .syntax
                    .source()
                    .authored_covering_span(Span::new(
                        generated.start - link.generated.start,
                        generated.end - link.generated.start,
                    ))
                    .map_err(|_| MappingError::InvalidRange);
            }
            if generated.start < link.generated.start && generated.end > link.generated.start {
                return Err(MappingError::CrossesBoundary);
            }
        }
        Err(MappingError::GeneratedOnly)
    }

    /// Actual compiler global UTF-16 offsets, rejecting mid-surrogate endpoints.
    pub fn map_utf16(&self, start: u32, length: u32) -> Result<Span, MappingError> {
        let end = start
            .checked_add(length)
            .ok_or(MappingError::InvalidUtf16Boundary)?;
        let byte = |at| {
            utf16_offset(self.document.as_str(), at)
                .and_then(|offset| u32::try_from(offset).ok())
                .ok_or(MappingError::InvalidUtf16Boundary)
        };
        self.map_span(Span::new(byte(start)?, byte(end)?))
    }
}

/// Type projection for complete native Vue 3, setup-only JS/TS and interpolations.
pub fn project_vue<'o, 'a>(
    original: NativeSfc<'o, 'a>,
) -> Result<VueProjection<'o, 'a>, VueProjectionError> {
    project::<Recorded>(original)
}

/// Byte-identical emission with mapping explicitly unavailable.
pub fn project_vue_no_links<'o, 'a>(
    original: NativeSfc<'o, 'a>,
) -> Result<VueProjection<'o, 'a>, VueProjectionError> {
    project::<NoLinks>(original)
}

fn project<'o, 'a, L: LinkSink>(
    original: NativeSfc<'o, 'a>,
) -> Result<VueProjection<'o, 'a>, VueProjectionError> {
    let observation = original.observation();
    let options = observation.descriptor().options();
    if options.version != VueVersion::V3 || options.dialect != VueDialect::Vue {
        return Err(VueProjectionError::UnsupportedDialect);
    }
    let descriptor = observation
        .descriptor()
        .admitted()
        .map_err(|_| VueProjectionError::Custody)?;
    if descriptor.styles().len() != 0 {
        return Err(VueProjectionError::UnsupportedStyles);
    }
    if original.file().ordinary().is_some() || observation.scripts().len() > 1 {
        return Err(VueProjectionError::UnsupportedScripts);
    }
    let file = original.file().file();
    let source = file.artifact().source();
    u32::try_from(source.len()).map_err(|_| VueProjectionError::SourceTooLarge)?;
    let setup = original.file().setup();
    let kind = if setup.is_some_and(|unit| unit.profile().typescript) {
        SourceKind::TypeScript
    } else {
        SourceKind::JavaScript
    };
    let mut writer = Writer::<L>::with_capacity(source.len());
    let script = observation
        .scripts()
        .first()
        .map(|script| script.block().span());
    if let Some(script) = observation.scripts().first() {
        let syntax = script.syntax().ok_or(VueProjectionError::Custody)?;
        let admitted = syntax
            .admitted_program()
            .ok_or(VueProjectionError::Custody)?;
        if !admitted.source_type().is_module() || admitted.source_type().is_jsx() {
            return Err(VueProjectionError::UnsupportedScripts);
        }
        writer.push_linked(script.block().source(), script.block().span());
    }
    writer.push("\n;\nexport {};\n");
    let embeds = observation
        .template()
        .map_or(&[][..], |template| template.embeds());
    let mut expressions = Vec::with_capacity(if L::RECORDING { embeds.len() } else { 0 });
    let mut consumed = 0;
    let mut primitive_setup = false;
    let mut result = Ok(());
    file.artifact()
        .visit_nodes(&mut |id, node| {
            if result.is_err() {
                return;
            }
            result = (|| {
                let interpolation = match node {
                    NodeRef::Op(Op::Text(_) | Op::Comment(_)) => return Ok(()),
                    // Neutral HTML membership is not Vue's versioned tag role.
                    // The pinned Vue 3 reference treats this original tag as a
                    // component; never omit its unfinished component typing.
                    NodeRef::Op(Op::Element(element)) if element.tag != "search" => return Ok(()),
                    NodeRef::Op(Op::Interpolation(interpolation)) => interpolation,
                    _ => return Err(VueProjectionError::UnsupportedTemplate),
                };
                let ExprRef::Js(expression) = interpolation.expression else {
                    return Err(VueProjectionError::Custody);
                };
                let embed = embeds.get(consumed).ok_or(VueProjectionError::Custody)?;
                let resolution = file.expression(id).ok_or(VueProjectionError::Custody)?;
                let table = resolution.table().ok_or(VueProjectionError::Custody)?;
                let actual = embed
                    .syntax
                    .expression()
                    .ok_or(VueProjectionError::Custody)?;
                if embed.node != Some(id)
                    || !core::ptr::eq(expression.ast, actual)
                    || !core::ptr::eq(expression.ast, table.expression().ast)
                    || !core::ptr::eq(expression.source, table.expression().source)
                    || expression.span != table.expression().span
                    || !core::ptr::eq(expression.source, embed.syntax.source().text())
                    || expression.span != embed.syntax.source().span()
                    || resolution.scope()
                        != setup
                            .map(|receipt| receipt.scope())
                            .or_else(|| file.scopes().first().map(|scope| scope.id))
                {
                    return Err(VueProjectionError::Custody);
                }
                for occurrence in table.occurrences() {
                    if occurrence.usage != Usage::Read {
                        return Err(VueProjectionError::UnsupportedTemplateBindings);
                    }
                    let binding = resolution
                        .binding(occurrence.binding)
                        .ok_or(VueProjectionError::Custody)?;
                    if !resolution.accepts(binding) || original.file().exposure(binding).is_none() {
                        return Err(VueProjectionError::Custody);
                    }
                }
                // Membership alone grants no Vue ref-unwrapping semantics.
                // Reuse the original sole-walk whole-unit primitive proof;
                // declarations are never reconstructed or scanned here.
                if !table.occurrences().is_empty() && !primitive_setup {
                    let script = descriptor.setup().ok_or(VueProjectionError::Custody)?;
                    let program = observation
                        .scripts()
                        .first()
                        .and_then(|script| script.syntax())
                        .and_then(|syntax| syntax.admitted_program())
                        .ok_or(VueProjectionError::Custody)?;
                    // A primitive initializer does not certify an effective JS
                    // JSDoc type. Vue Ref unwrapping remains unfinished here.
                    if kind == SourceKind::JavaScript && program.has_jsdoc_comments() {
                        return Err(VueProjectionError::UnsupportedTemplateBindings);
                    }
                    VueSetup::checked(file, script, program)
                        .map_err(|_| VueProjectionError::UnsupportedTemplateBindings)?;
                    primitive_setup = true;
                }
                writer.push("void (\n");
                let start =
                    u32::try_from(writer.len()).map_err(|_| VueProjectionError::SourceTooLarge)?;
                let prepared = embed.syntax.source();
                if let Some(map) = prepared.decode_map() {
                    for segment in map.segments() {
                        let decoded = segment.decoded();
                        writer.push_linked(
                            prepared
                                .text()
                                .get(decoded.start as usize..decoded.end as usize)
                                .ok_or(VueProjectionError::Custody)?,
                            segment.authored(),
                        );
                    }
                } else {
                    writer.push_linked(prepared.text(), prepared.span());
                }
                let end =
                    u32::try_from(writer.len()).map_err(|_| VueProjectionError::SourceTooLarge)?;
                writer.push("\n);\n");
                if L::RECORDING {
                    expressions.push(ExpressionLink {
                        generated: Span::new(start, end),
                        original: embed,
                    });
                }
                consumed += 1;
                Ok(())
            })();
        })
        .map_err(|_| VueProjectionError::Custody)?;
    result?;
    if consumed != embeds.len() {
        return Err(VueProjectionError::Custody);
    }
    u32::try_from(writer.len()).map_err(|_| VueProjectionError::SourceTooLarge)?;
    Ok(VueProjection {
        original,
        kind,
        script,
        expressions,
        document: writer.finish().into_document(),
    })
}

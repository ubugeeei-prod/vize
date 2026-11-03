//! Write checked retained-expression facts directly, without parsing/searching.
//!
//! L2 owns binding identities and every supported runtime identifier use.
//! Framework access spelling is supplied separately; generic emission does
//! not decide whether a binding is a Vue ref or a render-context property.
//! All source, spelling, projection and length checks precede writer mutation.

use alloc::vec::Vec;
use vize_l0::Span;

use vize_l2::resolution::Occurrence;
pub use vize_l2::resolution::ResolutionTable;

use crate::runtime::Helper;
use crate::write::{LinkSink, Writer};

mod resolved;
pub mod vue;
pub use resolved::{ResolutionSetError, ResolvedExpressions};
pub use vue::AccessStyle;

/// The spelling chosen for one resolved identifier use.
#[derive(Debug, Clone, Copy)]
pub enum AccessSpelling<'a> {
    Verbatim,
    Rewrite {
        prefix: &'a str,
        /// `None` retains the exact authored spelling, including JS escapes.
        replacement: Option<&'a str>,
        suffix: &'a str,
        helper: Option<Helper>,
    },
}

/// A framework's checked accessor decision, keyed by L2 binding identity.
pub trait AccessProvider {
    fn spelling(&self, occurrence: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessError {
    MissingBinding,
    UnsupportedUsage,
    MissingHelper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitError {
    /// Authored bytes when projection is available, else the whole embed.
    pub span: Span,
    pub kind: EmitErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitErrorKind {
    MissingResolution,
    SourceMismatch,
    InvalidProjection,
    OutputTooLarge,
    Access(AccessError),
}

struct Prepared<'a> {
    gap: &'a str,
    gap_authored: Span,
    original: &'a str,
    authored: Span,
    name: &'a str,
    shorthand_key: Option<&'a str>,
    spelling: AccessSpelling<'a>,
}

/// Write one completely resolved supported expression.
///
/// Unchanged decoded bytes keep their authored links. Expanded shorthand keeps
/// its property key; `__proto__` uses a computed key to remain an own property.
/// A rewritten value gets one named link covering
/// the complete accessor, including a renamed prop or ref suffix. Failure
/// leaves text, links, indentation and used helpers unchanged.
pub fn write_expression<L: LinkSink>(
    writer: &mut Writer<L>,
    authored_file: &str,
    table: &ResolutionTable<'_>,
    access: &impl AccessProvider,
) -> Result<(), EmitError> {
    let expression = table.expression();
    let source = expression.source;
    let fail = |kind| EmitError {
        span: expression.span,
        kind,
    };
    if !expression.matches_authored_source(authored_file) {
        return Err(fail(EmitErrorKind::SourceMismatch));
    }
    let project = |span: Span| {
        let authored = expression.authored_span(span)?;
        authored_file.get(authored.start as usize..authored.end as usize)?;
        Some(authored)
    };
    let mut prepared = Vec::with_capacity(table.occurrences().len());
    let mut cursor = 0;
    let mut generated = writer.len();
    for occurrence in table.occurrences() {
        let span = occurrence.span;
        let gap_span = Span::new(cursor, span.start);
        let gap = source
            .get(cursor as usize..span.start as usize)
            .ok_or_else(|| fail(EmitErrorKind::InvalidProjection))?;
        let original = source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| fail(EmitErrorKind::InvalidProjection))?;
        let gap_authored =
            project(gap_span).ok_or_else(|| fail(EmitErrorKind::InvalidProjection))?;
        let authored = project(span).ok_or_else(|| fail(EmitErrorKind::InvalidProjection))?;
        let spelling = access.spelling(occurrence).map_err(|error| EmitError {
            span: authored,
            kind: EmitErrorKind::Access(error),
        })?;
        let shorthand_key = occurrence
            .shorthand
            .then_some(if occurrence.name == "__proto__" {
                // A colon-form __proto__ key is a prototype setter, including an
                // escaped spelling. Computed keys preserve shorthand data semantics.
                "[\"__proto__\"]"
            } else {
                original
            });
        let added = match spelling {
            AccessSpelling::Verbatim => original.len(),
            AccessSpelling::Rewrite {
                prefix,
                replacement,
                suffix,
                ..
            } => {
                let key = if let Some(key) = shorthand_key {
                    key.len().checked_add(2)
                } else {
                    Some(0)
                };
                key.and_then(|length| length.checked_add(prefix.len()))
                    .and_then(|length| length.checked_add(replacement.unwrap_or(original).len()))
                    .and_then(|length| length.checked_add(suffix.len()))
                    .ok_or_else(|| fail(EmitErrorKind::OutputTooLarge))?
            }
        };
        generated = generated
            .checked_add(gap.len())
            .and_then(|length| length.checked_add(added))
            .ok_or_else(|| fail(EmitErrorKind::OutputTooLarge))?;
        prepared.push(Prepared {
            gap,
            gap_authored,
            original,
            authored,
            name: occurrence.name,
            shorthand_key,
            spelling,
        });
        cursor = span.end;
    }
    let source_end =
        u32::try_from(source.len()).map_err(|_| fail(EmitErrorKind::OutputTooLarge))?;
    let tail = source
        .get(cursor as usize..)
        .ok_or_else(|| fail(EmitErrorKind::InvalidProjection))?;
    let tail_authored = project(Span::new(cursor, source_end))
        .ok_or_else(|| fail(EmitErrorKind::InvalidProjection))?;
    generated = generated
        .checked_add(tail.len())
        .ok_or_else(|| fail(EmitErrorKind::OutputTooLarge))?;
    u32::try_from(generated).map_err(|_| fail(EmitErrorKind::OutputTooLarge))?;

    for part in prepared {
        if !part.gap.is_empty() {
            writer.push_linked(part.gap, part.gap_authored);
        }
        match part.spelling {
            AccessSpelling::Verbatim => writer.push_named(part.original, part.authored, part.name),
            AccessSpelling::Rewrite {
                prefix,
                replacement,
                suffix,
                helper,
            } => {
                if let Some(key) = part.shorthand_key {
                    writer.push_named(key, part.authored, part.name);
                    writer.push(": ");
                }
                writer.push_named_parts(
                    [prefix, replacement.unwrap_or(part.original), suffix],
                    part.authored,
                    part.name,
                );
                if let Some(helper) = helper {
                    writer.use_helper(helper);
                }
            }
        }
    }
    if !tail.is_empty() {
        writer.push_linked(tail, tail_authored);
    }
    Ok(())
}

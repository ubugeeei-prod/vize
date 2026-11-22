//! Inspect the intact part of a TypeScript script while its suffix is being
//! edited. Each published cursor result is tied to the current document.
use super::{SnapshotRefusal, SourceSnapshot};
use crate::{
    document::DocumentStore,
    utils::{offset_to_position, position_to_offset},
};
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Hover, HoverContents, LanguageString, MarkedString, Position, Range};
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, cstr};
use vize_l1::{
    container::vue::{DescriptorObservation, ScriptRole},
    embed::{Lang, authored_program::AuthoredProgram},
};
use vize_l2::query::script::assignments::{
    AssignmentError, LiteralAssignments, ScriptRefusal, Verdict,
    prefix::{LiteralPrefixIndex, PrefixRefusal, UnchangedLiterals},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveredScriptError {
    Host(SnapshotRefusal),
    Document,
    Source,
    Descriptor,
    Role,
    Syntax(ScriptRefusal),
    Evidence(AssignmentError),
    Prefix(PrefixRefusal),
    Position,
}

pub struct PreviousScript<'s> {
    snapshot: &'s SourceSnapshot,
    role: ScriptRole,
    index: LiteralPrefixIndex<'s>,
}
pub struct RecoveredScript<'idx, 's> {
    snapshot: &'s SourceSnapshot,
    local: UnchangedLiterals<'idx, 's>,
}

#[derive(Debug, PartialEq)]
pub struct RecoveredCursor {
    pub version: i32,
    /// Byte-identical complete declaration prefix, expressed in current LSP coordinates.
    pub covered: Range,
    pub bindings: usize,
    pub hover: Option<Hover>,
    /// The local initializer judgment at the requested binding.
    pub assignment: Option<Verdict>,
}

fn selected<'s>(
    snap: &'s SourceSnapshot,
    owner: &DescriptorObservation<'_>,
    role: ScriptRole,
) -> Result<SourceBlock<'s>, RecoveredScriptError> {
    if snap.language_id() != "vue" {
        return Err(RecoveredScriptError::Document);
    }
    if !core::ptr::eq(snap.source(), owner.source()) {
        return Err(RecoveredScriptError::Source);
    }
    let desc = owner
        .admitted()
        .map_err(|_| RecoveredScriptError::Descriptor)?;
    let script = match role {
        ScriptRole::Ordinary => desc.ordinary(),
        ScriptRole::Setup => desc.setup(),
    }
    .filter(|s| s.lang() == Lang::Ts)
    .ok_or(RecoveredScriptError::Role)?;
    let span = script.block().span();
    let text = snap
        .source()
        .get(span.start as usize..span.end as usize)
        .ok_or(RecoveredScriptError::Source)?;
    SourceRoot::new(snap.source())
        .and_then(|root| root.block(text, span.start))
        .map_err(|_| RecoveredScriptError::Source)
}

impl SourceSnapshot {
    /// Parse and certify a valid original script once. The retained index
    /// shares the owned snapshot text after the temporary AST arena is dropped.
    pub fn retain_literal_prefix<'s>(
        &'s self,
        docs: &DocumentStore,
        owner: &DescriptorObservation<'_>,
        role: ScriptRole,
    ) -> Result<PreviousScript<'s>, RecoveredScriptError> {
        self.check_current(docs)
            .map_err(RecoveredScriptError::Host)?;
        let block = selected(self, owner, role)?;
        let arena = Allocator::new();
        let syntax = AuthoredProgram::parse(&arena, block, SourceType::ts());
        let model = LiteralAssignments::new(&syntax).map_err(RecoveredScriptError::Syntax)?;
        let rows = model.prove();
        let checked = model.check(&rows).map_err(RecoveredScriptError::Evidence)?;
        let index =
            LiteralPrefixIndex::collect(block, &checked).map_err(RecoveredScriptError::Prefix)?;
        Ok(PreviousScript {
            snapshot: self,
            role,
            index,
        })
    }
}
impl PreviousScript<'_> {
    /// Compare the previous evidence with the actual newly selected block.
    /// The suffix need not parse. All returned spans belong to `current`.
    pub fn recover<'idx, 's>(
        &'idx self,
        docs: &DocumentStore,
        current: &'s SourceSnapshot,
        owner: &DescriptorObservation<'_>,
    ) -> Result<RecoveredScript<'idx, 's>, RecoveredScriptError> {
        current
            .check_current(docs)
            .map_err(RecoveredScriptError::Host)?;
        if self.snapshot.uri() != current.uri() || self.snapshot.revision() > current.revision() {
            return Err(RecoveredScriptError::Document);
        }
        let block = selected(current, owner, self.role)?;
        let local = self
            .index
            .revalidate(block, SourceType::ts())
            .map_err(RecoveredScriptError::Prefix)?;
        Ok(RecoveredScript {
            snapshot: current,
            local,
        })
    }
}
impl RecoveredScript<'_, '_> {
    /// Use the exact live rope for the cursor and coverage boundaries.
    pub fn publish(
        &self,
        docs: &DocumentStore,
        position: Position,
    ) -> Result<RecoveredCursor, RecoveredScriptError> {
        self.snapshot
            .with_current(docs, |doc| {
                let text = doc.position_text();
                let offset = position_to_offset(text, position)
                    .and_then(|x| u32::try_from(x).ok())
                    .ok_or(RecoveredScriptError::Position)?;
                let range = |s: Span| -> Result<Range, RecoveredScriptError> {
                    Ok(Range::new(
                        offset_to_position(text, s.start as usize)
                            .ok_or(RecoveredScriptError::Position)?,
                        offset_to_position(text, s.end as usize)
                            .ok_or(RecoveredScriptError::Position)?,
                    ))
                };
                let row = self.local.binding_at(offset);
                let hover = if let Some(r) = row {
                    let name = r.binding.slice(self.snapshot.source());
                    Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::LanguageString(
                            LanguageString {
                                language: "typescript".into(),
                                value: cstr!("{name}: {}", r.expected.keyword()).into(),
                            },
                        )),
                        range: Some(range(r.binding)?),
                    })
                } else {
                    None
                };
                Ok(RecoveredCursor {
                    version: doc.version,
                    covered: range(self.local.covered())?,
                    bindings: self.local.assignments().len(),
                    hover,
                    assignment: row.map(|r| r.verdict),
                })
            })
            .map_err(RecoveredScriptError::Host)?
    }
}
#[cfg(test)]
mod corpus;
#[cfg(all(test, feature = "micro-profiling"))]
mod profile;
#[cfg(test)]
mod tests;

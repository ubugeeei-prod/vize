//! Establish an initial checked prefix when an editor opens a damaged file.
use super::{LiteralPrefixIndex, PrefixRefusal, SourceBlock};
use crate::query::script::assignments::{AssignmentError, LiteralAssignments, ScriptRefusal};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::authored_program::AuthoredProgram;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixOpenError {
    Syntax(ScriptRefusal),
    Evidence(AssignmentError),
    Prefix(PrefixRefusal),
}

enum Attempt<'s> {
    Checked(LiteralPrefixIndex<'s>),
    Damaged { first_label: usize },
}

fn examine(block: SourceBlock<'_>, grammar: SourceType) -> Result<Attempt<'_>, PrefixOpenError> {
    let arena = Allocator::new();
    let syntax = AuthoredProgram::parse(&arena, block, grammar);
    let assignments = match LiteralAssignments::new(&syntax) {
        Ok(model) => model,
        Err(ScriptRefusal::Parse(_)) => {
            let first_label = syntax
                .parsed()
                .diagnostics
                .iter()
                .flat_map(|diagnostic| diagnostic.labels.iter())
                .map(|label| label.offset() as usize)
                .min()
                .unwrap_or(0);
            return Ok(Attempt::Damaged { first_label });
        }
        Err(reason) => return Err(PrefixOpenError::Syntax(reason)),
    };
    let rows = assignments.prove();
    let checked = assignments
        .check(&rows)
        .map_err(PrefixOpenError::Evidence)?;
    LiteralPrefixIndex::collect(block, &checked)
        .map(Attempt::Checked)
        .map_err(PrefixOpenError::Prefix)
}

impl<'s> LiteralPrefixIndex<'s> {
    /// Open an original block without edit history. A successful full parse is
    /// indexed once. On a syntax error only a separately parsed and verified
    /// prefix can contribute rows. Call `revalidate` on the complete current
    /// block to inspect those rows and account for directives in the suffix.
    pub fn open(block: SourceBlock<'s>, grammar: SourceType) -> Result<Self, PrefixOpenError> {
        let text = block.source();
        let mut limit = match examine(block, grammar)? {
            Attempt::Checked(index) => return Ok(index),
            Attempt::Damaged { first_label } => first_label.min(text.len().saturating_sub(1)),
        };
        let root = SourceRoot::new(block.root_source())
            .map_err(|_| PrefixOpenError::Prefix(PrefixRefusal::InvalidBoundary))?;
        // A raw semicolon proposes a cut. Only actual top-level statements
        // sealed by the separate parser and verifier will enter the index.
        while let Some(separator) = text
            .get(..text.floor_char_boundary(limit))
            .and_then(|s| s.rfind(';'))
        {
            let end = separator + 1;
            let prefix = text
                .get(..end)
                .and_then(|slice| root.block(slice, block.start()).ok())
                .ok_or(PrefixOpenError::Prefix(PrefixRefusal::InvalidBoundary))?;
            match examine(prefix, grammar)? {
                Attempt::Checked(index) => return Ok(index),
                Attempt::Damaged { first_label } => {
                    // A semicolon inside a block, template, or comment can
                    // leave another incomplete program. Geometric backoff
                    // bounds all retry input bytes by twice the first prefix.
                    limit = super::frame::retry_limit(end, first_label);
                }
            }
        }
        Ok(Self {
            original: block,
            grammar,
            entries: alloc::vec::Vec::new(),
        })
    }
}

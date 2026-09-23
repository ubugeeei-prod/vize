//! Source frames: how a range measured in block coordinates becomes a
//! file-absolute range before any output layer renders it.
//!
//! Output formatters derive line and column from **file-absolute** byte
//! offsets. A range measured in another frame renders on the wrong line —
//! FP-1 (`davinci-road/plan/ledger-fp.md`): `type/require-typed-emits` and
//! `type/require-typed-props` reported script-analysis offsets shifted by the
//! *template's* start, so squiggles landed on innocent template text.
//!
//! [`ScriptFrame`] is the one conversion from croquis' script-analysis frame
//! to the file, validated through the S0 frame ([`SourceRoot`] /
//! [`SourceBlock`]) that the Davinci stages key their spans on: a range that
//! is not inside the block it claims is refused, never guessed.

use vize_atelier_sfc::SfcDescriptor;
use vize_s0::{SourceBlock, SourceRoot, Span};

use crate::diagnostic::LintDiagnostic;

/// The coordinates croquis' lint analysis measures script offsets in.
///
/// `analyze_sfc_descriptor` with the lint demand analyzes the
/// `<script>` content, a `\n`, then the `<script setup>` content when both
/// blocks exist (setup offsets are shifted by the plain content's length plus
/// one); otherwise it analyzes the one block alone. The frame mirrors that
/// layout block by block, so each piece maps back to its own block even when
/// `<script setup>` precedes `<script>` in the file.
#[derive(Debug, Clone, Copy)]
pub struct ScriptFrame<'a> {
    plain: Option<SourceBlock<'a>>,
    setup: Option<SourceBlock<'a>>,
}

impl<'a> ScriptFrame<'a> {
    /// The script frame of `descriptor`'s lint analysis, or `None` when the
    /// file has no script block or a block's content is not the exact text
    /// at its recorded range (then no offset inside it can be located).
    #[must_use]
    pub fn for_lint(descriptor: &'a SfcDescriptor<'a>) -> Option<Self> {
        let root = SourceRoot::new(descriptor.source.as_ref()).ok()?;
        let block = |script: Option<&'a vize_atelier_sfc::SfcScriptBlock<'a>>| {
            script.map(|script| {
                let text = descriptor.source.get(script.loc.start..script.loc.end)?;
                if text != script.content.as_ref() {
                    return None;
                }
                root.block(text, u32::try_from(script.loc.start).ok()?).ok()
            })
        };
        let plain = block(descriptor.script.as_ref());
        let setup = block(descriptor.script_setup.as_ref());
        // A block that exists but cannot be framed poisons the whole frame:
        // its length decides where the other block's offsets start.
        if matches!(plain, Some(None)) || matches!(setup, Some(None)) {
            return None;
        }
        let frame = Self {
            plain: plain.flatten(),
            setup: setup.flatten(),
        };
        (frame.plain.is_some() || frame.setup.is_some()).then_some(frame)
    }

    /// The file-absolute span of the analysis range `start..end`, or `None`
    /// when it straddles the two blocks or leaves them.
    #[must_use]
    pub fn to_file(self, start: u32, end: u32) -> Option<Span> {
        if start > end {
            return None;
        }
        let (block, base) = match (self.plain, self.setup) {
            (Some(plain), Some(setup)) => {
                let plain_len = plain.end() - plain.start();
                if end <= plain_len {
                    (plain, 0)
                } else if start > plain_len {
                    (setup, plain_len + 1)
                } else {
                    return None;
                }
            }
            (Some(block), None) | (None, Some(block)) => (block, 0),
            (None, None) => return None,
        };
        let span = Span::new(block.start() + (start - base), block.start() + (end - base));
        block.contains_block_span(span).then_some(span)
    }

    /// Move every range `diagnostic` carries — primary, labels, fix edits —
    /// from this frame into the file. Returns `None`, leaving the diagnostic
    /// untouched, when any range cannot be located.
    #[must_use]
    pub fn reframe(self, diagnostic: &LintDiagnostic) -> Option<LintDiagnostic> {
        let mut moved = diagnostic.clone();
        let primary = self.to_file(diagnostic.start, diagnostic.end)?;
        (moved.start, moved.end) = (primary.start, primary.end);
        for label in &mut moved.labels {
            let span = self.to_file(label.start, label.end)?;
            (label.start, label.end) = (span.start, span.end);
        }
        if let Some(fix) = moved.fix.as_mut() {
            for edit in &mut fix.edits {
                let span = self.to_file(edit.start, edit.end)?;
                (edit.start, edit.end) = (span.start, span.end);
            }
        }
        Some(moved)
    }
}

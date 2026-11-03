//! Edits move the text and the native diagnostics anchored to it.

use std::sync::{Arc, OnceLock};

use tower_lsp::lsp_types::TextDocumentContentChangeEvent;
#[cfg(feature = "native")]
use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Range, Url};

use super::{Document, DocumentText, next_revision};
use crate::utils::position_to_offset;
#[cfg(feature = "native")]
use crate::{ide::diagnostics::NativeFamily, utils::offset_to_position};

/// The latest native family of an earlier revision, anchored to byte offsets
/// of the current text. An edit overlapping an anchor drops that diagnostic.
#[cfg(feature = "native")]
#[derive(Debug)]
pub(super) struct CarriedNative {
    anchored: Vec<(usize, usize, Diagnostic)>,
    /// Holds only for the text the family was computed on: after an edit the
    /// script parser alone describes the current syntax (#11626).
    owns_script_syntax: bool,
}

#[cfg(feature = "native")]
impl CarriedNative {
    fn replace(&mut self, start: usize, end: usize, inserted: usize) {
        if std::mem::take(&mut self.owns_script_syntax) {
            // TypeScript's syntax errors (TS1xxx) described the old text.
            self.anchored.retain(|(_, _, diagnostic)| {
                !matches!(diagnostic.code, Some(NumberOrString::Number(1000..=1999)))
            });
        }
        self.anchored.retain_mut(|(from, to, _)| {
            if *to <= start {
                return true;
            }
            if *from < end {
                return false;
            }
            *from = *from - end + start + inserted;
            *to = *to - end + start + inserted;
            true
        });
    }
}

/// The bytes a whole-text replacement changed: `start..end` of the old text
/// became `inserted` bytes.
#[cfg(feature = "native")]
fn replaced_window(old: &str, new: &str) -> (usize, usize, usize) {
    let (old, new) = (old.as_bytes(), new.as_bytes());
    let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let (old_rest, new_rest) = (
        old.get(prefix..).unwrap_or_default(),
        new.get(prefix..).unwrap_or_default(),
    );
    let suffix = old_rest
        .iter()
        .rev()
        .zip(new_rest.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    (prefix, old.len() - suffix, new_rest.len() - suffix)
}

impl Document {
    /// Apply an incremental change to the document.
    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent, new_version: i32) {
        self.version = new_version;
        self.revision = next_revision();
        self.petite_vue_detected = OnceLock::new();
        self.structural_snapshot = OnceLock::new();

        let Some(range) = change.range else {
            #[cfg(feature = "native")]
            let window = self
                .native
                .is_some()
                .then(|| replaced_window(self.flat_text(), &change.text));
            self.content = DocumentText::new(&change.text);
            self.text_snapshot = OnceLock::from(Arc::new(change.text.clone()));
            #[cfg(feature = "native")]
            if let (Some(native), Some((start, end, inserted))) = (&mut self.native, window) {
                native.replace(start, end, inserted);
            }
            return;
        };
        self.text_snapshot = OnceLock::new();
        let (Some(start), Some(end)) = (
            position_to_offset(&self.content, range.start),
            position_to_offset(&self.content, range.end),
        ) else {
            return;
        };
        let (Ok(start_char), Ok(end_char)) = (
            self.content.try_byte_to_char(start),
            self.content.try_byte_to_char(end),
        ) else {
            return;
        };
        if start_char > end_char {
            return;
        }
        self.content.replace(start_char..end_char, &change.text);
        #[cfg(feature = "native")]
        if let Some(native) = &mut self.native {
            native.replace(start, end, change.text.len());
        }
    }
}

#[cfg(feature = "native")]
impl super::DocumentStore {
    /// Anchor a complete result's native family to the text it describes.
    pub(crate) fn carry_native(&self, uri: &Url, version: i32, family: NativeFamily) -> bool {
        let Some(mut document) = self.documents.get_mut(uri) else {
            return false;
        };
        if document.version != version {
            return false;
        }
        let count = family.diagnostics.len();
        let anchored: Vec<_> = family
            .diagnostics
            .into_iter()
            .filter(|diagnostic| {
                // Related locations would not move with the text.
                diagnostic
                    .related_information
                    .as_ref()
                    .is_none_or(|related| {
                        related.iter().all(|related| &related.location.uri != uri)
                    })
            })
            .filter_map(|diagnostic| {
                let start = position_to_offset(&document.content, diagnostic.range.start)?;
                let end = position_to_offset(&document.content, diagnostic.range.end)?;
                Some((start, end, diagnostic))
            })
            .collect();
        if anchored.len() != count {
            tracing::warn!(
                "{uri}: {} native diagnostics cannot move with edits",
                count - anchored.len()
            );
        }
        document.native = Some(CarriedNative {
            anchored,
            owns_script_syntax: family.owns_script_syntax,
        });
        true
    }

    /// The carried native family at `version`'s text.
    pub(crate) fn carried_native(&self, uri: &Url, version: i32) -> Option<NativeFamily> {
        let document = self.documents.get(uri)?;
        let native = document
            .native
            .as_ref()
            .filter(|_| document.version == version)?;
        let diagnostics = native
            .anchored
            .iter()
            .filter_map(|(start, end, diagnostic)| {
                let range = Range {
                    start: offset_to_position(&document.content, *start)?,
                    end: offset_to_position(&document.content, *end)?,
                };
                Some(Diagnostic {
                    range,
                    ..diagnostic.clone()
                })
            })
            .collect();
        Some(NativeFamily {
            diagnostics,
            owns_script_syntax: native.owns_script_syntax,
        })
    }
}

#[cfg(all(test, feature = "native"))]
mod tests;

use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, SmallVec, compact_format};
use lsp_types::{
    Position, Range, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem, Uri,
    VersionedTextDocumentIdentifier,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Incremental virtual-document edit expressed in LSP coordinates.
///
/// # Examples
///
/// ```
/// use lsp_types::{Position, Range};
/// use corsa_lsp::VirtualChange;
///
/// let change = VirtualChange::splice(
///     Range::new(Position::new(0, 0), Position::new(0, 5)),
///     "hello",
/// );
/// assert_eq!(change.text, "hello");
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualChange {
    /// UTF-16 range being replaced, or `None` for a full-document replacement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    /// Optional range length field accepted by LSP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_length: Option<u32>,
    /// Replacement text.
    pub text: CompactString,
}

/// Virtual text document mirrored over LSP notifications.
///
/// # Examples
///
/// ```
/// use corsa_lsp::{VirtualChange, VirtualDocument};
///
/// let mut document = VirtualDocument::untitled("/virtual/demo.ts", "typescript", "const n = 1;")?;
/// document.apply_changes(&[VirtualChange::replace("const n = 2;")])?;
/// assert_eq!(document.text, "const n = 2;");
/// assert_eq!(document.version, 2);
/// # Ok::<(), corsa_lsp::CorsaError>(())
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualDocument {
    /// Canonical document URI used in LSP payloads.
    pub uri: Uri,
    /// Language identifier such as `"typescript"` or `"javascript"`.
    pub language_id: CompactString,
    /// Monotonic document version used in `didChange`.
    pub version: i32,
    /// Full current text of the document.
    pub text: CompactString,
}

impl VirtualChange {
    /// Replaces the entire document contents.
    pub fn replace(text: impl Into<CompactString>) -> Self {
        Self {
            range: None,
            range_length: None,
            text: text.into(),
        }
    }

    /// Replaces a UTF-16 range with new text.
    ///
    /// Offsets follow standard LSP UTF-16 coordinate rules.
    pub fn splice(range: Range, text: impl Into<CompactString>) -> Self {
        Self {
            range: Some(range),
            range_length: None,
            text: text.into(),
        }
    }

    /// Converts the change into an LSP content change event.
    pub fn into_event(self) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangeEvent {
            range: self.range,
            range_length: self.range_length,
            text: self.text.into(),
        }
    }
}

impl From<TextDocumentContentChangeEvent> for VirtualChange {
    fn from(value: TextDocumentContentChangeEvent) -> Self {
        Self {
            range: value.range,
            range_length: value.range_length,
            text: value.text.into(),
        }
    }
}

impl VirtualDocument {
    /// Creates a virtual document from an arbitrary URI.
    ///
    /// New documents start at version `1`, matching LSP expectations for
    /// freshly opened in-memory documents.
    pub fn new(
        uri: Uri,
        language_id: impl Into<CompactString>,
        text: impl Into<CompactString>,
    ) -> Self {
        Self {
            uri,
            language_id: language_id.into(),
            version: 1,
            text: text.into(),
        }
    }

    /// Converts an LSP `TextDocumentItem` into a virtual document.
    pub fn from_item(item: TextDocumentItem) -> Self {
        Self {
            uri: item.uri,
            language_id: item.language_id.into(),
            version: item.version,
            text: item.text.into(),
        }
    }

    /// Creates an `untitled:` virtual document.
    ///
    /// This is a convenient fit for editor-style scratch buffers.
    pub fn untitled(
        path: impl AsRef<str>,
        language_id: impl Into<CompactString>,
        text: impl Into<CompactString>,
    ) -> Result<Self> {
        Self::parse_uri(
            compact_format(format_args!(
                "untitled:{}",
                normalize_virtual_path(path.as_ref())
            )),
            language_id,
            text,
        )
    }

    /// Creates an in-memory `corsa://` virtual document.
    ///
    /// This scheme is helpful when replicated or synthetic documents should be
    /// clearly distinguishable from user-owned workspace files.
    pub fn in_memory(
        authority: impl AsRef<str>,
        path: impl AsRef<str>,
        language_id: impl Into<CompactString>,
        text: impl Into<CompactString>,
    ) -> Result<Self> {
        let mut raw = CompactString::from("corsa://");
        raw.push_str(authority.as_ref());
        raw.push_str(normalize_virtual_path(path.as_ref()).as_str());
        Self::parse_uri(raw, language_id, text)
    }

    /// Returns the stable map key used by overlays and replicated state.
    pub fn key(&self) -> CompactString {
        CompactString::from(self.uri.as_str())
    }

    /// Returns the unversioned LSP identifier for this document.
    pub fn identifier(&self) -> TextDocumentIdentifier {
        TextDocumentIdentifier::new(self.uri.clone())
    }

    /// Returns the versioned LSP identifier for this document.
    pub fn versioned_identifier(&self) -> VersionedTextDocumentIdentifier {
        VersionedTextDocumentIdentifier::new(self.uri.clone(), self.version)
    }

    /// Builds a `TextDocumentItem` payload suitable for `didOpen`.
    pub fn text_document_item(&self) -> TextDocumentItem {
        TextDocumentItem::new(
            self.uri.clone(),
            self.language_id.clone().into(),
            self.version,
            self.text.clone().into(),
        )
    }

    /// Applies a batch of changes and returns the LSP payloads that were emitted.
    ///
    /// The payloads are returned in the same order as the input changes, and
    /// the document version is incremented exactly once for the batch.
    pub fn apply_changes(
        &mut self,
        changes: &[VirtualChange],
    ) -> Result<SmallVec<[TextDocumentContentChangeEvent; 4]>> {
        if changes.is_empty() {
            return Ok(SmallVec::new());
        }
        let events = changes
            .iter()
            .cloned()
            .map(VirtualChange::into_event)
            .collect::<SmallVec<[TextDocumentContentChangeEvent; 4]>>();
        for change in changes {
            apply_change(&mut self.text, change)?;
        }
        self.version += 1;
        Ok(events)
    }

    fn parse_uri(
        raw: CompactString,
        language_id: impl Into<CompactString>,
        text: impl Into<CompactString>,
    ) -> Result<Self> {
        let uri = Uri::from_str(&raw).map_err(|err| {
            CorsaError::Protocol(compact_format(format_args!("invalid virtual uri: {err}")))
        })?;
        Ok(Self::new(uri, language_id, text))
    }
}

fn normalize_virtual_path(path: &str) -> CompactString {
    if path.starts_with('/') {
        CompactString::from(path)
    } else {
        compact_format(format_args!("/{path}"))
    }
}

fn apply_change(text: &mut CompactString, change: &VirtualChange) -> Result<()> {
    if let Some(range) = &change.range {
        let start = byte_offset(text, range.start)?;
        let end = byte_offset(text, range.end)?;
        if start > end {
            return Err(CorsaError::Protocol(
                "virtual edit has an inverted range".into(),
            ));
        }
        text.replace_range(start..end, change.text.as_str());
        return Ok(());
    }
    *text = change.text.clone();
    Ok(())
}

fn byte_offset(text: &str, position: Position) -> Result<usize> {
    let mut line = 0_u32;
    let mut column = 0_u32;
    for (index, ch) in text.char_indices() {
        if line == position.line && column == position.character {
            return Ok(index);
        }
        if ch == '\n' {
            if line == position.line {
                return position
                    .character
                    .eq(&column)
                    .then_some(index)
                    .ok_or_else(|| CorsaError::Protocol("virtual edit is out of bounds".into()));
            }
            line += 1;
            column = 0;
            continue;
        }
        if line == position.line {
            column += ch.len_utf16() as u32;
            if column > position.character {
                return Err(CorsaError::Protocol(
                    "virtual edit splits a code point".into(),
                ));
            }
        }
    }
    if line == position.line && column == position.character {
        return Ok(text.len());
    }
    Err(CorsaError::Protocol("virtual edit is out of bounds".into()))
}

//! Decoder for the source-file level fields of Corsa's binary AST payload.
//!
//! [`ApiClient::get_source_file`](crate::ApiClient::get_source_file) returns the
//! encoded AST as opaque bytes. Most of that payload is the node table, which
//! this crate deliberately leaves alone, but the header and the `SourceFile`
//! node's extended data carry everything a tool needs to work with content
//! mapped files: the virtual text the checker actually saw, the original text
//! the user edits, the mapper identity, the span map between them, and the
//! diagnostic directives the mapper asked for.
//!
//! The layout is documented in upstream `tsc/internal/api/encoder/encoder.go`
//! and is versioned by the protocol byte in the header, which
//! [`EncodedSourceFile::decode`] verifies before reading anything else.

use corsa_core::fast::compact_format;
use serde::{Deserialize, Serialize};

use crate::{CorsaError, Result};

use super::{
    DiagnosticDirectivePolicy, MappedDiagnosticDirective, SpanMap, SpanMapFeature, SpanMapKind,
    SpanMapSegment, TextRange, encoded::EncodedPayload,
};

/// Oldest binary source-file protocol version this decoder understands.
pub const MIN_SOURCE_FILE_PROTOCOL_VERSION: u8 = 7;

/// Newest binary source-file protocol version this decoder understands.
///
/// The source-file level layout is identical across the supported range;
/// version 8 only changed how the node table records a `NodeList`'s trailing
/// comma, which this decoder never reads.
pub const MAX_SOURCE_FILE_PROTOCOL_VERSION: u8 = 8;

const HEADER_SIZE: usize = 44;
const HEADER_OFFSET_STRING_OFFSETS: usize = 24;
const HEADER_OFFSET_STRING_DATA: usize = 28;
const HEADER_OFFSET_EXTENDED_DATA: usize = 32;
const HEADER_OFFSET_STRUCTURED_DATA: usize = 36;
const HEADER_OFFSET_NODES: usize = 40;

const NODE_SIZE: usize = 28;
const NODE_OFFSET_DATA: usize = 20;
const NODE_DATA_TYPE_MASK: u32 = 0xc000_0000;
const NODE_DATA_TYPE_EXTENDED: u32 = 2 << 30;
const NODE_DATA_PAYLOAD_MASK: u32 = 0x00ff_ffff;

/// Sentinel written where a source file has no value for a field.
const NO_STRUCTURED_DATA: u32 = 0xffff_ffff;

// Byte offsets inside the `SourceFile` extended data record.
const EXT_TEXT: usize = 0;
const EXT_FILE_NAME: usize = 4;
const EXT_PATH: usize = 8;
const EXT_LANGUAGE_VARIANT: usize = 12;
const EXT_SCRIPT_KIND: usize = 16;
const EXT_ORIGINAL_TEXT: usize = 48;
const EXT_SPAN_MAP: usize = 52;
const EXT_SUPPLEMENTAL_FILE_NAMES: usize = 56;
const EXT_CANONICAL_FILE_NAME: usize = 60;
const EXT_CONTENT_MAPPER: usize = 64;
const EXT_VIRTUAL_FILE_NAME: usize = 68;
const EXT_DIAGNOSTIC_DIRECTIVES: usize = 72;
const EXT_SOURCE_FILE_SIZE: usize = 76;

/// Content mapper state attached to one source file.
///
/// Present only for files a mapper produced. `span_map` may still be empty,
/// which upstream uses to describe fully synthesized output that has no
/// counterpart in the original file.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapping {
    /// `name@version` identity of the mapper that produced the file.
    pub content_mapper: String,
    /// Filename whose extension decided how the virtual text was parsed.
    pub virtual_file_name: String,
    /// Mapping between virtual and original positions, in UTF-16 code units.
    pub span_map: SpanMap,
    /// Directives that control diagnostics inside mapped ranges.
    pub diagnostic_directives: Vec<MappedDiagnosticDirective>,
    /// Compiler-assigned filenames of supplemental outputs for this file.
    pub supplemental_source_file_names: Vec<String>,
    /// Canonical file this output supplements, when it is a supplemental one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_source_file_name: Option<String>,
}

impl ContentMapping {
    /// Reports whether this file is a supplemental output of another file.
    pub fn is_supplemental(&self) -> bool {
        self.canonical_source_file_name.is_some()
    }
}

/// Source-file level fields decoded from an encoded AST payload.
///
/// # Examples
///
/// ```no_run
/// use corsa_client::{
///     ApiClient, EncodedSourceFile, Result, SpanMapFeature, UpdateSnapshotParams,
/// };
///
/// async fn report_mapper(client: &ApiClient) -> Result<()> {
///     let snapshot = client
///         .update_snapshot(UpdateSnapshotParams {
///             open_project: Some("./tsconfig.json".to_owned()),
///             ..UpdateSnapshotParams::default()
///         })
///         .await?;
///     let project = snapshot.projects[0].id.clone();
///     let payload = client
///         .get_source_file(snapshot.handle.clone(), project, "./src/App.vue")
///         .await?
///         .expect("the project contains the file");
///     let source_file = EncodedSourceFile::decode(payload.as_bytes())?;
///
///     if let Some(mapping) = source_file.content_mapping() {
///         // Turn a checker position in the virtual TypeScript back into a
///         // position in the `.vue` file the user edits.
///         let mapped = mapping
///             .span_map
///             .virtual_to_original_position_for_feature(42, SpanMapFeature::HOVER);
///         println!("{} -> {}", mapping.content_mapper, mapped.position);
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedSourceFile {
    /// Binary protocol version the payload was encoded with.
    pub protocol_version: u8,
    /// File name Corsa resolved for the source file.
    pub file_name: String,
    /// Canonicalized path Corsa keyed the source file by.
    pub path: String,
    /// Text the checker parsed. For a mapped file this is the virtual text.
    pub text: String,
    /// Text on disk. Equal to `text` for files no mapper touched.
    pub original_text: String,
    /// TypeScript `LanguageVariant` value.
    pub language_variant: u32,
    /// TypeScript `ScriptKind` value.
    pub script_kind: u32,
    /// Content mapper state, when a mapper produced this file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_mapping: Option<ContentMapping>,
}

impl EncodedSourceFile {
    /// Decodes the source-file fields out of an encoded AST payload.
    ///
    /// Only the header, the string table, the `SourceFile` node record, and the
    /// structured data it points at are read; the node table is skipped.
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let reader = PayloadReader::new(payload)?;
        let extended = reader.source_file_extended_data()?;

        let text_index = read_u32(extended, EXT_TEXT)?;
        let original_text_index = read_u32(extended, EXT_ORIGINAL_TEXT)?;
        let text = reader.string(text_index)?;
        let original_text = if original_text_index == text_index {
            text.clone()
        } else {
            reader.string(original_text_index)?
        };

        let span_map_offset = read_u32(extended, EXT_SPAN_MAP)?;
        let content_mapper_index = read_u32(extended, EXT_CONTENT_MAPPER)?;
        let content_mapping = if span_map_offset == NO_STRUCTURED_DATA
            && content_mapper_index == NO_STRUCTURED_DATA
        {
            None
        } else {
            Some(ContentMapping {
                content_mapper: reader
                    .optional_string(content_mapper_index)?
                    .unwrap_or_default(),
                virtual_file_name: reader
                    .optional_string(read_u32(extended, EXT_VIRTUAL_FILE_NAME)?)?
                    .unwrap_or_default(),
                span_map: reader.span_map(span_map_offset)?,
                diagnostic_directives: reader
                    .diagnostic_directives(read_u32(extended, EXT_DIAGNOSTIC_DIRECTIVES)?)?,
                supplemental_source_file_names: reader
                    .string_array(read_u32(extended, EXT_SUPPLEMENTAL_FILE_NAMES)?)?,
                canonical_source_file_name: reader
                    .optional_string(read_u32(extended, EXT_CANONICAL_FILE_NAME)?)?,
            })
        };

        Ok(Self {
            protocol_version: reader.protocol_version,
            file_name: reader.string(read_u32(extended, EXT_FILE_NAME)?)?,
            path: reader.string(read_u32(extended, EXT_PATH)?)?,
            text,
            original_text,
            language_variant: read_u32(extended, EXT_LANGUAGE_VARIANT)?,
            script_kind: read_u32(extended, EXT_SCRIPT_KIND)?,
            content_mapping,
        })
    }

    /// Reports whether a content mapper produced this file.
    pub fn is_content_mapped(&self) -> bool {
        self.content_mapping.is_some()
    }

    /// Content mapper state, when a mapper produced this file.
    pub fn content_mapping(&self) -> Option<&ContentMapping> {
        self.content_mapping.as_ref()
    }

    /// Span map between virtual and original text, when the file is mapped.
    pub fn span_map(&self) -> Option<&SpanMap> {
        self.content_mapping
            .as_ref()
            .map(|mapping| &mapping.span_map)
    }
}

impl EncodedPayload {
    /// Decodes the source-file fields of a `getSourceFile` payload.
    ///
    /// See [`EncodedSourceFile`] for what the decoded record carries.
    pub fn decode_source_file(&self) -> Result<EncodedSourceFile> {
        EncodedSourceFile::decode(self.as_bytes())
    }
}

/// Bounds-checked view over one encoded payload.
struct PayloadReader<'a> {
    payload: &'a [u8],
    protocol_version: u8,
    string_offsets: usize,
    string_data: usize,
    extended_data: usize,
    structured_data: usize,
    nodes: usize,
}

impl<'a> PayloadReader<'a> {
    fn new(payload: &'a [u8]) -> Result<Self> {
        if payload.len() < HEADER_SIZE {
            return Err(protocol(format_args!(
                "encoded source file is {} bytes, shorter than the {HEADER_SIZE} byte header",
                payload.len()
            )));
        }
        let protocol_version = payload[3];
        if !(MIN_SOURCE_FILE_PROTOCOL_VERSION..=MAX_SOURCE_FILE_PROTOCOL_VERSION)
            .contains(&protocol_version)
        {
            return Err(protocol(format_args!(
                "unsupported encoded source file protocol version {protocol_version} (this build decodes versions {MIN_SOURCE_FILE_PROTOCOL_VERSION} through {MAX_SOURCE_FILE_PROTOCOL_VERSION})"
            )));
        }
        let string_offsets = read_u32(payload, HEADER_OFFSET_STRING_OFFSETS)? as usize;
        let string_data = read_u32(payload, HEADER_OFFSET_STRING_DATA)? as usize;
        let extended_data = read_u32(payload, HEADER_OFFSET_EXTENDED_DATA)? as usize;
        let structured_data = read_u32(payload, HEADER_OFFSET_STRUCTURED_DATA)? as usize;
        let nodes = read_u32(payload, HEADER_OFFSET_NODES)? as usize;
        if !(string_offsets <= string_data
            && string_data <= extended_data
            && extended_data <= structured_data
            && structured_data <= nodes
            && nodes <= payload.len())
        {
            return Err(protocol(format_args!(
                "encoded source file sections are out of order or out of bounds: {string_offsets}, {string_data}, {extended_data}, {structured_data}, {nodes} in {} bytes",
                payload.len()
            )));
        }
        Ok(Self {
            payload,
            protocol_version,
            string_offsets,
            string_data,
            extended_data,
            structured_data,
            nodes,
        })
    }

    /// Returns the `SourceFile` node's extended data record.
    ///
    /// The encoder always writes the root node at index 1; index 0 is the nil
    /// sentinel every node record points at when it has no parent or sibling.
    fn source_file_extended_data(&self) -> Result<&'a [u8]> {
        let root = self.nodes + NODE_SIZE;
        if root + NODE_SIZE > self.payload.len() {
            return Err(protocol(format_args!(
                "encoded source file has no root node record at byte {root}"
            )));
        }
        let data = read_u32(self.payload, root + NODE_OFFSET_DATA)?;
        if data & NODE_DATA_TYPE_MASK != NODE_DATA_TYPE_EXTENDED {
            return Err(protocol(format_args!(
                "encoded root node does not carry source file extended data (node data {data:#010x})"
            )));
        }
        let start = self.extended_data + (data & NODE_DATA_PAYLOAD_MASK) as usize;
        let end = start + EXT_SOURCE_FILE_SIZE;
        self.payload.get(start..end).ok_or_else(|| {
            protocol(format_args!(
                "source file extended data at {start}..{end} is outside the payload"
            ))
        })
    }

    /// Reads the string at `index` in the string offsets table.
    fn string(&self, index: u32) -> Result<String> {
        let entry = self.string_offsets + index as usize * 4;
        let start = self.string_data + read_u32(self.payload, entry)? as usize;
        let end = self.string_data + read_u32(self.payload, entry + 4)? as usize;
        let bytes = self
            .payload
            .get(start..end.max(start))
            .filter(|_| end <= self.extended_data)
            .ok_or_else(|| {
                protocol(format_args!(
                    "string {index} spans {start}..{end}, outside the string data section"
                ))
            })?;
        // Corsa writes WTF-8 for JS strings that hold lone surrogates, so a
        // lossy decode keeps the rest of the payload usable.
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    fn optional_string(&self, index: u32) -> Result<Option<String>> {
        if index == NO_STRUCTURED_DATA {
            return Ok(None);
        }
        self.string(index).map(Some)
    }

    /// Returns a msgpack cursor over the structured data blob at `offset`.
    fn structured(&self, offset: u32) -> Result<Option<MsgpackCursor<'a>>> {
        if offset == NO_STRUCTURED_DATA {
            return Ok(None);
        }
        let start = self.structured_data + offset as usize;
        let bytes = self.payload.get(start..self.nodes).ok_or_else(|| {
            protocol(format_args!(
                "structured data at {start} is outside the structured data section"
            ))
        })?;
        Ok(Some(MsgpackCursor::new(bytes)))
    }

    fn span_map(&self, offset: u32) -> Result<SpanMap> {
        let Some(mut cursor) = self.structured(offset)? else {
            return Ok(SpanMap::default());
        };
        let count = cursor.array_len()?;
        let mut segments = Vec::with_capacity(count);
        for _ in 0..count {
            let fields = cursor.array_len()?;
            if !(5..=6).contains(&fields) {
                return Err(protocol(format_args!(
                    "span map segment has {fields} fields, expected 5 or 6"
                )));
            }
            let virtual_start = cursor.uint()?;
            let virtual_length = cursor.uint()?;
            let original_start = cursor.uint()?;
            let original_length = cursor.uint()?;
            let kind = SpanMapKind::try_from(u8::try_from(cursor.uint()?).unwrap_or(u8::MAX))
                .map_err(|error| protocol(format_args!("{error}")))?;
            let features = if fields == 6 {
                SpanMapFeature(cursor.uint()?)
            } else {
                SpanMapFeature::ALL
            };
            segments.push(SpanMapSegment {
                virtual_start,
                virtual_end: virtual_start.saturating_add(virtual_length),
                original_start,
                original_end: original_start.saturating_add(original_length),
                kind,
                features,
            });
        }
        Ok(SpanMap::new(segments))
    }

    fn diagnostic_directives(&self, offset: u32) -> Result<Vec<MappedDiagnosticDirective>> {
        let Some(mut cursor) = self.structured(offset)? else {
            return Ok(Vec::new());
        };
        let count = cursor.array_len()?;
        let mut directives = Vec::with_capacity(count);
        for _ in 0..count {
            let fields = cursor.array_len()?;
            if fields != 6 {
                return Err(protocol(format_args!(
                    "diagnostic directive has {fields} fields, expected 6"
                )));
            }
            let original_start = cursor.uint()?;
            let original_length = cursor.uint()?;
            let virtual_start = cursor.uint()?;
            let virtual_length = cursor.uint()?;
            let policy = DiagnosticDirectivePolicy::try_from(
                u8::try_from(cursor.uint()?).unwrap_or(u8::MAX),
            )
            .map_err(|error| protocol(format_args!("{error}")))?;
            directives.push(MappedDiagnosticDirective {
                original_range: TextRange::new(
                    original_start,
                    original_start.saturating_add(original_length),
                ),
                virtual_range: TextRange::new(
                    virtual_start,
                    virtual_start.saturating_add(virtual_length),
                ),
                policy,
                unused_code: cursor.uint()?,
            });
        }
        Ok(directives)
    }

    fn string_array(&self, offset: u32) -> Result<Vec<String>> {
        let Some(mut cursor) = self.structured(offset)? else {
            return Ok(Vec::new());
        };
        let count = cursor.array_len()?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(cursor.string()?);
        }
        Ok(values)
    }
}

/// Reader for the msgpack subset Corsa writes into the structured data section.
struct MsgpackCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> MsgpackCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self.position + count;
        let slice = self.bytes.get(self.position..end).ok_or_else(|| {
            protocol(format_args!(
                "structured data ended after {} bytes while reading {count} more",
                self.bytes.len()
            ))
        })?;
        self.position = end;
        Ok(slice)
    }

    fn marker(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn be_uint(&mut self, width: usize) -> Result<u32> {
        let bytes = self.take(width)?;
        Ok(bytes
            .iter()
            .fold(0_u64, |value, &byte| (value << 8) | u64::from(byte))
            .min(u64::from(u32::MAX)) as u32)
    }

    /// Reads an array header and returns its element count.
    fn array_len(&mut self) -> Result<usize> {
        let marker = self.marker()?;
        match marker {
            0x90..=0x9f => Ok(usize::from(marker & 0x0f)),
            0xdc => Ok(self.be_uint(2)? as usize),
            0xdd => Ok(self.be_uint(4)? as usize),
            other => Err(protocol(format_args!(
                "expected a msgpack array marker, got {other:#04x}"
            ))),
        }
    }

    /// Reads an unsigned integer.
    fn uint(&mut self) -> Result<u32> {
        let marker = self.marker()?;
        match marker {
            0x00..=0x7f => Ok(u32::from(marker)),
            0xcc => self.be_uint(1),
            0xcd => self.be_uint(2),
            0xce => self.be_uint(4),
            0xcf => self.be_uint(8),
            other => Err(protocol(format_args!(
                "expected a msgpack uint marker, got {other:#04x}"
            ))),
        }
    }

    /// Reads a string.
    fn string(&mut self) -> Result<String> {
        let marker = self.marker()?;
        let length = match marker {
            0xa0..=0xbf => usize::from(marker & 0x1f),
            0xd9 => self.be_uint(1)? as usize,
            0xda => self.be_uint(2)? as usize,
            0xdb => self.be_uint(4)? as usize,
            other => {
                return Err(protocol(format_args!(
                    "expected a msgpack string marker, got {other:#04x}"
                )));
            }
        };
        Ok(String::from_utf8_lossy(self.take(length)?).into_owned())
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    bytes
        .get(offset..offset + 4)
        .and_then(|slice| slice.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| {
            protocol(format_args!(
                "encoded source file ends before the 4 bytes at {offset}"
            ))
        })
}

fn protocol(message: std::fmt::Arguments<'_>) -> CorsaError {
    CorsaError::Protocol(compact_format(message))
}

#[cfg(test)]
#[path = "source_file_tests.rs"]
mod tests;

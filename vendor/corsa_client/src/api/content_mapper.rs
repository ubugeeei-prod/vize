//! Content mapper models mirrored from upstream TypeScript.
//!
//! A content mapper is a package declared in `tsconfig.json` under
//! `contentMappers` that transforms otherwise unsupported file content (for
//! example `.vue` or `.svelte`) into virtual TypeScript while the program is
//! built. The checker then reports positions inside the *virtual* text, so any
//! tool that wants to surface a result in the file the user actually edits has
//! to map those positions back through the span map the mapper produced.
//!
//! This module ports the upstream span map semantics
//! (`tsc/internal/spanmap/spanmap.go` and `packages/typescript/src/ast/spanMap.ts`)
//! so Rust and Node consumers get the same answers as the official client. The
//! span map for a file arrives inside the payload returned by
//! [`ApiClient::get_source_file`](crate::ApiClient::get_source_file); decode it
//! with [`EncodedSourceFile`](crate::EncodedSourceFile).
//!
//! # Examples
//!
//! ```
//! use corsa_client::{SpanMap, SpanMapFeature, SpanMapFidelity, SpanMapKind, SpanMapSegment, TextRange};
//!
//! // `<script>` block copied verbatim from offset 8 of a `.vue` file.
//! let map = SpanMap::new([SpanMapSegment::verbatim(0, 12, 8, 20)]);
//! let mapped = map.virtual_to_original_position(3);
//!
//! assert_eq!(mapped.position, 11);
//! assert_eq!(mapped.fidelity, SpanMapFidelity::Exact);
//!
//! // Synthesized prologue text has no counterpart in the original file.
//! let map = SpanMap::new([SpanMapSegment::new(10, 22, 8, 20, SpanMapKind::Verbatim)]);
//! assert_eq!(map.virtual_to_original_position(3).fidelity, SpanMapFidelity::None);
//!
//! // The reverse direction can produce several projections.
//! let projections = map.original_to_virtual_positions(9, SpanMapFeature::HOVER);
//! assert_eq!(projections.len(), 1);
//! assert_eq!(projections[0].position, 11);
//! # let _ = TextRange::new(0, 1);
//! ```

use std::{
    fmt,
    ops::{BitAnd, BitOr, BitOrAssign},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Virtual file extensions a content mapper is allowed to emit.
///
/// Mirrors `contentmapper.IsSupportedVirtualExtension` upstream.
pub const SUPPORTED_VIRTUAL_EXTENSIONS: [&str; 9] = [
    ".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts", ".json",
];

/// Reports whether Corsa accepts `extension` as a mapper's virtual extension.
///
/// # Examples
///
/// ```
/// use corsa_client::is_supported_virtual_extension;
///
/// assert!(is_supported_virtual_extension(".ts"));
/// assert!(!is_supported_virtual_extension(".vue"));
/// ```
pub fn is_supported_virtual_extension(extension: &str) -> bool {
    SUPPORTED_VIRTUAL_EXTENSIONS.contains(&extension)
}

/// Half-open range of UTF-16 code unit offsets.
///
/// Corsa encodes every content mapper position in UTF-16 code units, which is
/// also what LSP clients expect, so no re-encoding is needed on this path.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TextRange {
    /// Inclusive start offset.
    pub pos: u32,
    /// Exclusive end offset.
    pub end: u32,
}

impl TextRange {
    /// Creates a range, clamping `end` up to `pos` when it would be inverted.
    pub const fn new(pos: u32, end: u32) -> Self {
        Self {
            pos,
            end: if end < pos { pos } else { end },
        }
    }

    /// Creates the empty range at `position`.
    pub const fn empty(position: u32) -> Self {
        Self {
            pos: position,
            end: position,
        }
    }

    /// Returns the number of UTF-16 code units the range covers.
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.pos)
    }

    /// Reports whether the range covers no text.
    pub const fn is_empty(&self) -> bool {
        self.end <= self.pos
    }
}

/// How one span map segment relates its virtual text to the original text.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum SpanMapKind {
    /// The virtual text is a character-for-character copy of the original.
    #[default]
    Verbatim,
    /// The segment is indivisible: positions inside it map to its boundaries.
    Atom,
    /// The segment names the same entity as the original without copying it.
    Alias,
}

impl From<SpanMapKind> for u8 {
    fn from(kind: SpanMapKind) -> Self {
        match kind {
            SpanMapKind::Verbatim => 0,
            SpanMapKind::Atom => 1,
            SpanMapKind::Alias => 2,
        }
    }
}

impl TryFrom<u8> for SpanMapKind {
    type Error = UnknownSpanMapKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Verbatim),
            1 => Ok(Self::Atom),
            2 => Ok(Self::Alias),
            other => Err(UnknownSpanMapKind(other)),
        }
    }
}

/// Error returned when a runtime reports a span map kind this crate predates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownSpanMapKind(pub u8);

impl fmt::Display for UnknownSpanMapKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown span map kind {}", self.0)
    }
}

impl std::error::Error for UnknownSpanMapKind {}

/// How faithful a mapping between virtual and original text is.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum SpanMapFidelity {
    /// Precise, edit-safe projection through a single verbatim segment.
    #[default]
    Exact,
    /// The position or range lies inside one indivisible segment.
    Atom,
    /// The endpoints were mapped through different segments.
    Approximate,
    /// The input has no counterpart in the target text.
    None,
}

impl SpanMapFidelity {
    /// Reports whether the mapping is precise enough to drive an edit.
    pub const fn is_exact(self) -> bool {
        matches!(self, Self::Exact)
    }

    /// Reports whether the mapping stayed inside a single segment.
    pub const fn is_single_segment(self) -> bool {
        matches!(self, Self::Exact | Self::Atom)
    }

    /// Reports whether the input had no counterpart in the target text.
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

impl From<SpanMapFidelity> for u8 {
    fn from(fidelity: SpanMapFidelity) -> Self {
        match fidelity {
            SpanMapFidelity::Exact => 0,
            SpanMapFidelity::Atom => 1,
            SpanMapFidelity::Approximate => 2,
            SpanMapFidelity::None => 3,
        }
    }
}

impl TryFrom<u8> for SpanMapFidelity {
    type Error = UnknownSpanMapFidelity;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Exact),
            1 => Ok(Self::Atom),
            2 => Ok(Self::Approximate),
            3 => Ok(Self::None),
            other => Err(UnknownSpanMapFidelity(other)),
        }
    }
}

/// Error returned when a runtime reports a fidelity this crate predates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownSpanMapFidelity(pub u8);

impl fmt::Display for UnknownSpanMapFidelity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown span map fidelity {}", self.0)
    }
}

impl std::error::Error for UnknownSpanMapFidelity {}

/// Language-service features a span map segment opts into.
///
/// A mapper can restrict a segment so that, for example, hover works but rename
/// does not. Original-to-virtual queries take the feature being served and skip
/// segments that do not participate in it.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SpanMapFeature(pub u32);

impl SpanMapFeature {
    /// Hover.
    pub const HOVER: Self = Self(1 << 0);
    /// Signature help.
    pub const SIGNATURE_HELP: Self = Self(1 << 1);
    /// Completions.
    pub const COMPLETION: Self = Self(1 << 2);
    /// Go to definition.
    pub const DEFINITION: Self = Self(1 << 3);
    /// Go to type definition.
    pub const TYPE_DEFINITION: Self = Self(1 << 4);
    /// Go to implementation.
    pub const IMPLEMENTATION: Self = Self(1 << 5);
    /// Find all references.
    pub const REFERENCES: Self = Self(1 << 6);
    /// Document highlights.
    pub const DOCUMENT_HIGHLIGHTS: Self = Self(1 << 7);
    /// Rename.
    pub const RENAME: Self = Self(1 << 8);
    /// Call hierarchy.
    pub const CALL_HIERARCHY: Self = Self(1 << 9);
    /// Code actions.
    pub const CODE_ACTIONS: Self = Self(1 << 10);
    /// Formatting.
    pub const FORMATTING: Self = Self(1 << 11);
    /// Inlay hints.
    pub const INLAY_HINTS: Self = Self(1 << 12);
    /// Semantic tokens.
    pub const SEMANTIC_TOKENS: Self = Self(1 << 13);
    /// Folding ranges.
    pub const FOLDING_RANGES: Self = Self(1 << 14);
    /// Selection ranges.
    pub const SELECTION_RANGES: Self = Self(1 << 15);
    /// Linked editing ranges.
    pub const LINKED_EDITING: Self = Self(1 << 16);
    /// Auto-insert.
    pub const AUTO_INSERT: Self = Self(1 << 17);
    /// Document symbols.
    pub const DOCUMENT_SYMBOLS: Self = Self(1 << 18);
    /// Code lens.
    pub const CODE_LENS: Self = Self(1 << 19);
    /// No feature.
    pub const NONE: Self = Self(0);
    /// Every feature upstream currently defines.
    pub const ALL: Self = Self((1 << 20) - 1);

    /// Returns the full feature set, for use as a serde default.
    pub const fn all() -> Self {
        Self::ALL
    }

    /// Returns the raw bitset.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Reports whether every bit of `other` is set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Reports whether any bit of `other` is set.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for SpanMapFeature {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for SpanMapFeature {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for SpanMapFeature {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// One half-open virtual range mapped onto one half-open original range.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanMapSegment {
    /// Inclusive start of the segment in the virtual text.
    pub virtual_start: u32,
    /// Exclusive end of the segment in the virtual text.
    pub virtual_end: u32,
    /// Inclusive start of the segment in the original text.
    pub original_start: u32,
    /// Exclusive end of the segment in the original text.
    pub original_end: u32,
    /// How the two ranges relate.
    pub kind: SpanMapKind,
    /// Language-service features the segment participates in.
    #[serde(default = "SpanMapFeature::all")]
    pub features: SpanMapFeature,
}

impl SpanMapSegment {
    /// Creates a segment that participates in every feature.
    pub const fn new(
        virtual_start: u32,
        virtual_end: u32,
        original_start: u32,
        original_end: u32,
        kind: SpanMapKind,
    ) -> Self {
        Self {
            virtual_start,
            virtual_end,
            original_start,
            original_end,
            kind,
            features: SpanMapFeature::ALL,
        }
    }

    /// Creates a verbatim segment that participates in every feature.
    pub const fn verbatim(
        virtual_start: u32,
        virtual_end: u32,
        original_start: u32,
        original_end: u32,
    ) -> Self {
        Self::new(
            virtual_start,
            virtual_end,
            original_start,
            original_end,
            SpanMapKind::Verbatim,
        )
    }

    /// Restricts the segment to the given feature set.
    pub const fn with_features(mut self, features: SpanMapFeature) -> Self {
        self.features = features;
        self
    }

    /// The segment's range in the virtual text.
    pub const fn virtual_range(&self) -> TextRange {
        TextRange::new(self.virtual_start, self.virtual_end)
    }

    /// The segment's range in the original text.
    pub const fn original_range(&self) -> TextRange {
        TextRange::new(self.original_start, self.original_end)
    }

    const fn supports(&self, feature: SpanMapFeature) -> bool {
        self.features.intersects(feature)
    }

    const fn same_original_range(&self, other: &Self) -> bool {
        self.original_start == other.original_start && self.original_end == other.original_end
    }
}

/// One projection of a position, together with how faithful it is.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MappedPosition {
    /// Projected offset in the target text.
    pub position: u32,
    /// How faithful the projection is.
    pub fidelity: SpanMapFidelity,
}

/// One projection of a range, together with how faithful it is.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MappedRange {
    /// Projected range in the target text.
    pub range: TextRange,
    /// How faithful the projection is.
    pub fidelity: SpanMapFidelity,
}

/// How TypeScript diagnostics inside a mapped range should be treated.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum DiagnosticDirectivePolicy {
    /// Suppress diagnostics reported inside the range.
    #[default]
    Ignore,
    /// Require a diagnostic inside the range and report `unused_code` otherwise.
    Expect,
}

impl From<DiagnosticDirectivePolicy> for u8 {
    fn from(policy: DiagnosticDirectivePolicy) -> Self {
        match policy {
            DiagnosticDirectivePolicy::Ignore => 0,
            DiagnosticDirectivePolicy::Expect => 1,
        }
    }
}

impl TryFrom<u8> for DiagnosticDirectivePolicy {
    type Error = UnknownDiagnosticDirectivePolicy;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ignore),
            1 => Ok(Self::Expect),
            other => Err(UnknownDiagnosticDirectivePolicy(other)),
        }
    }
}

/// Error returned when a runtime reports a policy this crate predates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownDiagnosticDirectivePolicy(pub u8);

impl fmt::Display for UnknownDiagnosticDirectivePolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown diagnostic directive policy {}", self.0)
    }
}

impl std::error::Error for UnknownDiagnosticDirectivePolicy {}

/// A framework-specific directive that controls diagnostics in a mapped range.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MappedDiagnosticDirective {
    /// Range in the original file the directive was written against.
    pub original_range: TextRange,
    /// Range in the virtual text the directive applies to.
    pub virtual_range: TextRange,
    /// What to do with diagnostics inside the range.
    pub policy: DiagnosticDirectivePolicy,
    /// Diagnostic code reported when an `Expect` directive matched nothing.
    pub unused_code: u32,
}

/// A content mapper as declared in a `tsconfig.json` `contentMappers` entry.
///
/// # Examples
///
/// ```
/// use corsa_client::ContentMapperDefinition;
///
/// let raw = serde_json::json!({
///     "contentMappers": [{ "package": "vue-mapper", "extensions": [".vue"] }],
/// });
/// let mappers = ContentMapperDefinition::from_raw_config(&raw);
///
/// assert_eq!(mappers.len(), 1);
/// assert_eq!(mappers[0].package, "vue-mapper");
/// assert!(mappers[0].handles_extension(".vue"));
/// ```
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMapperDefinition {
    /// npm package that implements the mapper.
    #[serde(default)]
    pub package: String,
    /// Otherwise unsupported file extensions the mapper registers.
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Mapper-specific options forwarded verbatim to the mapper process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
}

impl ContentMapperDefinition {
    /// Reads the `contentMappers` array out of a raw `tsconfig` object.
    ///
    /// Returns an empty vector when the config declares no mappers, which is
    /// also what a runtime that predates content mappers produces.
    pub fn from_raw_config(raw: &Value) -> Vec<Self> {
        raw.get("contentMappers")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| serde_json::from_value(entry.clone()).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Reports whether this mapper claims `extension` (compared case-sensitively).
    pub fn handles_extension(&self, extension: &str) -> bool {
        self.extensions.iter().any(|declared| declared == extension)
    }
}

/// Lazily built interval index used for original-to-virtual lookups.
#[derive(Clone, Debug)]
struct OriginalIndex {
    segments: Vec<SpanMapSegment>,
    leaf_count: usize,
    max_ends: Vec<u32>,
}

/// Bidirectional, span-aware mapping between virtual and original text.
///
/// Segments are kept sorted by virtual start, exactly as upstream does, so the
/// binary searches below see the same ordering as the official client.
#[derive(Clone, Debug, Default)]
pub struct SpanMap {
    segments: Vec<SpanMapSegment>,
    original_index: OnceLock<OriginalIndex>,
}

impl PartialEq for SpanMap {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}

impl Eq for SpanMap {}

impl Serialize for SpanMap {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.segments.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SpanMap {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::new(Vec::<SpanMapSegment>::deserialize(deserializer)?))
    }
}

impl FromIterator<SpanMapSegment> for SpanMap {
    fn from_iter<T: IntoIterator<Item = SpanMapSegment>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl SpanMap {
    /// Creates a span map, sorting the segments by virtual start.
    pub fn new(segments: impl IntoIterator<Item = SpanMapSegment>) -> Self {
        let mut segments: Vec<SpanMapSegment> = segments.into_iter().collect();
        segments.sort_by_key(|segment| segment.virtual_start);
        Self {
            segments,
            original_index: OnceLock::new(),
        }
    }

    /// The segments in virtual order.
    pub fn segments(&self) -> &[SpanMapSegment] {
        &self.segments
    }

    /// Number of segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Reports whether the map has no segments.
    ///
    /// An empty map is valid: it describes fully synthesized virtual text that
    /// has no counterpart in the original file.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Maps a virtual range back to the original text.
    ///
    /// Gaps map to insertion points with [`SpanMapFidelity::None`], and ranges
    /// that cross a segment boundary map their endpoints separately and report
    /// [`SpanMapFidelity::Approximate`].
    pub fn virtual_to_original_span(&self, range: TextRange) -> MappedRange {
        self.map_range(range)
    }

    /// Maps a virtual range only when every covered segment serves `feature`.
    pub fn virtual_to_original_span_for_feature(
        &self,
        range: TextRange,
        feature: SpanMapFeature,
    ) -> MappedRange {
        let mapped = self.virtual_to_original_span(range);
        if self.virtual_range_supports_feature(range, feature) {
            mapped
        } else {
            MappedRange {
                fidelity: SpanMapFidelity::None,
                ..mapped
            }
        }
    }

    /// Maps a virtual position back to the original text.
    pub fn virtual_to_original_position(&self, position: u32) -> MappedPosition {
        self.map_point(position)
    }

    /// Maps a virtual position only when its segment serves `feature`.
    pub fn virtual_to_original_position_for_feature(
        &self,
        position: u32,
        feature: SpanMapFeature,
    ) -> MappedPosition {
        let mapped = self.virtual_to_original_position(position);
        let (index, inside) = segment_index_at(&self.segments, position);
        let supported = inside && index.is_some_and(|index| self.segments[index].supports(feature));
        if supported {
            mapped
        } else {
            MappedPosition {
                fidelity: SpanMapFidelity::None,
                ..mapped
            }
        }
    }

    /// Returns every virtual projection of an original position for `feature`.
    ///
    /// Segment ends are inclusive here, so a position on a segment boundary can
    /// project into both neighbours. Results are ordered by virtual position;
    /// uncovered or feature-disabled positions produce no results.
    pub fn original_to_virtual_positions(
        &self,
        position: u32,
        feature: SpanMapFeature,
    ) -> Vec<MappedPosition> {
        let index = self.original_index();
        let mut results: Vec<MappedPosition> = Vec::new();
        for group in segment_groups_at_original_position(index, position) {
            for segment in group.segments {
                if !segment.supports(feature) {
                    continue;
                }
                let mapped = if segment.kind == SpanMapKind::Verbatim {
                    MappedPosition {
                        position: map_verbatim_position(&segment, position, true),
                        fidelity: SpanMapFidelity::Exact,
                    }
                } else {
                    MappedPosition {
                        position: if group.at_end {
                            segment.virtual_end
                        } else {
                            segment.virtual_start
                        },
                        fidelity: SpanMapFidelity::Atom,
                    }
                };
                if !results.contains(&mapped) {
                    results.push(mapped);
                }
            }
        }
        results.sort_by_key(|result| result.position);
        results
    }

    /// Returns every feature-compatible virtual projection of an original range.
    ///
    /// A range contained by one or more segments produces one exact or atom
    /// result per matching segment. A range that starts in one duplicate group
    /// and ends in another yields the smallest candidate range around each
    /// location, reported as [`SpanMapFidelity::Approximate`], rather than one
    /// range spanning every copy.
    pub fn original_to_virtual_spans(
        &self,
        range: TextRange,
        feature: SpanMapFeature,
    ) -> Vec<MappedRange> {
        let start = range.pos;
        let end = range.end.max(start);
        if start == end {
            return self
                .original_to_virtual_positions(start, feature)
                .into_iter()
                .map(|mapped| MappedRange {
                    range: TextRange::empty(mapped.position),
                    fidelity: mapped.fidelity,
                })
                .collect();
        }
        let index = self.original_index();
        let last_character = end - 1;
        let (Some(start_segments), Some(end_segments)) = (
            segments_at_original_position(index, start),
            segments_at_original_position(index, last_character),
        ) else {
            return Vec::new();
        };

        let containing: Vec<SpanMapSegment> = start_segments
            .iter()
            .copied()
            .filter(|segment| end <= segment.original_end)
            .collect();
        if !containing.is_empty() {
            let mut results =
                original_to_virtual_spans_in_segments(&containing, start, end, feature);
            if !results.is_empty() {
                results.sort_by_key(|result| result.range.pos);
                return results;
            }
        }

        let mut starts = original_projections(&start_segments, start, feature, false);
        let mut ends = original_projections(&end_segments, end, feature, true);
        starts.sort_unstable();
        ends.sort_unstable();
        if starts.is_empty() || ends.is_empty() {
            return Vec::new();
        }
        starts
            .iter()
            .enumerate()
            .filter_map(|(position, &virtual_start)| {
                let virtual_end = *ends.iter().find(|&&end| end >= virtual_start)?;
                let shadowed = starts
                    .get(position + 1)
                    .is_some_and(|&next| next <= virtual_end);
                (!shadowed).then_some(MappedRange {
                    range: TextRange::new(virtual_start, virtual_end),
                    fidelity: SpanMapFidelity::Approximate,
                })
            })
            .collect()
    }

    fn map_range(&self, range: TextRange) -> MappedRange {
        let start = range.pos;
        let end = range.end.max(start);
        if start == end {
            let mapped = self.map_point(start);
            return MappedRange {
                range: TextRange::empty(mapped.position),
                fidelity: mapped.fidelity,
            };
        }
        let (start_index, start_inside) = segment_index_at(&self.segments, start);
        let (end_index, end_inside) = segment_index_at(&self.segments, end - 1);

        if start_index == end_index && start_inside == end_inside {
            let Some(index) = start_index.filter(|_| start_inside) else {
                let position = insertion_point(&self.segments, start_index);
                return MappedRange {
                    range: TextRange::empty(position),
                    fidelity: SpanMapFidelity::None,
                };
            };
            let segment = &self.segments[index];
            if segment.kind == SpanMapKind::Verbatim {
                let mapped_start = map_verbatim_position(segment, start, false);
                let mapped_end = mapped_start.max(map_verbatim_position(segment, end, false));
                return MappedRange {
                    range: TextRange::new(mapped_start, mapped_end),
                    fidelity: SpanMapFidelity::Exact,
                };
            }
            return MappedRange {
                range: segment.original_range(),
                fidelity: SpanMapFidelity::Atom,
            };
        }

        let mapped_start = map_boundary(&self.segments, start, start_index, start_inside, false);
        let mapped_end = mapped_start.max(map_boundary(
            &self.segments,
            end,
            end_index,
            end_inside,
            true,
        ));
        MappedRange {
            range: TextRange::new(mapped_start, mapped_end),
            fidelity: SpanMapFidelity::Approximate,
        }
    }

    fn map_point(&self, position: u32) -> MappedPosition {
        let (index, inside) = segment_index_at(&self.segments, position);
        let Some(index) = index.filter(|_| inside) else {
            return MappedPosition {
                position: insertion_point(&self.segments, index),
                fidelity: SpanMapFidelity::None,
            };
        };
        let segment = &self.segments[index];
        if segment.kind == SpanMapKind::Verbatim {
            return MappedPosition {
                position: map_verbatim_position(segment, position, false),
                fidelity: SpanMapFidelity::Exact,
            };
        }
        MappedPosition {
            position: segment.original_start,
            fidelity: SpanMapFidelity::Atom,
        }
    }

    fn virtual_range_supports_feature(&self, range: TextRange, feature: SpanMapFeature) -> bool {
        let start = range.pos;
        let end = range.end.max(start);
        if start == end {
            let (index, inside) = segment_index_at(&self.segments, start);
            return inside && index.is_some_and(|index| self.segments[index].supports(feature));
        }
        let (index, inside) = segment_index_at(&self.segments, start);
        if !inside {
            return false;
        }
        let Some(mut index) = index else {
            return false;
        };
        let mut covered_through = start;
        while index < self.segments.len() && covered_through < end {
            let segment = &self.segments[index];
            if segment.virtual_start > covered_through
                || segment.virtual_end <= covered_through
                || !segment.supports(feature)
            {
                return false;
            }
            covered_through = segment.virtual_end;
            index += 1;
        }
        covered_through >= end
    }

    fn original_index(&self) -> &OriginalIndex {
        self.original_index.get_or_init(|| {
            let mut segments = self.segments.clone();
            segments.sort_by(|left, right| {
                left.original_start
                    .cmp(&right.original_start)
                    .then(left.original_end.cmp(&right.original_end))
                    .then(left.virtual_start.cmp(&right.virtual_start))
            });
            let mut leaf_count = 1;
            while leaf_count < segments.len() {
                leaf_count *= 2;
            }
            let mut max_ends = vec![0_u32; 2 * leaf_count];
            for (offset, segment) in segments.iter().enumerate() {
                max_ends[leaf_count + offset] = segment.original_end;
            }
            for node in (1..leaf_count).rev() {
                max_ends[node] = max_ends[2 * node].max(max_ends[2 * node + 1]);
            }
            OriginalIndex {
                segments,
                leaf_count,
                max_ends,
            }
        })
    }
}

struct SegmentGroup {
    segments: Vec<SpanMapSegment>,
    at_end: bool,
}

/// Maps an original-range boundary through every matching segment.
///
/// `at_end` selects the exclusive-end behaviour: the caller has already located
/// the segments with `end - 1`, so atoms project to their virtual end.
fn original_projections(
    segments: &[SpanMapSegment],
    position: u32,
    feature: SpanMapFeature,
    at_end: bool,
) -> Vec<u32> {
    segments
        .iter()
        .filter(|segment| segment.supports(feature))
        .map(|segment| {
            if segment.kind == SpanMapKind::Verbatim {
                map_verbatim_position(segment, position, true)
            } else if at_end {
                segment.virtual_end
            } else {
                segment.virtual_start
            }
        })
        .collect()
}

/// Maps a range fully contained by each segment.
fn original_to_virtual_spans_in_segments(
    segments: &[SpanMapSegment],
    start: u32,
    end: u32,
    feature: SpanMapFeature,
) -> Vec<MappedRange> {
    segments
        .iter()
        .filter(|segment| segment.supports(feature))
        .map(|segment| {
            if segment.kind == SpanMapKind::Verbatim {
                let mapped_start = map_verbatim_position(segment, start, true);
                let mapped_end = mapped_start.max(map_verbatim_position(segment, end, true));
                MappedRange {
                    range: TextRange::new(mapped_start, mapped_end),
                    fidelity: SpanMapFidelity::Exact,
                }
            } else {
                MappedRange {
                    range: segment.virtual_range(),
                    fidelity: SpanMapFidelity::Atom,
                }
            }
        })
        .collect()
}

/// Returns every segment containing the original-text `position`.
///
/// Segment ends are exclusive; starts, including zero-length segment starts,
/// are included.
fn segments_at_original_position(
    index: &OriginalIndex,
    position: u32,
) -> Option<Vec<SpanMapSegment>> {
    let start = first_original_segment_at_or_after(&index.segments, position);
    let mut results = segments_ending_at_or_after(index, start, position, false);
    let end = first_original_segment_after(&index.segments, position);
    results.extend_from_slice(&index.segments[start..end]);
    (!results.is_empty()).then_some(results)
}

/// Returns segments among `[0, limit)` whose original end reaches `position`.
fn segments_ending_at_or_after(
    index: &OriginalIndex,
    limit: usize,
    position: u32,
    include_end: bool,
) -> Vec<SpanMapSegment> {
    let mut results = Vec::new();
    collect_segments_ending_at_or_after(
        index,
        1,
        0,
        index.leaf_count,
        limit,
        position,
        include_end,
        &mut results,
    );
    results
}

/// Walks the flat max-end tree left to right, preserving original-text order.
#[allow(clippy::too_many_arguments)]
fn collect_segments_ending_at_or_after(
    index: &OriginalIndex,
    node: usize,
    start: usize,
    end: usize,
    limit: usize,
    position: u32,
    include_end: bool,
    results: &mut Vec<SpanMapSegment>,
) {
    let max_end = index.max_ends[node];
    if start >= limit || max_end < position || (!include_end && max_end == position) {
        return;
    }
    if end - start == 1 {
        results.push(index.segments[start]);
        return;
    }
    let middle = start + (end - start) / 2;
    collect_segments_ending_at_or_after(
        index,
        2 * node,
        start,
        middle,
        limit,
        position,
        include_end,
        results,
    );
    collect_segments_ending_at_or_after(
        index,
        2 * node + 1,
        middle,
        end,
        limit,
        position,
        include_end,
        results,
    );
}

fn first_original_segment_at_or_after(segments: &[SpanMapSegment], position: u32) -> usize {
    segments.partition_point(|segment| segment.original_start < position)
}

fn first_original_segment_after(segments: &[SpanMapSegment], position: u32) -> usize {
    segments.partition_point(|segment| segment.original_start <= position)
}

/// Returns every group of equal-range segments containing or touching `position`.
fn segment_groups_at_original_position(index: &OriginalIndex, position: u32) -> Vec<SegmentGroup> {
    let limit = first_original_segment_after(&index.segments, position);
    let segments = segments_ending_at_or_after(index, limit, position, true);
    let mut groups = Vec::new();
    let mut start = 0;
    while start < segments.len() {
        let mut end = start + 1;
        while end < segments.len() && segments[start].same_original_range(&segments[end]) {
            end += 1;
        }
        let segment = segments[start];
        if position <= segment.original_end {
            groups.push(SegmentGroup {
                segments: segments[start..end].to_vec(),
                at_end: position == segment.original_end && position != segment.original_start,
            });
        }
        start = end;
    }
    groups
}

/// Finds the segment containing `position` in virtual coordinates, or the
/// preceding segment when `position` falls in a gap.
///
/// The boolean distinguishes containment from a gap.
fn segment_index_at(segments: &[SpanMapSegment], position: u32) -> (Option<usize>, bool) {
    let low = segments.partition_point(|segment| segment.virtual_start < position);
    if segments
        .get(low)
        .is_some_and(|segment| segment.virtual_start == position)
    {
        return (Some(low), true);
    }
    let Some(previous) = low.checked_sub(1) else {
        return (None, false);
    };
    let end = segments[previous].virtual_end;
    if position < end || (previous == segments.len() - 1 && position == end) {
        return (Some(previous), true);
    }
    (Some(previous), false)
}

/// Returns the original-text insertion point for a gap after `previous`.
fn insertion_point(segments: &[SpanMapSegment], previous: Option<usize>) -> u32 {
    previous.map_or(0, |previous| segments[previous].original_end)
}

/// Linearly maps and clamps a position inside a length-preserving segment.
fn map_verbatim_position(segment: &SpanMapSegment, position: u32, reverse: bool) -> u32 {
    let (source_start, target_start, target_end) = if reverse {
        (
            segment.original_start,
            segment.virtual_start,
            segment.virtual_end,
        )
    } else {
        (
            segment.virtual_start,
            segment.original_start,
            segment.original_end,
        )
    };
    let shifted = i64::from(target_start) + i64::from(position) - i64::from(source_start);
    shifted.clamp(i64::from(target_start), i64::from(target_end)) as u32
}

/// Maps a range boundary, using insertion points for gaps and the selected
/// endpoint for atoms.
fn map_boundary(
    segments: &[SpanMapSegment],
    position: u32,
    index: Option<usize>,
    inside: bool,
    high: bool,
) -> u32 {
    let Some(index) = index.filter(|_| inside) else {
        return insertion_point(segments, index);
    };
    let segment = &segments[index];
    if segment.kind == SpanMapKind::Verbatim {
        return map_verbatim_position(segment, position, false);
    }
    if high {
        segment.original_end
    } else {
        segment.original_start
    }
}

#[cfg(test)]
#[path = "content_mapper_tests.rs"]
mod tests;

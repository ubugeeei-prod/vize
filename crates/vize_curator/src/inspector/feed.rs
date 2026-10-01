//! The Spolvero feed v1 (P2-18): the observer's folio output as one
//! consumable, schema-versioned JSON document.
//!
//! A [`StageFeed`] is the payload a folio directory (or an inspector
//! payload) hands to Spolvero consumers: a `schema_version` to negotiate on
//! (`devtool.md`'s data-layer requirement - a consumer refuses a mismatch
//! loudly instead of misrendering), the producing surface's `command`, and
//! the folio pages in emission order. The committed schema is
//! `docs/davinci/plan/spolvero-feed.schema.json`, validated in tests through
//! the strict TS-15 subset validator - one producer here, one validator in
//! the tree.
//!
//! # Relation to [`Collector`] (build on, not duplicate)
//!
//! P2-13's `Collector` already collects everything the feed carries: which
//! passes emitted a page, in what order, with what canonical text, under the
//! `--folio-after-change` hash gate. The feed is a *serialization* of that
//! collection - [`StageFeed::of_dump`] copies the dump's pages verbatim
//! and never re-decides gating or ordering. Surfaces without a pass pipeline
//! (the inspector's L1 pages, produced by a parse rather than a pass) push
//! [`StagePage`]s directly, with `pass` naming the producing step.
//!
//! Like the dump, the feed is deliberately IO-free and
//! transport-agnostic: [`StageFeed::to_json`] returns text, and whether
//! that text becomes a file in the folio directory, a member of the
//! inspector payload, or a protocol frame is the caller's decision (the
//! transport itself is P2-19's open question, not this module's).

use alloc::vec::Vec;
use core::fmt::Write as _;

use vize_l0::String;

use vize_l0::dump::collector::Collector;
use vize_l0::dump::json::push_json_string;
use vize_l0::dump::remarks::push_remark_fields;
use vize_l0::pass::observer::RecordedRemark;

/// The feed format version. Incompatible shape changes bump this **and**
/// the committed schema's `const` together.
pub const SPOLVERO_FEED_SCHEMA_VERSION: u32 = 1;

/// A consumer-side schema negotiation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageFeedSchemaMismatch {
    /// The feed schema this crate knows how to consume.
    pub expected: u32,
    /// The feed schema presented by the payload.
    pub found: u32,
}

/// One page of the feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagePage {
    /// Source file the page was produced for, when the producing surface
    /// works per-file (the inspector). `None` for a single-artifact
    /// historical single-artifact pipeline run.
    pub path: Option<String>,
    /// Stage that produced the page (`s1`, `s2`, `croquis`, ...).
    pub stage: String,
    /// Producing step: the pass name for a pipeline dump, or the
    /// non-pass step that made the page (`parse` for the L1 surface tree).
    pub pass: String,
    /// The page text: the artifact's canonical `Full`-mode folio text, or
    /// for L1 the byte-faithful surface render.
    pub text: String,
}

/// An executed inspection that could not represent its native payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageUnavailable {
    pub path: Option<String>,
    pub stage: String,
    pub pass: String,
    pub reason: String,
}

/// One optimization remark in the feed (P3-13): the recorded remark plus
/// the file it was produced for. Spans are byte offsets into the artifact
/// the pipeline lowered - the same frame as that file's pages (the template
/// content for the inspector and `analyzeSfc`, or a single typed artifact).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageRemark {
    /// Source file, as for [`StagePage::path`].
    pub path: Option<String>,
    /// The remark (`remarks-format.md` shape).
    pub remark: RecordedRemark,
}

/// The Spolvero feed v1 payload.
///
/// `schema_version` is not a field: it is [`SPOLVERO_FEED_SCHEMA_VERSION`],
/// emitted by [`to_json`](Self::to_json), so a feed value cannot carry a
/// version its shape does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageFeed {
    /// The producing surface (for example `inspector` or `analyze-sfc`).
    pub command: String,
    /// The pages, in emission order.
    pub pages: Vec<StagePage>,
    /// The optimization remarks (P3-13), per file in canonical order - the
    /// decision explanations Spolvero renders beside the pages. Additive
    /// to v1: every producer emits the member (possibly empty), and the
    /// schema keeps it optional so earlier v1 documents stay valid.
    pub remarks: Vec<StageRemark>,
    /// Separate failed inspections; omitted from the wire when empty.
    pub unavailable: Vec<StageUnavailable>,
}

impl StageFeed {
    /// Negotiate the feed schema before reading any shape-dependent field.
    pub fn negotiate_schema_version(schema_version: u32) -> Result<(), StageFeedSchemaMismatch> {
        if schema_version == SPOLVERO_FEED_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(StageFeedSchemaMismatch {
                expected: SPOLVERO_FEED_SCHEMA_VERSION,
                found: schema_version,
            })
        }
    }

    /// An empty feed for `command`.
    #[must_use]
    pub fn new(command: &str) -> Self {
        Self {
            command: String::from(command),
            pages: Vec::new(),
            remarks: Vec::new(),
            unavailable: Vec::new(),
        }
    }

    /// The feed of a [`Collector`]: the dump's pages, verbatim and in
    /// emission order. An empty (fully hash-gated) dump becomes a feed
    /// with zero pages - "the gate emitted nothing" stays observable.
    #[must_use]
    pub fn of_dump(command: &str, dump: &Collector) -> Self {
        Self {
            command: String::from(command),
            pages: dump
                .pages
                .iter()
                .map(|page| StagePage {
                    path: None,
                    stage: page.stage.clone(),
                    pass: page.pass.clone(),
                    text: page.text.clone(),
                })
                .collect(),
            remarks: Vec::new(),
            unavailable: Vec::new(),
        }
    }

    /// Serialize to the committed v1 JSON shape: one line, key order
    /// `schema_version`, `command`, `pages` (pages: `path`, `stage`,
    /// `pass`, `text`), `remarks` (`path`, then the remark document's item
    /// fields), trailing newline.
    ///
    /// Hand-written rather than serde-derived so the `no_std + alloc`
    /// library stays dependency-free; the escaping law (output parses back
    /// to exactly the input text) is pinned by the TS-52 tests.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::default();
        out.push_str("{\"schema_version\":");
        let _ = write!(out, "{SPOLVERO_FEED_SCHEMA_VERSION}");
        out.push_str(",\"command\":");
        push_json_string(&mut out, self.command.as_str());
        out.push_str(",\"pages\":[");
        for (index, page) in self.pages.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str("{\"path\":");
            match page.path.as_deref() {
                Some(path) => push_json_string(&mut out, path),
                None => out.push_str("null"),
            }
            out.push_str(",\"stage\":");
            push_json_string(&mut out, page.stage.as_str());
            out.push_str(",\"pass\":");
            push_json_string(&mut out, page.pass.as_str());
            out.push_str(",\"text\":");
            push_json_string(&mut out, page.text.as_str());
            out.push('}');
        }
        out.push_str("],\"remarks\":[");
        for (index, entry) in self.remarks.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str("{\"path\":");
            match entry.path.as_deref() {
                Some(path) => push_json_string(&mut out, path),
                None => out.push_str("null"),
            }
            out.push(',');
            push_remark_fields(&mut out, &entry.remark);
            out.push('}');
        }
        out.push(']');
        if !self.unavailable.is_empty() {
            out.push_str(",\"unavailable\":[");
            for (index, failure) in self.unavailable.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str("{\"path\":");
                match failure.path.as_deref() {
                    Some(path) => push_json_string(&mut out, path),
                    None => out.push_str("null"),
                }
                out.push_str(",\"stage\":");
                push_json_string(&mut out, failure.stage.as_str());
                out.push_str(",\"pass\":");
                push_json_string(&mut out, failure.pass.as_str());
                out.push_str(",\"reason\":");
                push_json_string(&mut out, failure.reason.as_str());
                out.push('}');
            }
            out.push(']');
        }
        out.push_str("}\n");
        out
    }
}

#[cfg(test)]
#[path = "feed/fallible_tests.rs"]
mod fallible_tests;

//! The Spolvero feed v1 (P2-18): the observer's folio output as one
//! consumable, schema-versioned JSON document.
//!
//! A [`SpolveroFeed`] is the payload a folio directory (or an inspector
//! payload) hands to Spolvero consumers: a `schema_version` to negotiate on
//! (`devtool.md`'s data-layer requirement - a consumer refuses a mismatch
//! loudly instead of misrendering), the producing surface's `command`, and
//! the folio pages in emission order. The committed schema is
//! `davinci-road/plan/spolvero-feed.schema.json`, validated in tests through
//! the strict TS-15 subset validator - one producer here, one validator in
//! the tree.
//!
//! # Relation to [`FolioDump`] (build on, not duplicate)
//!
//! P2-13's `FolioDump` already collects everything the feed carries: which
//! passes emitted a page, in what order, with what canonical text, under the
//! `--folio-after-change` hash gate. The feed is a *serialization* of that
//! collection - [`SpolveroFeed::of_dump`] copies the dump's pages verbatim
//! and never re-decides gating or ordering. Surfaces without a pass pipeline
//! (the inspector's S1 pages, produced by a parse rather than a pass) push
//! [`SpolveroPage`]s directly, with `pass` naming the producing step.
//!
//! Like the dump, the feed is deliberately IO-free (`no_std + alloc`) and
//! transport-agnostic: [`SpolveroFeed::to_json`] returns text, and whether
//! that text becomes a file in the folio directory, a member of the
//! inspector payload, or a protocol frame is the caller's decision (the
//! transport itself is P2-19's open question, not this module's).

use alloc::vec::Vec;
use core::fmt::Write as _;

use vize_s0::String;

use crate::folio::dump::FolioDump;

/// The feed format version. Incompatible shape changes bump this **and**
/// the committed schema's `const` together.
pub const SPOLVERO_FEED_SCHEMA_VERSION: u32 = 1;

/// A consumer-side schema negotiation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpolveroFeedSchemaMismatch {
    /// The feed schema this crate knows how to consume.
    pub expected: u32,
    /// The feed schema presented by the payload.
    pub found: u32,
}

/// One page of the feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpolveroPage {
    /// Source file the page was produced for, when the producing surface
    /// works per-file (the inspector). `None` for a single-artifact
    /// pipeline run (`davinci-opt`, whose artifact arrives on stdin).
    pub path: Option<String>,
    /// Stage that produced the page (`s1`, `s2`, `croquis`, ...).
    pub stage: String,
    /// Producing step: the pass name for a pipeline dump, or the
    /// non-pass step that made the page (`parse` for the S1 surface tree).
    pub pass: String,
    /// The page text: the artifact's canonical `Full`-mode folio text, or
    /// for S1 the byte-faithful surface render.
    pub text: String,
}

/// The Spolvero feed v1 payload.
///
/// `schema_version` is not a field: it is [`SPOLVERO_FEED_SCHEMA_VERSION`],
/// emitted by [`to_json`](Self::to_json), so a feed value cannot carry a
/// version its shape does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpolveroFeed {
    /// The producing surface (`davinci-opt`, `inspector`, `analyze-sfc`).
    pub command: String,
    /// The pages, in emission order.
    pub pages: Vec<SpolveroPage>,
}

impl SpolveroFeed {
    /// Negotiate the feed schema before reading any shape-dependent field.
    pub fn negotiate_schema_version(schema_version: u32) -> Result<(), SpolveroFeedSchemaMismatch> {
        if schema_version == SPOLVERO_FEED_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(SpolveroFeedSchemaMismatch {
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
        }
    }

    /// The feed of a [`FolioDump`]: the dump's pages, verbatim and in
    /// emission order. An empty (fully hash-gated) dump becomes a feed
    /// with zero pages - "the gate emitted nothing" stays observable.
    #[must_use]
    pub fn of_dump(command: &str, dump: &FolioDump) -> Self {
        Self {
            command: String::from(command),
            pages: dump
                .pages
                .iter()
                .map(|page| SpolveroPage {
                    path: None,
                    stage: page.stage.clone(),
                    pass: page.pass.clone(),
                    text: page.text.clone(),
                })
                .collect(),
        }
    }

    /// Serialize to the committed v1 JSON shape: one line, key order
    /// `schema_version`, `command`, `pages` (pages: `path`, `stage`,
    /// `pass`, `text`), trailing newline.
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
        out.push_str("]}\n");
        out
    }
}

/// Append `text` as a JSON string literal: `"` and `\` escaped, control
/// characters as `\n`/`\r`/`\t` or `\u00XX`, everything else (multi-byte
/// UTF-8 included) verbatim - the minimal escape set RFC 8259 requires.
fn push_json_string(out: &mut String, text: &str) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control < ' ' => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

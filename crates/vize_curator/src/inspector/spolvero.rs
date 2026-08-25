//! Spolvero feed construction for the inspector (Davinci P2-18).
//!
//! The inspector's stage pages, in the feed shape `vize_davinci` owns
//! (`folio::feed::SpolveroFeed`, committed schema
//! `davinci-road/plan/spolvero-feed.schema.json`). There is exactly one
//! serializer of that shape - `SpolveroFeed::to_json` - so this module
//! builds pages and parses the feed's own output into the
//! `serde_json::Value` the payload embeds; it never re-encodes the shape.
//!
//! # What the feed carries today
//!
//! - **S1**: one page per `.vue` file with a template block, produced by
//!   parsing the template into `vize_s1`'s lossless surface tree and
//!   rendering it back (`stage: "s1"`, `pass: "parse"` - a parse product,
//!   not a pass product). By the S1 byte-fidelity law (TS-19) the text
//!   equals the authored template bytes, malformed input included - which
//!   is exactly what the ladder's S1 rung shows, proven through the tree
//!   rather than copied from the source.
//! - **S2**: nothing yet. `vize_s2` has no producer until the S1→S2
//!   lowering (P2-8) lands; the feed shape is stage-agnostic, so S2 pages
//!   join by pushing more [`SpolveroPage`]s here, with no schema change.
//!
//! Files that are not `.vue`, fail SFC parsing, or have no template block
//! contribute no page: the feed is a stage-dump channel, not a diagnostics
//! channel (diagnostics stay on their own surfaces).

use vize_carton::{Allocator, String, cstr};
pub use vize_davinci::folio::feed::{SpolveroFeed, SpolveroPage};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

use super::payload::InspectorSourceFile;

/// The S1 page for one template: S1 parse + byte-faithful render.
#[must_use]
pub fn s1_page(path: &str, template: &str) -> SpolveroPage {
    let allocator = Allocator::default();
    let (tree, _errors) = vize_s1::parse(&allocator, template);
    let mut text = String::default();
    vize_s1::render::render(&tree, &mut |slice| text.push_str(slice));
    SpolveroPage {
        path: Some(String::from(path)),
        stage: cstr!("s1"),
        pass: cstr!("parse"),
        text,
    }
}

/// A feed's embeddable JSON value, through the one serializer.
///
/// # Panics
///
/// Never in practice: `SpolveroFeed::to_json` emits valid JSON by the
/// feed's escaping law (pinned by the TS-52 tests).
#[must_use]
pub fn spolvero_value(command: &str, pages: Vec<SpolveroPage>) -> serde_json::Value {
    let feed = SpolveroFeed {
        command: String::from(command),
        pages,
    };
    serde_json::from_str(feed.to_json().as_str())
        .expect("SpolveroFeed::to_json emits valid JSON by the feed escaping law")
}

/// The inspector payload's feed: S1 pages for every parseable `.vue` file
/// with a template, in payload file order (S2 joins with P2-8).
pub(super) fn payload_spolvero(files: &[InspectorSourceFile]) -> serde_json::Value {
    let mut pages = Vec::new();
    for file in files {
        if !file.path.ends_with(".vue") {
            continue;
        }
        let Ok(descriptor) = parse_sfc(file.source.as_str(), SfcParseOptions::default()) else {
            continue;
        };
        if let Some(template) = descriptor.template.as_ref() {
            pages.push(s1_page(file.path.as_str(), template.content.as_ref()));
        }
    }
    spolvero_value("inspector", pages)
}

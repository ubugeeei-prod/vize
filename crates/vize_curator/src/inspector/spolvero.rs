//! Spolvero feed construction for the inspector (Davinci P2-18).
//!
//! The inspector's stage pages, in the feed shape `vize_davinci` owns
//! (`folio::feed::SpolveroFeed`, committed schema
//! `docs/davinci/plan/spolvero-feed.schema.json`). There is exactly one
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
//! - **The full ladder** ([`ladder_pages`]): S1, the S2 (Disegno) lowering
//!   page, the transform plan's walks (`[fusion-plan-folio]`), one S2 page
//!   per executed transform pass, and the S3 (Impeto)
//!   graph, partition-fact and value pages - all from one S1 parse through
//!   the real lowerings and pass manager. The wasm `analyzeSfc` result (the
//!   playground's Davinci view) carries it. The inspector payload keeps its
//!   S1-only pages: it rides inside share URLs (the P2-18 growth note), and
//!   the playground recomputes the ladder from the same sources.
//! - **Remarks** (P3-13): every inline HTML template's optimization remarks
//!   from the S2 transform pipeline ([`template_remarks`]), spans in the
//!   template's byte frame (the pages' frame) - the decision explanations
//!   Spolvero renders (C-5).
//! - **Step and walk timings** ([`ladder_run`], [`ladder_profile`]): the
//!   same run timed by a host-supplied clock and exported as a P0-11 profile
//!   document (C-3), since the feed schema carries no timing.
//!
//! Files that are not `.vue`, fail SFC parsing, or have no template block
//! contribute no page: the feed is a stage-dump channel, not a diagnostics
//! channel (diagnostics stay on their own surfaces).

mod ladder;
mod profile;

pub use ladder::{LadderClock, LadderRun, LadderStep, ladder_pages, ladder_run};
pub use profile::{LADDER_STEP_KEY, LADDER_WALK_KEY, ladder_profile};
pub use vize_davinci::folio::feed::{SpolveroFeed, SpolveroPage, SpolveroRemark};
use vize_davinci::pass::RemarkCollector;
use vize_s0::{Allocator, String, cstr};

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

/// The optimization remarks (P3-13) the S2 transform pipeline emits for
/// one template: S1 parse, S1→S2 lowering (Vue 3 dialect), the transform
/// pipeline under a remark collector, in canonical order. Spans are byte
/// offsets into `template` - the frame of the feed's S1/S2 pages, so a
/// remark and the page lines it explains highlight the same source bytes.
#[must_use]
pub fn template_remarks(path: &str, template: &str) -> Vec<SpolveroRemark> {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, template);
    let mut lowered =
        vize_s1_to_s2::lower_with_caps(&allocator, &tree, &errors, vize_s1_to_s2::LegacyCaps::VUE3);
    let mut collector = RemarkCollector::new();
    let _facts = vize_s1_to_s2::pass::run_transform(&mut lowered, &mut collector);
    collector
        .finish()
        .into_iter()
        .map(|remark| SpolveroRemark {
            path: Some(String::from(path)),
            remark,
        })
        .collect()
}

/// A feed's embeddable JSON value, through the one serializer.
///
/// # Panics
///
/// Never in practice: `SpolveroFeed::to_json` emits valid JSON by the
/// feed's escaping law (pinned by the TS-52 tests).
#[must_use]
pub fn spolvero_value(command: &str, pages: Vec<SpolveroPage>) -> serde_json::Value {
    spolvero_value_with_remarks(command, pages, Vec::new())
}

/// [`spolvero_value`] carrying optimization remarks beside the pages.
///
/// # Panics
///
/// As [`spolvero_value`].
#[must_use]
pub fn spolvero_value_with_remarks(
    command: &str,
    pages: Vec<SpolveroPage>,
    remarks: Vec<SpolveroRemark>,
) -> serde_json::Value {
    let feed = SpolveroFeed {
        command: String::from(command),
        pages,
        remarks,
    };
    serde_json::from_str(feed.to_json().as_str())
        .expect("SpolveroFeed::to_json emits valid JSON by the feed escaping law")
}

/// The inspector payload's feed: S1 pages for every parseable `.vue` file
/// with a template, in payload file order (see the module docs for why the
/// payload stays S1-only), plus each inline HTML template's optimization
/// remarks (P3-13).
pub(super) fn payload_spolvero(files: &[InspectorSourceFile]) -> serde_json::Value {
    let mut pages = Vec::new();
    let mut remarks = Vec::new();
    for file in files {
        if !file.path.ends_with(".vue") {
            continue;
        }
        let Ok(descriptor) = parse_sfc(file.source.as_str(), SfcParseOptions::default()) else {
            continue;
        };
        if let Some(template) = descriptor.template.as_ref() {
            pages.push(s1_page(file.path.as_str(), template.content.as_ref()));
            let html = template.lang.as_deref().is_none_or(|lang| lang == "html");
            if template.src.is_none() && html {
                remarks.extend(template_remarks(
                    file.path.as_str(),
                    template.content.as_ref(),
                ));
            }
        }
    }
    spolvero_value_with_remarks("inspector", pages, remarks)
}

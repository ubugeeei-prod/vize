//! Five reviewed #13313 corrections to the immutable pre-retirement witness.
//!
//! A component (SFC) template now keeps its authored structure, as Vue 3.5
//! does. The frozen digests of these five cases recorded the HTML tree
//! construction's repairs (an implied `</li>`, a split nested `<a>`), which now
//! run only for documents or under `html_tree_construction`. Each correction
//! proves that lane still reproduces the frozen digest exactly, and pins the
//! authored digest the default lane produces.

use super::Parser;
use vize_l0::{Allocator, cstr, hash::StableHasher128};
use vize_relief::options::ParserOptions;

struct Correction {
    index: usize,
    context: &'static str,
    source: &'static str,
    after: [u8; 16],
    bit: u8,
}

const FIXTURES: [Correction; 4] = [
    Correction {
        index: 2127,
        context: "document=false; sibling_li_closes full@10",
        source: "<li>a<li>b",
        after: [
            159, 159, 132, 67, 101, 56, 111, 52, 236, 180, 94, 49, 31, 223, 60, 172,
        ],
        bit: 1,
    },
    Correction {
        index: 2129,
        context: "document=false; sibling_li_closes suffix@0",
        source: "<li>a<li>b",
        after: [
            159, 159, 132, 67, 101, 56, 111, 52, 236, 180, 94, 49, 31, 223, 60, 172,
        ],
        bit: 2,
    },
    Correction {
        index: 2144,
        context: "document=false; sibling_li_closes prefix@8",
        source: "<li>a<li",
        after: [
            215, 212, 86, 198, 220, 141, 98, 196, 250, 241, 154, 62, 11, 141, 122, 3,
        ],
        bit: 4,
    },
    Correction {
        index: 2146,
        context: "document=false; sibling_li_closes prefix@9",
        source: "<li>a<li>",
        after: [
            223, 99, 193, 133, 186, 31, 64, 185, 120, 118, 198, 66, 47, 184, 178, 205,
        ],
        bit: 8,
    },
];

const OPTIONS: [Correction; 1] = [Correction {
    index: 4,
    context: "<section><a><span><a></a><span v-pre>{{ inside }}</span>{{ tail }}</span></a></section>",
    source: "<section><a><span><a></a><span v-pre>{{ inside }}</span>{{ tail }}</span></a></section>",
    after: [
        73, 167, 161, 26, 184, 223, 245, 24, 75, 27, 139, 113, 31, 179, 152, 205,
    ],
    bit: 1,
}];

pub(super) const FIXTURE_HITS: u8 = 0b1111;
pub(super) const OPTION_HITS: u8 = 0b1;
/// The aggregates after the corrections. Their predecessors stay bound: the
/// empty-value witness still reads its `AFTER_AGGREGATE` from its capture, and
/// the options aggregate captured from 5bca3a881 was
/// `[139, 236, 84, 26, 227, 6, 37, 205, 133, 21, 41, 76, 178, 19, 23, 75]`.
pub(super) const FIXTURES_AGGREGATE: [u8; 16] = [
    251, 194, 32, 67, 178, 99, 136, 193, 234, 143, 126, 129, 0, 159, 54, 152,
];
pub(super) const OPTIONS_AGGREGATE: [u8; 16] = [
    116, 106, 38, 208, 244, 250, 175, 162, 153, 145, 14, 69, 116, 92, 30, 228,
];

/// `fixtures` selects the fixture oracle's table, otherwise the options one.
pub(super) fn corrected_digest(
    fixtures: bool,
    index: usize,
    fields: &[&[u8]; 3],
    context: &str,
    options: &ParserOptions,
    historical: Option<&[u8]>,
) -> Option<([u8; 16], u8)> {
    let table: &[Correction] = if fixtures { &FIXTURES } else { &OPTIONS };
    let correction = table.iter().find(|correction| correction.index == index)?;
    let defaults = ParserOptions::default();
    let default_lane = !options.html_tree_construction
        && options.dialect == defaults.dialect
        && options.delimiters == defaults.delimiters
        && options.whitespace == defaults.whitespace
        && !options.experimental_in_tag_comments;
    (context == correction.context
        && fields[0] == correction.source.as_bytes()
        && default_lane
        && historical == Some(repaired_digest(correction.source).as_slice()))
    .then_some((correction.after, correction.bit))
}

/// The digest of `source` parsed as a component with the HTML tree
/// construction's repairs: the shape the frozen witness recorded.
fn repaired_digest(source: &str) -> [u8; 16] {
    let allocator = Allocator::new();
    let options = ParserOptions {
        html_tree_construction: true,
        ..ParserOptions::default()
    };
    let (tree, errors) = Parser::with_options(&allocator, source, options).parse();
    let mut hash = StableHasher128::new();
    for bytes in [
        source.as_bytes(),
        cstr!("{tree:#?}").as_bytes(),
        cstr!("{errors:#?}").as_bytes(),
    ] {
        hash.update(&(bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    hash.digest()
}

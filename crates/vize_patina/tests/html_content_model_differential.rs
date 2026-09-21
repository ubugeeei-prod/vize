//! The checker's side of the parser differential (P4-11a, tier `exact`).
//!
//! Every chain of the differential universe — every pair over all tags the
//! fact table names (plus representatives of unlisted, custom and foreign
//! tags), every triple over the tags some tree-construction rule names — is
//! checked in a no-quirks `<body>` context. There the checker must decide
//! every node: an `unknown` parser verdict fails this test outright. The
//! verdicts are committed as `html_content_model/checker-verdicts.tsv` and
//! compared exactly on every run; `legacy-tools/davinci/html-content-model-oracle.mjs`
//! recomputes each one with an independent HTML parser (Chromium in CI) and
//! requires exact agreement.
//!
//! Regenerate after an intended change:
//! `VIZE_HTML_CONTENT_MODEL_WRITE=1 cargo test -p vize_patina --test html_content_model_differential`

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;

use vize_patina::html_content_model::{
    Context, Element, Family, NodeKind, Skeleton, Verdict, check, facts,
};
use vize_s0::Span;

const EXTRA: [&str; 9] = [
    "image",
    "path",
    "g",
    "mglyph",
    "malignmark",
    "foo",
    "my-el",
    "font",
    "nobr",
];

/// Never a parent: void elements, and the subtrees outside the declared
/// domain (native template contents, `noscript`, document structure).
const LEAF_ONLY: [&str; 21] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "template", "noscript", "html", "head", "body", "frameset", "#text",
];

/// Every tag some tree-construction rule names, plus representatives.
const TRIPLE: [&str; 45] = [
    "p",
    "div",
    "span",
    "a",
    "button",
    "form",
    "li",
    "ul",
    "dl",
    "dd",
    "dt",
    "h1",
    "h2",
    "table",
    "tbody",
    "tr",
    "td",
    "caption",
    "colgroup",
    "col",
    "select",
    "option",
    "optgroup",
    "hr",
    "input",
    "ruby",
    "rt",
    "rtc",
    "rb",
    "svg",
    "math",
    "foreignObject",
    "mi",
    "annotation-xml",
    "font",
    "image",
    "nobr",
    "object",
    "label",
    "textarea",
    "iframe",
    "section",
    "body",
    "path",
    "#text",
];

fn pair_universe() -> Vec<String> {
    let mut names: BTreeSet<String> = facts()
        .universe()
        .map(|(_, _, name)| name.to_string())
        .collect();
    names.extend(EXTRA.iter().map(|name| name.to_string()));
    let mut universe: Vec<String> = names.into_iter().collect();
    universe.push("#text".to_string());
    universe
}

/// The first parser divergence along the chain (1-based), 0 when faithful.
fn verdict(chain: &[&str], unknown: &mut Vec<String>) -> char {
    let mut skeleton = Skeleton::default();
    let mut opened = Vec::new();
    for (index, tag) in chain.iter().enumerate() {
        let span = Span::new(index as u32, index as u32 + 1);
        if *tag == "#text" {
            skeleton.leaf(
                NodeKind::Text {
                    whitespace_only: false,
                },
                span,
            );
        } else {
            opened.push(skeleton.open(NodeKind::Element(Element::new(tag, span)), span));
        }
    }
    for index in opened.into_iter().rev() {
        skeleton.close(index);
    }
    let report = check(&skeleton, 0, &Context::Body);
    for (index, verdict) in report.verdicts.iter().enumerate() {
        match verdict {
            Verdict::Proven { class, .. } if class.family() == Family::Parser => {
                return char::from_digit(index as u32 + 1, 10).unwrap_or('?');
            }
            Verdict::Unknown => {
                unknown.push(chain.join(" > "));
                return '?';
            }
            _ => {}
        }
    }
    '0'
}

fn render(unknown: &mut Vec<String>) -> String {
    let pairs = pair_universe();
    let mut out = String::from(
        "# Checker parser-family verdicts over the P4-11a differential universe (generated).\n\
         # Regenerate: VIZE_HTML_CONTENT_MODEL_WRITE=1 cargo test -p vize_patina --test html_content_model_differential\n\
         # Verify: node legacy-tools/davinci/html-content-model-oracle.mjs --engine chromium|parse5\n\
         # One digit per leaf of the section universe: 0 faithful, k = the k-th node diverges first.\n",
    );
    for (section, parents, leaves) in [
        ("pairs", pairs.clone(), pairs.clone()),
        (
            "triples",
            TRIPLE.iter().map(|tag| tag.to_string()).collect(),
            TRIPLE.iter().map(|tag| tag.to_string()).collect(),
        ),
    ] {
        let _ = writeln!(out, "universe\t{section}\t{}", leaves.join(" "));
        let prefixes: Vec<Vec<String>> = if section == "pairs" {
            parents.iter().map(|parent| vec![parent.clone()]).collect()
        } else {
            parents
                .iter()
                .flat_map(|a| parents.iter().map(move |b| vec![a.clone(), b.clone()]))
                .collect()
        };
        for prefix in prefixes {
            if prefix.iter().any(|tag| LEAF_ONLY.contains(&tag.as_str())) {
                continue;
            }
            let digits: String = leaves
                .iter()
                .map(|leaf| {
                    let mut chain: Vec<&str> = prefix.iter().map(String::as_str).collect();
                    chain.push(leaf);
                    verdict(&chain, unknown)
                })
                .collect();
            let _ = writeln!(out, "chain\t{section}\t{}\t{digits}", prefix.join(" "));
        }
    }
    out
}

/// Per node of `chain`: `Some(true)` proven parser divergence, `Some(false)`
/// proven parser-stable, `None` unknown or not evaluated.
fn node_verdicts(chain: &[&str], context: &Context) -> Vec<Option<bool>> {
    let mut skeleton = Skeleton::default();
    let mut opened = Vec::new();
    for (index, tag) in chain.iter().enumerate() {
        let span = Span::new(index as u32, index as u32 + 1);
        if *tag == "#text" {
            skeleton.leaf(
                NodeKind::Text {
                    whitespace_only: false,
                },
                span,
            );
        } else {
            opened.push(skeleton.open(NodeKind::Element(Element::new(tag, span)), span));
        }
    }
    for index in opened.into_iter().rev() {
        skeleton.close(index);
    }
    check(&skeleton, 0, context)
        .verdicts
        .iter()
        .map(|verdict| match verdict {
            Verdict::Proven { class, .. } => Some(class.family() == Family::Parser),
            Verdict::Refuted | Verdict::ContentUnknown => Some(false),
            Verdict::Unknown | Verdict::Skipped => None,
        })
        .collect()
}

/// Soundness of `unknown`: a verdict proven for `B > C` with the context
/// above `B` unknown must hold under every ancestor `A` of the universe that
/// the parser inserts `B` under faithfully — the property that lets a
/// per-file verdict survive cross-component composition. Checked against the
/// body-context verdicts, which the parser oracle pins exactly.
#[test]
fn truncated_verdicts_hold_in_every_faithful_context() {
    let mut violations = Vec::new();
    let (mut proven, mut refuted, mut undecided) = (0usize, 0usize, 0usize);
    for b in TRIPLE.iter().filter(|tag| !LEAF_ONLY.contains(tag)) {
        for c in TRIPLE {
            let truncated = node_verdicts(&[b, c], &Context::Truncated);
            let Some(claim) = truncated[1] else {
                undecided += 1;
                continue;
            };
            if claim {
                proven += 1
            } else {
                refuted += 1
            }
            for a in TRIPLE.iter().filter(|tag| !LEAF_ONLY.contains(tag)) {
                let body = node_verdicts(&[a, b, c], &Context::Body);
                if body[0] != Some(false) || body[1] != Some(false) {
                    continue;
                }
                if body[2] != Some(claim) {
                    violations.push(format!(
                        "{a} > {b} > {c}: truncated {claim}, body {:?}",
                        body[2]
                    ));
                }
            }
        }
    }
    assert_eq!(violations, Vec::<String>::new());
    assert_eq!((proven, refuted, undecided), (285, 595, 920));
}

#[test]
fn checker_decides_every_body_chain_and_matches_the_committed_verdicts() {
    let mut unknown = Vec::new();
    let rendered = render(&mut unknown);
    assert_eq!(
        unknown,
        Vec::<String>::new(),
        "undecided chains in a body context"
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/html_content_model/checker-verdicts.tsv");
    if std::env::var_os("VIZE_HTML_CONTENT_MODEL_WRITE").is_some() {
        std::fs::write(&path, &rendered).expect("write checker verdicts");
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, rendered,
        "checker verdicts drifted; regenerate and re-run the oracle"
    );
}

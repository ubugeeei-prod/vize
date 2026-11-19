//! The htmlish divergence ledger (`docs/davinci/plan/htmlish-divergences.json`)
//! against its oracles: every packet under `fixtures/htmlish/` holds a source and
//! the tree its dialect's oracle builds (parse5 for `document`, `@vue/compiler-dom`
//! for `vue`; `tools/commands/davinci/htmlish-oracle.ts` writes them). L1 parses
//! the same source under the dialect's profile and must build the same tree, or
//! the row must be listed below with the ledger divergence that decides it.

#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::markup::htmlish::document::{DocumentHtmlNode, ParsedDocument};
use vize_l1::markup::{SurfaceChild, parse_with_authored_diagnostics, render};

/// The ledger divergence a refused Document packet is listed under.
const REFUSAL: &str = "document-structure-refusal";

/// Every packet whose L1 tree departs from its oracle, with the ledger
/// divergence that decides it. A packet that matches must not be listed, so the
/// table shrinks as L1 closes a gap.
const DIVERGENT: &[(&str, &str)] = &[
    ("attribute-whitespace.document", REFUSAL),
    ("bogus-comments.document", REFUSAL),
    ("br-end-tag.document", REFUSAL),
    ("br-end-tag.vue", "vue-void-end-tag-reported"),
    ("comment-edges.document", REFUSAL),
    ("comment-edges.vue", "comment-bang-close"),
    ("declarations.vue", "vue-declaration-reported"),
    ("implied-end-tags.vue", "vue-implied-end-tags"),
    ("plaintext.document", REFUSAL),
    ("raw-text-legacy-elements.document", REFUSAL),
    ("script-data-escapes.document", REFUSAL),
    ("template-contents.document", REFUSAL),
];

struct Shape {
    elements: Vec<Value>,
    texts: Vec<(Value, String)>,
    comments: Vec<(Value, String)>,
    errors: bool,
}

impl Shape {
    fn json(mut self) -> Value {
        // Within a parent the order is authored order; across parents the two
        // walks differ, so compare grouped by parent.
        self.texts
            .sort_by_key(|(parent, _)| parent.as_u64().map_or(0, |index| index + 1));
        self.comments
            .sort_by_key(|(parent, _)| parent.as_u64().map_or(0, |index| index + 1));
        json!({
            "elements": self.elements,
            "texts": self.texts.into_iter().map(|(p, t)| json!([p, t])).collect::<Vec<_>>(),
            "comments": self.comments.into_iter().map(|(p, t)| json!([p, t])).collect::<Vec<_>>(),
            "errors": self.errors,
        })
    }
}

fn oracle_shape(packet: &Value) -> Value {
    let pairs = |key: &str| -> Vec<(Value, String)> {
        packet[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| (pair[0].clone(), pair[1].as_str().unwrap().to_owned()))
            .collect()
    };
    Shape {
        elements: packet["elements"].as_array().unwrap().clone(),
        texts: pairs("texts"),
        comments: pairs("comments"),
        errors: !packet["errors"].as_array().unwrap().is_empty(),
    }
    .json()
}

fn vue_shape(source: &str) -> Value {
    let allocator = Allocator::default();
    // The product entry: the repaired tree plus the complete diagnostic vector.
    let (tree, _, errors) = parse_with_authored_diagnostics(&allocator, source);
    let mut rendered = String::new();
    render(&tree, &mut |piece| rendered.push_str(piece));
    assert_eq!(rendered, source, "render(tree) == source");
    let mut shape = Shape {
        elements: Vec::new(),
        texts: Vec::new(),
        comments: Vec::new(),
        errors: !errors.is_empty(),
    };
    walk_vue(&tree.children, None, &mut shape);
    shape.json()
}

/// The Vue tree of a surface: elements, text, interpolations and comments.
/// Holes and processing instructions build no Vue node, so text on both sides
/// of one joins, as the oracle's text does.
fn walk_vue(children: &[SurfaceChild<'_>], parent: Option<usize>, shape: &mut Shape) {
    let parent_json = || parent.map_or(Value::Null, |index| json!(index));
    let mut open_text: Option<usize> = None;
    for child in children {
        match child {
            SurfaceChild::Element(element) => {
                open_text = None;
                let name = element.open.lt_name.text.trim_start_matches('<');
                let attributes: Vec<Value> = element
                    .open
                    .attrs
                    .iter()
                    .map(|attribute| {
                        json!([
                            attribute.name.text,
                            attribute.value.as_ref().map(|value| value.content.text)
                        ])
                    })
                    .collect();
                shape
                    .elements
                    .push(json!([name, parent_json(), attributes]));
                let index = shape.elements.len() - 1;
                walk_vue(&element.children, Some(index), shape);
            }
            SurfaceChild::Text(text) => match open_text {
                Some(at) => shape.texts[at].1.push_str(text.text),
                None => {
                    open_text = Some(shape.texts.len());
                    shape.texts.push((parent_json(), text.text.to_owned()));
                }
            },
            SurfaceChild::Interpolation(interpolation) => {
                open_text = None;
                shape.texts.push((
                    parent_json(),
                    [
                        interpolation.open.text,
                        interpolation.content.text,
                        interpolation.close.text,
                    ]
                    .concat(),
                ));
            }
            SurfaceChild::Comment(comment) => {
                open_text = None;
                let body = comment.text.strip_prefix("<!--").unwrap_or(comment.text);
                let body = ["-->", "--!>", "->", ">"]
                    .iter()
                    .find_map(|closer| body.strip_suffix(closer))
                    .unwrap_or(body);
                shape.comments.push((parent_json(), body.to_owned()));
            }
            SurfaceChild::ProcessingInstruction(_)
            | SurfaceChild::Cdata(_)
            | SurfaceChild::Unexpected(_) => {}
        }
    }
}

fn document_shape(source: &str) -> Result<Value, String> {
    let allocator = Allocator::default();
    let owner = ParsedDocument::lex_in(&allocator, SourceRoot::new(source).unwrap());
    let tree = owner
        .html_structure()
        .map_err(|refusal| format!("{refusal:?}"))?;
    let elements: Vec<_> = tree.elements().collect();
    let index_of = |span| {
        elements
            .iter()
            .position(|element| element.opening_span() == span)
    };
    let mut shape = Shape {
        elements: Vec::new(),
        texts: Vec::new(),
        comments: Vec::new(),
        errors: false,
    };
    for (index, element) in elements.iter().enumerate() {
        let parent = element
            .parent()
            .and_then(|parent| index_of(parent.opening_span()))
            .map_or(Value::Null, |parent| json!(parent));
        let attributes: Vec<Value> = element
            .attributes()
            .map(|attribute| {
                json!([
                    attribute.name_characters().collect::<String>(),
                    attribute.value_characters().collect::<String>()
                ])
            })
            .collect();
        shape
            .elements
            .push(json!([element.name(), parent, attributes]));
        for node in element
            .child_nodes()
            .map_err(|refusal| format!("{refusal:?}"))?
        {
            match node.map_err(|refusal| format!("{refusal:?}"))? {
                DocumentHtmlNode::Element(_) => {}
                DocumentHtmlNode::Text(text) => {
                    shape
                        .texts
                        .push((json!(index), text.characters().collect()));
                }
                DocumentHtmlNode::Comment(comment) => {
                    shape
                        .comments
                        .push((json!(index), comment.characters().collect()));
                }
            }
        }
    }
    Ok(shape.json())
}

fn packets() -> Vec<(String, Value)> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/htmlish");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if name == "sources.json" || !name.ends_with(".json") {
            continue;
        }
        let packet: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        out.push((name.trim_end_matches(".json").to_owned(), packet));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn every_htmlish_packet_matches_its_oracle_or_its_ledger_divergence() {
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/davinci/plan/htmlish-divergences.json"),
        )
        .unwrap(),
    )
    .unwrap();
    // Every ledger divergence but the hole policy (`error-recovery`, which
    // `surface_fidelity.rs` holds) decides at least one packet here.
    let decided: BTreeSet<&str> = ledger["divergences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|divergence| divergence["id"].as_str().unwrap())
        .filter(|id| *id != "error-recovery")
        .collect();
    let divergent: BTreeMap<&str, &str> = DIVERGENT.iter().copied().collect();
    let used: BTreeSet<&str> = divergent.values().copied().collect();
    assert_eq!(used, decided);

    let mut problems = Vec::new();
    let mut seen = 0;
    for (name, packet) in packets() {
        let source = packet["source"].as_str().unwrap();
        let expected = oracle_shape(&packet);
        let actual = match packet["dialect"].as_str().unwrap() {
            "vue" => Ok(vue_shape(source)),
            "document" => document_shape(source),
            dialect => panic!("unknown dialect {dialect}"),
        };
        let listed = divergent.get(name.as_str()).copied();
        seen += usize::from(listed.is_some());
        match (actual, listed) {
            (Ok(actual), None) if actual == expected => {}
            (Ok(actual), Some(_)) if actual == expected => {
                problems.push(format!(
                    "{name} now matches its oracle: drop it from DIVERGENT"
                ));
            }
            (Ok(_), Some(id)) if id != REFUSAL => {}
            (Ok(actual), _) => {
                problems.push(format!("{name}\n  vize:   {actual}\n  oracle: {expected}"));
            }
            (Err(_), Some(REFUSAL)) => {}
            (Err(refusal), _) => problems.push(format!("{name}: refused {refusal}")),
        }
    }
    assert_eq!(
        seen,
        DIVERGENT.len(),
        "DIVERGENT names a packet that does not exist"
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Each comment token covers its own closer: `<!--->` and `--!>` once left
/// their tail to the next node.
#[test]
fn comment_tokens_cover_their_closers() {
    let allocator = Allocator::default();
    let (tree, _, _) = parse_with_authored_diagnostics(&allocator, "<!--><!---><!--a--!>b");
    let tokens: Vec<(&str, &str)> = tree
        .children
        .iter()
        .map(|child| match child {
            SurfaceChild::Comment(token) => ("comment", token.text),
            SurfaceChild::Text(token) => ("text", token.text),
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        tokens,
        [
            ("comment", "<!-->"),
            ("comment", "<!--->"),
            ("comment", "<!--a--!>"),
            ("text", "b"),
        ]
    );
}

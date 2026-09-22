//! The expression-world probe: one batch, its expected facts and projection,
//! and the committed goldens the expression echo guest replays.

use std::path::{Path, PathBuf};

use vize_davinci::fact::{AlphaDocument, ExpressionFact, ExpressionFacts, FactTable};
use vize_davinci::folio::{Folio, FolioMode};
use vize_extension_host::Span;
use vize_extension_host::expression::{
    Binding, Expression, ExpressionBatch, ProjectionPage, ProjectionRow, Range,
};
use vize_s0::String;

/// Set to rewrite the committed expression goldens from [`facts`] and
/// [`projection`].
pub const BLESS_ENV: &str = "VIZE_EXTENSION_GOLDEN_BLESS";

pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expression")
}

/// `<p :title="msg + suffix">{{ count * 2 }}</p>`, block at file offset 100.
pub fn batch() -> ExpressionBatch {
    let binding = |name: &str, kind: &str| Binding {
        name: String::from(name),
        kind: String::from(kind),
    };
    let expression = |id, source: &str, start| Expression {
        id,
        source: String::from(source),
        span: Span {
            start,
            end: start + source.len() as u32,
        },
    };
    ExpressionBatch {
        environment: vec![
            binding("count", "setup-ref"),
            binding("msg", "setup-ref"),
            binding("suffix", "setup-const"),
        ],
        expressions: vec![
            expression(0, "msg + suffix", 110),
            expression(1, "count * 2", 126),
        ],
    }
}

/// What a dialect answers about [`batch`]: exact references, nothing
/// constant.
pub fn facts() -> AlphaDocument<ExpressionFacts> {
    let fact = |references: &str| ExpressionFact {
        references: String::from(references),
        exact: true,
        constant: false,
    };
    let table: FactTable<ExpressionFacts> = [(0u32, fact("msg,suffix")), (1, fact("count"))]
        .into_iter()
        .collect();
    AlphaDocument::export(&table)
}

fn range(start: u32, end: u32) -> Range {
    Range { start, end }
}

/// The projection of [`batch`] into checkable statements, with span links.
pub fn projection() -> ProjectionPage {
    ProjectionPage {
        rows: vec![
            ProjectionRow {
                generated: range(1, 13),
                authored: range(110, 122),
                kind: 3,
                features: u8::MAX,
                sub_spans: vec![
                    (range(1, 4), range(110, 113)),
                    (range(7, 13), range(116, 122)),
                ],
            },
            ProjectionRow {
                generated: range(17, 26),
                authored: range(126, 135),
                kind: 2,
                features: 0b0100_1001,
                sub_spans: vec![(range(17, 22), range(126, 131))],
            },
        ],
        text: String::from("(msg + suffix);\n(count * 2);\n"),
    }
}

/// A page's canonical text.
pub fn text<T: Folio>(page: &T) -> String {
    let mut out = String::default();
    page.print(&mut out, FolioMode::Full)
        .expect("printing into a String cannot fail");
    out
}

/// The committed `(facts, projection)` goldens.
pub fn committed() -> [String; 2] {
    ["probe.facts.folio", "probe.projection.folio"].map(|name| {
        let bytes = std::fs::read(golden_dir().join(name)).expect("the committed golden");
        String::from_utf8(bytes).expect("UTF-8")
    })
}

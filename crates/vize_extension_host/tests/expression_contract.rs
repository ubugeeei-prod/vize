//! The expression-dialect world's pages and acceptance (P6-1b), exact
//! oracles only: the committed goldens are the canonical pages of the probe,
//! the projection rows convert one to one into the P4-5a
//! `ProjectionMapping`, and every refusal carries its exact message.

mod support;

use support::expression::{BLESS_ENV, batch, committed, facts, golden_dir, projection, text};
use vize_canon::virtual_ts::{
    ProjectionFeatures, ProjectionMapping, ProjectionMeta, ProjectionSpanKind, VizeMapping,
    VizeSubSpan,
};
use vize_davinci::folio::{Folio, FolioError};
use vize_extension_host::expression::{Analysis, ProjectionPage, accept_analysis};
use vize_extension_host::{Diagnostic, Page, Severity, Span, Stage};
use vize_s0::{String, cstr};

fn analysis() -> Analysis {
    Analysis {
        facts: Page {
            schema_version: 1,
            text: text(&facts()),
        },
        projection: Page {
            schema_version: 1,
            text: text(&projection()),
        },
        diagnostics: Vec::new(),
    }
}

#[test]
fn committed_goldens_are_the_canonical_probe_pages() {
    let produced = [text(&facts()), text(&projection())];
    if std::env::var_os(BLESS_ENV).is_some() {
        for (name, page) in ["probe.facts.folio", "probe.projection.folio"]
            .iter()
            .zip(&produced)
        {
            std::fs::write(golden_dir().join(name), page.as_bytes()).expect("writes");
        }
    }
    assert_eq!(committed(), produced);
    let parsed = ProjectionPage::parse(&produced[1]).expect("the golden parses");
    assert_eq!(parsed, projection());
    assert_eq!(text(&parsed), produced[1]);
}

#[test]
fn the_probe_analysis_is_accepted() {
    let accepted = accept_analysis(&batch(), analysis()).expect("the probe is accepted");
    assert_eq!(accepted.facts, facts());
    assert_eq!(accepted.projection, projection());
}

const KINDS: [ProjectionSpanKind; 10] = [
    ProjectionSpanKind::Unknown,
    ProjectionSpanKind::Script,
    ProjectionSpanKind::Interpolation,
    ProjectionSpanKind::DirectiveExpr,
    ProjectionSpanKind::DirectiveArg,
    ProjectionSpanKind::EventHandler,
    ProjectionSpanKind::VForVar,
    ProjectionSpanKind::SlotBinding,
    ProjectionSpanKind::ComponentRef,
    ProjectionSpanKind::TemplateExpression,
];

#[test]
fn projection_rows_are_the_p4_5a_mapping_rows() {
    let named: Vec<_> = ProjectionFeatures::NAMED
        .iter()
        .map(|(_, name)| name.replace('_', "-"))
        .collect();
    assert_eq!(named, vize_extension_host::expression::projection::FEATURES);
    let page = projection();
    let mut mapping = ProjectionMapping::new();
    for row in &page.rows {
        let features = ProjectionFeatures::NAMED
            .iter()
            .enumerate()
            .filter(|(bit, _)| row.features & (1 << bit) != 0)
            .fold(ProjectionFeatures::NONE, |all, (_, (flag, _))| {
                all.union(*flag)
            });
        let range = |r: vize_extension_host::expression::Range| r.start as usize..r.end as usize;
        let mut span = VizeMapping::new(range(row.generated), range(row.authored));
        span.sub_spans = row
            .sub_spans
            .iter()
            .map(|(generated, authored)| VizeSubSpan {
                gen_range: range(*generated),
                src_range: range(*authored),
            })
            .collect();
        let meta = ProjectionMeta {
            features,
            kind: KINDS[usize::from(row.kind)],
        };
        mapping.push_with(span, meta);
    }
    assert_eq!(mapping.to_authored(1), Some(110));
    assert_eq!(mapping.to_authored(18), Some(127));
    assert_eq!(mapping.to_generated(116), Some(7));
    assert_eq!(
        mapping.meta(0),
        ProjectionMeta::of_kind(ProjectionSpanKind::DirectiveExpr)
    );
    assert_eq!(mapping.meta(1).kind, ProjectionSpanKind::Interpolation);
    assert_eq!(
        mapping.meta(1).features,
        ProjectionFeatures::HOVER
            .union(ProjectionFeatures::DEFINITION)
            .union(ProjectionFeatures::DIAGNOSTICS)
    );
    assert_eq!(mapping.spans()[0].sub_spans.len(), 2);
}

/// One edit to the probe analysis.
type Tamper = fn(&mut Analysis);

fn refused(tamper: Tamper) -> String {
    let mut analysis = analysis();
    tamper(&mut analysis);
    let error = accept_analysis(&batch(), analysis).expect_err("the analysis must be refused");
    cstr!("{error}")
}

fn replace(page: &mut Page, from: &str, to: &str) {
    page.text = page.text.replace(from, to).into();
}

#[test]
fn analyses_are_refused_exactly() {
    let cases: [(Tamper, &str); 10] = [
        (
            |a| a.facts.schema_version = 2,
            "facts-page schema version 2 is unreadable: this host reads version 1",
        ),
        (
            |a| replace(&mut a.facts, "group=expression-facts", "group=bindings"),
            "facts-page: folio parse error at line 2: expected `group=expression-facts`, found `group=bindings`",
        ),
        (
            |a| replace(&mut a.facts, "1=count\n", ""),
            "facts-page names expressions [0], the batch has [0, 1]",
        ),
        (
            |a| replace(&mut a.facts, "1=count\n", "1=count,total\n"),
            "expression 1 exactly references \"total\", which is not in the environment",
        ),
        (
            |a| replace(&mut a.projection, "row 17:26", "row 17:40"),
            "projection row 1 leaves the generated text",
        ),
        (
            |a| {
                replace(
                    &mut a.projection,
                    "126:135 interpolation",
                    "126:136 interpolation",
                )
            },
            "projection row 1 links outside every expression",
        ),
        (
            |a| replace(&mut a.projection, "sub 17:22", "sub 16:22"),
            "projection row 1 has a sub-span outside the row",
        ),
        (
            |a| {
                replace(&mut a.projection, "(count * 2);", "(éount * 2);");
                replace(&mut a.projection, "row 17:26", "row 17:27");
                replace(&mut a.projection, "sub 17:22", "sub 18:23");
            },
            "projection row 1 has a sub-span off a character boundary",
        ),
        (
            |a| replace(&mut a.projection, "rows=2", "rows=02"),
            "projection-page is not canonical: it differs from its reprint at byte 18",
        ),
        (
            |a| a.diagnostics.push(diagnostic(122, 126)),
            "diagnostic 0 span 122:126 lies outside every expression",
        ),
    ];
    for (tamper, message) in cases {
        assert_eq!(refused(tamper), message);
    }
}

fn diagnostic(start: u32, end: u32) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        stage: Stage::Semantic,
        span: Span { start, end },
        message: String::from("probe"),
        parts: Vec::new(),
        witness: None,
    }
}

#[test]
fn malformed_projection_pages_are_refused_exactly() {
    let cases: [(&str, usize, &str); 7] = [
        (
            "[projection]\nrows=0\n",
            0,
            "missing section [projection.text]",
        ),
        (
            "rows=0\n[projection.text]\n",
            1,
            "first section must be [projection]",
        ),
        (
            "[projection]\nrows=x\n[projection.text]\n",
            2,
            "expected `rows=<count>`, found `rows=x`",
        ),
        (
            "[projection]\nrows=1\n[projection.rows]\n  sub 0:1 0:1\n[projection.text]\n",
            4,
            "`sub` before any `row`",
        ),
        (
            "[projection]\nrows=1\n[projection.rows]\nrow 0:1 0:1 widget all\n[projection.text]\n",
            4,
            "unknown kind `widget`",
        ),
        (
            "[projection]\nrows=1\n[projection.rows]\nrow 0:1 0:1 script hover,zoom\n[projection.text]\n",
            4,
            "unknown feature `zoom`",
        ),
        (
            "[projection]\nrows=1\n[projection.rows]\nrow 2:1 0:1 script all\n[projection.text]\n",
            4,
            "invalid range `2:1`",
        ),
    ];
    for (text, line, message) in cases {
        assert_eq!(
            ProjectionPage::parse(text),
            Err(FolioError::new(line, String::from(message))),
            "{text:?}"
        );
    }
}

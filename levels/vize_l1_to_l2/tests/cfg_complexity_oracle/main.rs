//! TS-34 for the template-complexity fact group (Davinci P4-9a): the
//! production pass (`vize_l1_to_l2::pass::cfg`) against the naive
//! evaluator (`spec.rs`), exact agreement on totals **and** on every
//! breakdown row, over
//!
//! - hand-computed pins, one per rule of `complexity-metrics.md`, where
//!   both implementations must also equal the number worked out by hand;
//! - the committed construct-matrix plane
//!   (`tests/fixtures/davinci-matrix/`, TS-12 keeps it fresh), census
//!   pinned;
//! - a corpus shard via `VIZE_DAVINCI_COMPLEXITY_CORPUS=<dir>[,<dir>…]`
//!   (`tests/tooling/davinci-complexity-corpus.test.ts` runs the two
//!   test-scripts submodules in CI), which also prints the distribution
//!   table the metric spec records.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests assert by panicking"
)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

mod ast;
mod graph;
mod runner;
mod spec;

use std::path::{Path, PathBuf};

use runner::{Report, both_template, collect_vue_files, distribution_table, percentile, run_files};

/// `(cyclomatic, cognitive, unknown, max_nesting)`.
type Totals = (u32, u32, u32, u32);

/// The totals of `template`, asserted identical across both
/// implementations first.
fn totals(template: &str) -> Totals {
    let (production, naive) = both_template(template);
    assert_eq!(
        production, naive,
        "implementations disagree on {template:?}"
    );
    (
        production.cyclomatic,
        production.cognitive,
        production.unknown,
        production.max_nesting,
    )
}

#[test]
fn every_rule_matches_its_hand_computed_value() {
    let cases: &[(&str, Totals)] = &[
        // Empty: one path, nothing to understand.
        ("", (1, 0, 0, 0)),
        // `if`: one decision, one structure.
        (r#"<p v-if="a">x</p>"#, (2, 1, 0, 1)),
        // `else`: no decision, a flat +1.
        (r#"<p v-if="a">x</p><p v-else>y</p>"#, (2, 2, 0, 1)),
        // `else-if`: a decision each, flat +1 each.
        (
            r#"<p v-if="a"/><p v-else-if="b"/><p v-else/>"#,
            (3, 3, 0, 1),
        ),
        // Without `v-else` the last condition still decides.
        (r#"<p v-if="a"/><p v-else-if="b"/>"#, (3, 2, 0, 1)),
        // `for`: one decision, one structure.
        (r#"<li v-for="x in xs">{{ x }}</li>"#, (2, 1, 0, 1)),
        // Nesting: if(1+0) + for(1+1) + if(1+2).
        (
            r#"<div v-if="a"><li v-for="x in xs"><b v-if="x.ok"/></li></div>"#,
            (4, 6, 0, 3),
        ),
        // `logical`: one decision per operator, +1 per run of like ones.
        ("{{ a && b && c }}", (3, 1, 0, 0)),
        ("{{ a && b || c && d }}", (4, 3, 0, 0)),
        ("{{ (a || b) ?? c }}", (3, 2, 0, 0)),
        // `conditional`: +1 + nesting, and a `?:` nests the next one.
        ("{{ a ? b : c ? d : e }}", (3, 3, 0, 0)),
        // `scoped-slot`: no increment, one level deeper for its body.
        (r#"<C v-slot="{ x }"><b v-if="x"/></C>"#, (2, 2, 0, 2)),
        // A named slot without params is not scoped.
        (
            r#"<C><template #h><b v-if="x"/></template></C>"#,
            (2, 1, 0, 1),
        ),
        // Handlers are evaluated code: their operators count.
        (r#"<b @click="ok && go()"/>"#, (2, 1, 0, 0)),
        // `unknown`: a multi-statement handler has no retained AST.
        (r#"<b @click="a++; b++"/>"#, (1, 0, 1, 0)),
        // A `v-for` source evaluates outside the loop it drives.
        (r#"<i v-for="x in (a ? xs : ys)"/>"#, (3, 2, 0, 1)),
        // A `v-model` write side is the read's text: counted once.
        (r#"<input v-model="a ? b : c"/>"#, (2, 1, 0, 0)),
    ];
    for (template, expected) in cases {
        assert_eq!(totals(template), *expected, "{template:?}");
    }
}

#[test]
fn an_operator_tree_inside_a_leaf_is_its_own_tree() {
    // `f(b || c)` ends the outer tree: two trees, two runs, and the inner
    // row sits at its own span.
    let (production, naive) = both_template("{{ a && f(b || c) }}");
    assert_eq!(production, naive);
    let logical: Vec<(u32, u32, u32, u32)> = production
        .rows
        .iter()
        .filter(|row| row.kind == "logical")
        .map(|row| (row.start, row.end, row.cyclomatic, row.cognitive))
        .collect();
    assert_eq!(logical, vec![(3, 17, 1, 1), (10, 16, 1, 1)]);
}

#[test]
fn parenthesized_mixed_operator_trees_keep_all_independent_rows() {
    for (template, expected) in [
        ("{{ a && (b || (c && d)) }}", (4, 3, 0, 0)),
        ("{{ ((a || b) && (c || d)) ?? (e && f) }}", (6, 5, 0, 0)),
        ("{{ a && ((b ? (c || d) : (e && f))) }}", (5, 4, 0, 0)),
    ] {
        assert_eq!(totals(template), expected, "{template:?}");
    }
}

fn nested_logical_run(depth: usize, right_nested: bool) -> String {
    let mut expression = String::from("leaf");
    for _ in 0..depth {
        expression = if right_nested {
            format!("value && ({expression})")
        } else {
            format!("({expression}) && value")
        };
    }
    expression
}

#[test]
fn deep_left_and_right_logical_runs_keep_full_naive_agreement() {
    use vize_l0::expression_guard::{MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse};

    for right_nested in [false, true] {
        let expression = nested_logical_run(MAX_EXPRESSION_NESTING_DEPTH, right_nested);
        // Successful retained admission is required before this can prove AST scoring.
        assert!(expression_is_safe_to_parse(&expression));
        let arena = vize_l0::Allocator::new();
        vize_l2::expr::JsExpr::parse_in(
            &arena,
            &expression,
            vize_l0::Span::new(0, u32::try_from(expression.len()).expect("bounded fixture")),
        )
        .expect("the actual guard-boundary tree retains a real AST");
        let template = format!("{{{{ {expression} }}}}");
        let (production, naive) = both_template(&template);
        assert_eq!(production, naive, "right_nested={right_nested}");
        assert_eq!(
            production.cyclomatic,
            u32::try_from(MAX_EXPRESSION_NESTING_DEPTH).expect("guard fits u32") + 1,
        );
        assert_eq!(production.cognitive, 1);
        assert_eq!(production.unknown, 0);
        assert_eq!(production.rows.len(), 1);
    }
}

#[test]
fn past_guard_logical_runs_never_invent_retained_ast_facts() {
    use vize_l0::expression_guard::expression_is_safe_to_parse;

    // Preserve the exact 64-depth failed fixture as a genuine refusal control.
    for right_nested in [false, true] {
        let expression = nested_logical_run(64, right_nested);
        assert!(!expression_is_safe_to_parse(&expression));
        let arena = vize_l0::Allocator::new();
        assert!(matches!(
            vize_l2::expr::JsExpr::parse_in(
                &arena,
                &expression,
                vize_l0::Span::new(0, u32::try_from(expression.len()).expect("bounded fixture")),
            ),
            Err(vize_l2::expr::OpaqueReason::NestingRefused),
        ));
        let template = format!("{{{{ {expression} }}}}");
        let (production, naive) = both_template(&template);
        assert_eq!(production, naive, "right_nested={right_nested}");
        assert_eq!(production.unknown, 1);
        assert_eq!(production.rows.len(), 1);
        assert_eq!(
            production.rows.first().expect("actual refusal row").kind,
            "unknown"
        );
    }
}

fn matrix_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/davinci-matrix")
}

#[test]
fn the_matrix_plane_agrees_with_the_naive_evaluator() {
    let mut files = Vec::new();
    collect_vue_files(&matrix_dir(), &mut files);
    let report = run_files(&files);
    eprintln!("{}", report.scope_line("matrix plane"));
    report
        .verdict("matrix plane")
        .unwrap_or_else(|failure| panic!("{failure}"));
    // The census, pinned: a regenerated plane that moves any of these is
    // a deliberate re-pin (see the P4-9a record).
    assert_eq!(
        (report.files, report.templates, report.rows, report.unknown),
        (90, 90, 36, 0),
        "matrix census moved"
    );
}

#[test]
fn the_corpus_shard_agrees_with_the_naive_evaluator() {
    let Some(roots) = std::env::var_os("VIZE_DAVINCI_COMPLEXITY_CORPUS") else {
        eprintln!("VIZE_DAVINCI_COMPLEXITY_CORPUS unset: matrix plane only");
        return;
    };
    let mut files = Vec::new();
    for root in roots.to_string_lossy().split(',') {
        let root = PathBuf::from(root);
        // Cargo runs test binaries from the package directory; let
        // workspace-root-relative paths work too (the P2-8 lane's rule).
        let root = if root.is_relative() && !root.is_dir() {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(&root)
        } else {
            root
        };
        assert!(
            root.is_dir(),
            "VIZE_DAVINCI_COMPLEXITY_CORPUS must name directories: {}",
            root.display()
        );
        collect_vue_files(&root, &mut files);
    }
    assert!(!files.is_empty(), "corpus shard found no .vue files");
    let report = run_files(&files);
    eprintln!("{}", report.scope_line("corpus shard"));
    eprintln!("{}", distribution_table(&report));
    report
        .verdict("corpus shard")
        .unwrap_or_else(|failure| panic!("{failure}"));
}

#[test]
fn a_degenerate_run_fails() {
    let empty = Report::default();
    assert_eq!(
        empty.verdict("empty"),
        Err(String::from(
            "complexity empty: zero templates were scored — the run proves nothing \
             (0 files); a degenerated suite must fail, not pass"
        ))
    );
}

#[test]
fn percentiles_are_nearest_rank() {
    let sample: Vec<u32> = (1..=20).collect();
    assert_eq!(percentile(&sample, 50), 10);
    assert_eq!(percentile(&sample, 95), 19);
    assert_eq!(percentile(&sample, 99), 20);
    assert_eq!(percentile(&[7], 95), 7);
    assert_eq!(percentile(&[], 95), 0);
}

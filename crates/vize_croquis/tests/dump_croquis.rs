//! Historical Croquis dump fixtures and pipeline grammar, through typed APIs.
//!
//! The retired host bound arbitrary pass names to no-op bodies. These tests
//! preserve the canonical artifact and syntax laws without presenting those
//! names as production compiler passes.

#![expect(clippy::expect_used, reason = "committed fixtures are test oracles")]

use std::path::{Path, PathBuf};

use vize_croquis::dump::Page as CroquisPage;
use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::pass::{parse_pipelines, print_pipelines};

fn committed_pages() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/croquis");
    let mut pages: Vec<_> = std::fs::read_dir(root)
        .expect("fixture directory exists")
        .map(|entry| entry.expect("readable fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "folio"))
        .collect();
    pages.sort();
    pages
}

#[test]
fn fourteen_committed_croquis_pages_remain_byte_canonical() {
    let pages = committed_pages();
    assert_eq!(pages.len(), 14);
    for path in pages {
        let original = std::fs::read_to_string(&path).expect("committed page reads");
        let page = CroquisPage::parse(&original).expect("committed page parses");
        assert_eq!(page.print_to_string(DumpMode::Full).as_str(), original);
    }
}

#[test]
fn noncanonical_page_prints_one_deterministic_canonical_form() {
    let input = "[vir]\nscript_setup=false\nscopes=0\nbindings=1\n\n[bindings]\nc:b,a\n\n";
    let printed = CroquisPage::parse(input)
        .expect("noncanonical page parses")
        .print_to_string(DumpMode::Full);
    assert_ne!(printed.as_str(), input);
    assert_eq!(
        CroquisPage::parse(printed.as_str())
            .expect("canonical page reparses")
            .print_to_string(DumpMode::Full),
        printed
    );
}

#[test]
fn malformed_page_reports_the_exact_parse_error() {
    let error = CroquisPage::parse("x\n").expect_err("content before [vir] is invalid");
    assert_eq!(
        error.to_string(),
        "folio parse error at line 1: content before the [vir] header"
    );
}

#[test]
fn historical_pipeline_text_is_only_a_grammar_contract() {
    for syntax in ["l2(alpha,beta)", "l2(alpha),l2-to-l3(beta,gamma)", "l2()"] {
        let parsed = parse_pipelines(syntax).expect("canonical grammar parses");
        assert_eq!(print_pipelines(&parsed).as_str(), syntax);
    }
    for (syntax, message) in [
        ("", "empty pipeline string at offset 0"),
        ("s2", "expected `(` after stage name at offset 2"),
        ("s2(a", "unterminated pass list, expected `)` at offset 4"),
        ("L2(a)", "unexpected character `L` at offset 0"),
        ("s2-(a)", "identifier must not end with `-` at offset 2"),
    ] {
        assert_eq!(
            parse_pipelines(syntax)
                .expect_err("malformed grammar must fail")
                .to_string(),
            message
        );
    }
}

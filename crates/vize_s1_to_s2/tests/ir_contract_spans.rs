//! P2-17 mechanical span gate: the owned S2 folio carries no span that
//! cannot be resolved against the authored source.

mod support;

use std::fs;

use davinci_test_support::surface_fixture as battery;
use support::{assert_folio_spans_resolve, with_lowered, with_transformed};
use vize_s0::{Allocator, SourceRoot, Span, String};
use vize_s2::folio::{DisegnoFolio, FolioExpr, FolioInterpolation, FolioOp};

#[test]
fn owned_folio_spans_resolve_for_committed_battery() {
    for fixture in battery::WELL_FORMED.iter().chain(battery::MALFORMED) {
        with_lowered(fixture.source, |_lowered, folio| {
            assert_folio_spans_resolve(fixture.source, folio, fixture.name);
        });
        with_transformed(fixture.source, |_lowered, folio, _facts, _budget| {
            assert_folio_spans_resolve(fixture.source, folio, fixture.name);
        });
    }
}

#[test]
fn owned_folio_spans_resolve_for_optional_corpus() {
    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    assert!(
        !sweep.files.is_empty(),
        "corpus sweep found no .vue files under {}",
        sweep.root.display()
    );
    let mut checked = 0usize;
    for file in &sweep.files {
        let source = fs::read_to_string(file).unwrap_or_else(|error| {
            panic!("failed to read corpus file {}: {error}", file.display())
        });
        let context = file.to_string_lossy();
        with_lowered(&source, |_lowered, folio| {
            assert_folio_spans_resolve(&source, folio, context.as_ref());
        });
        checked += 1;
    }
    assert_eq!(
        checked,
        sweep.files.len(),
        "corpus sweep did not check every .vue file under {}",
        sweep.root.display()
    );
    eprintln!(
        "davinci owned folio span corpus sweep: files={} checked={}",
        sweep.files.len(),
        checked
    );
}

#[test]
fn owned_folio_expression_sources_resolve_through_sfc_block_offsets() {
    let source = "é prefix\n<template><p :id=\"item.id\">{{ item.name }}</p></template>";
    let template_start = source.find("<p").expect("template body starts");
    let template_end = source.find("</template>").expect("template closes");
    let template = &source[template_start..template_end];
    let root = SourceRoot::new(source).expect("source is small");
    let block = root
        .block(template, template_start as u32)
        .expect("template block is an authored slice");

    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, template);
    let lowered = vize_s1_to_s2::lower_source_block(&allocator, &tree, &errors, block);
    let folio = DisegnoFolio::of(&lowered.root.ops);

    assert_folio_spans_resolve(source, &folio, "sfc-block-expression-slices");
}

#[test]
#[should_panic(expected = "expression source is not authored")]
fn owned_folio_rejects_same_length_expression_source_mismatches() {
    let folio = DisegnoFolio {
        ops: vec![FolioOp::Interpolation(FolioInterpolation {
            expression: FolioExpr::Js {
                source: String::from("y"),
                span: Span::new(3, 4),
            },
            span: Span::new(0, 7),
        })],
    };

    assert_folio_spans_resolve("{{ x }}", &folio, "corrupt-expression-source");
}

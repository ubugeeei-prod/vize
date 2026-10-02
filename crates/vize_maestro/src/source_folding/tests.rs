use super::{SourceFoldingRefusal, program_comment_ranges};
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};
use vize_l0::{Allocator, Span};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape,
    syntax::{ProgramGoal, ProgramOptions, parse_once, parse_program_once},
};

fn expected(start: u32, end: u32) -> FoldingRange {
    FoldingRange {
        start_line: start,
        start_character: None,
        end_line: end,
        end_character: None,
        kind: Some(FoldingRangeKind::Comment),
        collapsed_text: None,
    }
}

#[test]
fn actual_program_profiles_retain_complete_ordered_comment_ranges() {
    for (lang, jsx, statement) in [
        (Lang::Js, false, "const value = 1;"),
        (Lang::Ts, false, "const value: number = 1;"),
        (Lang::Js, true, "const value = <div />;"),
        (Lang::Ts, true, "const value: JSX.Element = <div />;"),
    ] {
        for goal in [ProgramGoal::Module, ProgramGoal::Script] {
            let arena = Allocator::default();
            let source = format!("/*\n first\n*/\n{statement}\n/*\n second\n*/\n");
            let input = EmbedSource::authored(&source, Span::new(0, source.len() as u32))
                .expect("original whole-file source");
            let parsed = parse_program_once(&arena, input, ProgramOptions { lang, jsx, goal });
            assert!(parsed.admitted_program().is_some(), "{parsed:?}");
            assert_eq!(
                program_comment_ranges(&parsed),
                Ok(vec![expected(0, 1), expected(4, 5)]),
                "{lang:?}/{jsx}/{goal:?}"
            );
        }
    }
}

#[test]
fn lsp_cr_lf_crlf_coordinates_keep_astral_and_unicode_separators_authored() {
    for newline in ["\r", "\n", "\r\n"] {
        let arena = Allocator::default();
        let source = format!("/* 😀\u{2028}\u{2029}{newline} 日本語{newline}*/");
        let input = EmbedSource::authored(&source, Span::new(0, source.len() as u32))
            .expect("original mixed-line source");
        let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
        assert_eq!(program_comment_ranges(&parsed), Ok(vec![expected(0, 1)]));
    }
}

#[test]
fn comment_shaped_literals_and_empty_regions_do_not_invent_ranges() {
    let arena = Allocator::default();
    let source = "const x = '/* fake */';\n// line\n/* one line */\n/*\n*/\n/*\n interior\n*/";
    let input = EmbedSource::authored(source, Span::new(0, source.len() as u32))
        .expect("original literals and real comments");
    let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
    assert_eq!(program_comment_ranges(&parsed), Ok(vec![expected(5, 6)]));
}

#[test]
fn retained_comments_on_syntax_failure_cannot_supply_admission() {
    let arena = Allocator::default();
    let source = "/*\n retained\n*/ const = ;";
    let input = EmbedSource::authored(source, Span::new(0, source.len() as u32))
        .expect("original rejected program");
    let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
    assert!(parsed.comments().next().is_some());
    assert_eq!(
        program_comment_ranges(&parsed),
        Err(SourceFoldingRefusal::UnadmittedProgram)
    );
}

#[test]
fn authentic_selected_script_is_refused_without_its_full_file_bridge() {
    let arena = Allocator::default();
    let source = "<script>/*\n retained\n*/</script>";
    let input = EmbedSource::authored(source, Span::new(8, source.len() as u32 - 9))
        .expect("authored script slice");
    let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
    assert!(parsed.admitted_program().is_some());
    assert_eq!(
        program_comment_ranges(&parsed),
        Err(SourceFoldingRefusal::UnsupportedSourceCoordinates)
    );
}

#[test]
fn wrapped_expression_comments_do_not_become_program_folding() {
    let arena = Allocator::default();
    let source = "/*\n retained\n*/ value";
    let input = EmbedSource::authored(source, Span::new(0, source.len() as u32))
        .expect("authored expression");
    let parsed = parse_once(
        &arena,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
            source: input,
        },
    );
    assert_eq!(
        program_comment_ranges(&parsed),
        Err(SourceFoldingRefusal::UnadmittedProgram)
    );
}

#[test]
fn authentic_zero_origin_prefix_has_only_program_local_ranges() {
    let arena = Allocator::default();
    let source = "/*\n retained\n*/ const prefix = 1;\n<template>not a Program</template>";
    let prefix_end = source.find("<template>").expect("larger original source");
    let input = EmbedSource::authored(source, Span::new(0, prefix_end as u32))
        .expect("authentic selected zero-origin prefix");
    let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
    assert!(parsed.admitted_program().is_some());
    assert!(parsed.source().text().len() < source.len());
    assert_eq!(program_comment_ranges(&parsed), Ok(vec![expected(0, 1)]));
    // This is a valid Program projection, without sealed File/document admission.
}

#[test]
fn flow_and_nonfatal_recovery_observations_cannot_admit_comment_folds() {
    for (source, hole) in [
        (
            "/*\n retained\n*/ const value = /x/uv;",
            vize_l1::embed::syntax::EmbedHole::Syntax,
        ),
        (
            "/* @flow */\n/*\n retained\n*/ super;",
            vize_l1::embed::syntax::EmbedHole::UnsupportedFlow,
        ),
    ] {
        let arena = Allocator::default();
        let input = EmbedSource::authored(source, Span::new(0, source.len() as u32))
            .expect("authentic rejected Program input");
        let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
        assert_eq!(parsed.hole(), Some(hole));
        assert!(parsed.comments().next().is_some());
        assert_eq!(
            program_comment_ranges(&parsed),
            Err(SourceFoldingRefusal::UnadmittedProgram)
        );
    }
}

#[test]
fn decoded_admitted_program_requires_its_authored_coordinate_bridge() {
    let arena = Allocator::default();
    let source = "/*&amp;\n retained\n*/";
    let input =
        vize_l1::embed::prepare_attribute_value(&arena, source, Span::new(0, source.len() as u32))
            .expect("authentic decoded Program source");
    assert!(input.decode_map().is_some());
    let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
    assert!(parsed.admitted_program().is_some());
    assert_eq!(
        program_comment_ranges(&parsed),
        Err(SourceFoldingRefusal::UnsupportedSourceCoordinates)
    );
}

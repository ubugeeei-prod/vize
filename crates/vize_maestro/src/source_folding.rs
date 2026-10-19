//! Experimental comment folding from one authentic native Program observation.
//!
//! This projection reads the retained native parse. It does not parse again or
//! select a production LSP route. SFC scripts need their producer-owned full-file
//! bridge before this standalone provider can project into a component document.

use oxc_ast::ast::CommentKind;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};
use vize_l0::line_index::LineBreaks;
use vize_l1::embed::syntax::NativeSyntax;

/// A refusal preserves the caller's original observation for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFoldingRefusal {
    UnadmittedProgram,
    UnsupportedSourceCoordinates,
    InvalidAuthoredComment,
}

/// Project retained multiline block comments in their original source order.
///
/// Only genuine admitted zero-origin Programs are accepted. Wrapped expressions,
/// recovered/Flow syntax and decoded or nonzero-origin embeds stay refused.
/// These coordinate checks do not attest a sealed File or an on-disk document.
/// Ranges hide the comment's interior and retain its closing delimiter line;
/// adjacent-line and single-line comments therefore produce no range.
pub fn program_comment_ranges(
    syntax: &NativeSyntax<'_>,
) -> Result<Vec<FoldingRange>, SourceFoldingRefusal> {
    if syntax.admitted_program().is_none() {
        return Err(SourceFoldingRefusal::UnadmittedProgram);
    }
    let source = syntax.source();
    let text = source.text();
    let span = source.span();
    if source.decode_map().is_some() || span.start != 0 || span.end as usize != text.len() {
        return Err(SourceFoldingRefusal::UnsupportedSourceCoordinates);
    }
    // LSP counts CR, LF and CRLF, while LS/PS stay inside the authored line.
    // Build this table once; each retained comment uses two binary lookups.
    let lines: Vec<_> = LineBreaks::Lsp.line_starts(text).collect();
    let line_at = |offset: usize| -> u32 {
        lines
            .partition_point(|start| *start <= offset)
            .saturating_sub(1) as u32
    };
    let mut ranges = Vec::new();
    for comment in syntax.comments() {
        if !matches!(
            comment.kind(),
            CommentKind::SingleLineBlock | CommentKind::MultiLineBlock
        ) {
            continue;
        }
        let span = comment
            .authored_span()
            .map_err(|_| SourceFoldingRefusal::InvalidAuthoredComment)?;
        let bytes = text
            .get(span.start as usize..span.end as usize)
            .ok_or(SourceFoldingRefusal::InvalidAuthoredComment)?;
        if !bytes.starts_with("/*") || !bytes.ends_with("*/") {
            return Err(SourceFoldingRefusal::InvalidAuthoredComment);
        }
        let start_line = line_at(span.start as usize);
        let close_line = line_at(span.end as usize - 2);
        if close_line.saturating_sub(start_line) < 2 {
            continue;
        }
        ranges.push(FoldingRange {
            start_line,
            start_character: None,
            end_line: close_line - 1,
            end_character: None,
            kind: Some(FoldingRangeKind::Comment),
            collapsed_text: None,
        });
    }
    Ok(ranges)
}

#[cfg(test)]
mod tests {
    use super::{SourceFoldingRefusal, program_comment_ranges};
    use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};
    use vize_l0::{Allocator, Span, cstr};
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
                let source = cstr!("/*\n first\n*/\n{statement}\n/*\n second\n*/\n");
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
            let source = cstr!("/* 😀\u{2028}\u{2029}{newline} 日本語{newline}*/");
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
        let input = vize_l1::embed::prepare_attribute_value(
            &arena,
            source,
            Span::new(0, source.len() as u32),
        )
        .expect("authentic decoded Program source");
        assert!(input.decode_map().is_some());
        let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
        assert!(parsed.admitted_program().is_some());
        assert_eq!(
            program_comment_ranges(&parsed),
            Err(SourceFoldingRefusal::UnsupportedSourceCoordinates)
        );
    }
}

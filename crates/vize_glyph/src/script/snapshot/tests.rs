use super::*;
use crate::EndOfLine;
use crate::script::{format_script_content_stable, format_sfc_script_content_stable};
use vize_l0::{
    Allocator, SourceBlock, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource, Lang,
        syntax::{ProgramGoal, ProgramOptions, parse_program_once},
    },
    markup::SurfaceParseOptions,
};

mod corpus;
mod profile;

#[derive(Default, Debug)]
struct Counts {
    scripts: usize,
    comparisons: usize,
    outputs: usize,
    errors: usize,
    output_bytes: usize,
}

fn same(
    original: Result<String, FormatError>,
    observed: Result<&str, &FormatError>,
    label: &str,
    totals: &mut Counts,
) {
    totals.comparisons += 1;
    match (original, observed) {
        (Ok(a), Ok(b)) => {
            assert_eq!(a.as_bytes(), b.as_bytes(), "{label}");
            totals.outputs += 1;
            totals.output_bytes += a.len();
        }
        (Err(FormatError::ScriptParseError(a)), Err(FormatError::ScriptParseError(b)))
        | (Err(FormatError::ScriptFormatError(a)), Err(FormatError::ScriptFormatError(b))) => {
            assert_eq!(a.as_bytes(), b.as_bytes(), "{label}");
            totals.errors += 1;
        }
        (a, b) => panic!("different formatter outcomes for {label}: {a:?} / {b:?}"),
    }
}

fn compare(
    arena: &Allocator,
    block: SourceBlock<'_>,
    language: ProgramOptions,
    label: &str,
    configs: &[(FormatOptions, Option<ImportSortOptions>)],
    counts: &mut Counts,
) {
    let source = EmbedSource::authored(block.root_source(), block.span()).unwrap();
    let syntax = parse_program_once(arena, source, language);
    let text = source.text();
    counts.scripts += 1;
    for (settings, sorting) in configs {
        let sort = sorting.as_ref();
        let query = ScriptFormatQuery::new(arena, &syntax, settings, sort).unwrap();
        same(
            format_script_content_stable(text, settings, arena, language.source_type(), sort),
            query.standalone(),
            label,
            counts,
        );
        same(
            format_sfc_script_content_stable(text, settings, arena, language.source_type(), sort),
            query.sfc_body(),
            label,
            counts,
        );
    }
}

fn variation() -> FormatOptions {
    FormatOptions {
        end_of_line: EndOfLine::Auto,
        print_width: 38,
        single_quote: true,
        jsx_single_quote: true,
        semi: false,
        use_tabs: true,
        ..FormatOptions::default()
    }
}

#[test]
fn standalone_and_sfc_profiles_keep_diagnostics_options_and_block_identity() {
    let cases = [
        ("", Lang::Js, false),
        (" \n\u{3000}\t", Lang::Ts, false),
        (";;;\r\n;", Lang::Js, false),
        ("/* kept */ ; ;", Lang::Js, false),
        ("'use strict';;", Lang::Js, false),
        (
            "// α\r\nimport Z from './z';\r\nimport A from './a';\r\nexport default [Z,A];",
            Lang::Ts,
            false,
        ),
        (
            "import './b';\rimport './a';\rexport const z='雪';",
            Lang::Js,
            false,
        ),
        (
            "const test=<T,>(x:T)=>x; const node=<Card value={test(3)}/>;",
            Lang::Ts,
            true,
        ),
        ("const a= <div x='雪'/>; export {a}", Lang::Js, true),
        ("/* @flow */ export const n: number = 1", Lang::Js, false),
        ("return x", Lang::Js, false),
        ("const typed = <T>v;", Lang::Ts, false),
        ("const broken=(;", Lang::Ts, true),
        ("export const x = (1);\n", Lang::Js, false),
    ];
    let mut options = Vec::new();
    for ending in [
        EndOfLine::Lf,
        EndOfLine::Crlf,
        EndOfLine::Cr,
        EndOfLine::Auto,
    ] {
        for skip in [false, true] {
            for sorting in [None, Some(ImportSortOptions::default())] {
                options.push((
                    FormatOptions {
                        end_of_line: ending,
                        skip_script_stabilization: skip,
                        ..variation()
                    },
                    sorting,
                ));
            }
        }
    }
    let mut count = Counts::default();
    for (text, lang, jsx) in cases {
        for goal in [ProgramGoal::Script, ProgramGoal::Module] {
            let arena = Allocator::default();
            compare(
                &arena,
                SourceRoot::new(text).unwrap().whole_block(),
                ProgramOptions { lang, jsx, goal },
                text,
                &options,
                &mut count,
            );
        }
    }
    assert!(count.errors > 0 && count.outputs > 0);
    assert_eq!(count.comparisons, cases.len() * 2 * options.len() * 2);
}

#[test]
fn readers_share_original_bytes_errors_and_the_transferred_heap_allocation() {
    let arena = Allocator::default();
    let options = FormatOptions::default();
    for text in [
        "export const longName = '雪雪雪雪雪雪雪雪雪雪雪雪雪雪雪雪雪雪';",
        ";;;",
        "",
        "return value;",
        "const invalid = (;",
    ] {
        let block = SourceRoot::new(text).unwrap().whole_block();
        let syntax = parse_program_once(
            &arena,
            EmbedSource::authored(text, block.span()).unwrap(),
            ProgramOptions::module(Lang::Js),
        );
        let query = ScriptFormatQuery::new(&arena, &syntax, &options, None).unwrap();
        assert!(query.output.get().is_none() && query.block_replacement.get().is_none());
        let outcome = query.sfc_body();
        if let Err(error) = outcome {
            assert!(core::ptr::eq(error, query.standalone().unwrap_err()));
            assert!(core::ptr::eq(error, query.sfc_body().unwrap_err()));
            continue;
        }
        let body = outcome.unwrap();
        let standalone = query.standalone().unwrap();
        if text == ";;;" {
            assert_eq!(body, ";");
            assert!(standalone.trim().is_empty());
        } else {
            assert!(core::ptr::eq(body, standalone));
        }
        if standalone.len() > 64 {
            let original = standalone.as_ptr();
            assert_eq!(query.into_standalone().unwrap().as_ptr(), original);
        }
    }
}

#[test]
fn unrequested_programs_remain_unformatted_and_wrapped_grammar_is_refused() {
    use vize_l1::embed::{Embed, Grammar, syntax::parse_once};
    let text = "value";
    let arena = Allocator::default();
    let options = FormatOptions::default();
    let src =
        EmbedSource::authored(text, SourceRoot::new(text).unwrap().whole_block().span()).unwrap();
    for shape in [
        Shape::Expr,
        Shape::HandlerBody,
        Shape::ForHead,
        Shape::SlotParams,
        Shape::FilterChain,
    ] {
        let syntax = parse_once(
            &arena,
            Embed {
                grammar: Grammar {
                    shape,
                    lang: Lang::Js,
                },
                source: src,
            },
        );
        assert_eq!(
            ScriptFormatQuery::new(&arena, &syntax, &options, None).err(),
            Some(shape)
        );
    }
    let syntax = parse_program_once(&arena, src, ProgramOptions::module(Lang::Js));
    let untouched = ScriptFormatQuery::new(&arena, &syntax, &options, None).unwrap();
    assert!(untouched.output.get().is_none());
    let expected =
        format_script_content_stable(text, &options, &arena, syntax.source_type(), None).unwrap();
    assert_eq!(untouched.into_standalone().unwrap(), expected);
}

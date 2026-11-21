//! On the same original script, sign off both the ordinary L2 file pass and
//! Glyph's one-pass printing options against their existing entry points.

#![cfg(test)]

use std::{fs, path::Path};
use vize_glyph::{FormatOptions, format_script_with_source_type};
use vize_l0::{
    Allocator, SourceBlock, SourceRoot, String, ToCompactString,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource, Lang,
        syntax::{
            EmbedSyntax, ProgramGoal, ProgramOptions, parse_program_once, snapshot::ProgramSnapshot,
        },
    },
    markup::SurfaceParseOptions,
};
use vize_l2::{
    file::FileArtifact,
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};

mod profile;

fn lexical<'a>(
    arena: &'a Allocator,
    block: SourceBlock<'a>,
    syntax: &EmbedSyntax<'a>,
) -> Option<FileArtifact<'a>> {
    let mut file = FileProducer::new(arena, block.root_source()).unwrap();
    file.program(
        ProgramInput::checked(syntax.admitted_program()?, block, 0)
            .expect("an admitted script must bind to its exact authored block"),
        ProgramScope::Nested,
    )
    .unwrap();
    Some(file.finish().unwrap())
}

fn rows(file: &FileArtifact<'_>) -> String {
    let units: Vec<_> = file
        .units()
        .iter()
        .map(|u| (u.id, u.span, u.profile, u.scope, u.interruption()))
        .collect();
    vize_l0::cstr!(
        "{units:?}{:?}{:?}{:?}{:?}{:?}{:?}",
        file.scopes(),
        file.bindings().map(|b| b.declaration()).collect::<Vec<_>>(),
        file.references(),
        file.imports(),
        file.exports(),
        file.issues()
    )
}

#[derive(Debug, Default)]
struct Counts {
    scripts: usize,
    semantic: usize,
    semantic_refused: usize,
    printed: usize,
    print_refused: usize,
    bytes: usize,
}

fn check(arena: &Allocator, block: SourceBlock<'_>, options: ProgramOptions, counts: &mut Counts) {
    let source = EmbedSource::authored(block.root_source(), block.span()).unwrap();
    let semantic = parse_program_once(arena, source, options);
    let snapshot = ProgramSnapshot::observe_printing(arena, &semantic).unwrap();
    let baseline = parse_program_once(arena, source, options);
    let actual = snapshot.semantic();
    assert_eq!(actual.hole(), baseline.hole());
    let messages = |s: &EmbedSyntax<'_>| {
        s.diagnostics()
            .map(|d| {
                (
                    d.message().to_owned(),
                    d.severity(),
                    d.labels().map(|r| r.decoded_span()).collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(messages(actual), messages(&baseline));
    assert!(std::ptr::eq(actual, &semantic));
    match (
        lexical(arena, block, actual),
        lexical(arena, block, &baseline),
    ) {
        (Some(a), Some(b)) => {
            assert_eq!(rows(&a), rows(&b));
            counts.semantic += 1;
        }
        (None, None) => counts.semantic_refused += 1,
        _ => panic!("semantic admission differs"),
    }
    let settings = FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    };
    let current =
        format_script_with_source_type(source.text(), &settings, arena, options.source_type());
    let actual = snapshot.for_print();
    match (actual, current) {
        (Ok(program), Ok(expected)) => {
            let scratch = Allocator::default();
            let printed = oxc_formatter::format_program(
                scratch.as_oxc(),
                program,
                settings.to_oxc_format_options(),
                None,
            )
            .print()
            .unwrap()
            .into_code();
            assert_eq!(printed, expected.as_str(), "{:?}", block.span());
            assert!(std::ptr::eq(program, snapshot.for_print().unwrap()));
            counts.printed += 1;
            counts.bytes += printed.len();
        }
        (Err(diagnostics), Err(vize_glyph::FormatError::ScriptParseError(message))) => {
            assert_eq!(
                diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.to_compact_string())
                    .collect::<Vec<_>>()
                    .join("; "),
                message.as_str()
            );
            assert!(std::ptr::eq(diagnostics, snapshot.for_print().unwrap_err()));
            counts.print_refused += 1;
        }
        _ => panic!("printer admission differs at {:?}", block.span()),
    }
    counts.scripts += 1;
}

#[test]
fn the_formatter_and_lexical_analyses_consume_their_original_profiles() {
    let mut count = Counts::default();
    for (text, lang, jsx, goal) in [
        (
            "export const x = (((1))); // retained",
            Lang::Js,
            false,
            ProgramGoal::Module,
        ),
        (
            "const value = <T>item;",
            Lang::Ts,
            false,
            ProgramGoal::Module,
        ),
        (
            "import C from 'c'; export const x=<C />;",
            Lang::Ts,
            true,
            ProgramGoal::Module,
        ),
        ("const x = <div/>", Lang::Js, false, ProgramGoal::Module),
        (
            "let value = 1; value;",
            Lang::Js,
            false,
            ProgramGoal::Script,
        ),
        (
            "/* α */ return value;",
            Lang::Js,
            false,
            ProgramGoal::Module,
        ),
        ("const value =", Lang::Js, false, ProgramGoal::Module),
        ("", Lang::Js, false, ProgramGoal::Module),
    ] {
        let arena = Allocator::default();
        check(
            &arena,
            SourceRoot::new(text).unwrap().whole_block(),
            ProgramOptions { lang, jsx, goal },
            &mut count,
        );
    }
    assert!(
        count.semantic_refused > 0
            && count.print_refused > 0
            && count.printed > 0
            && count.semantic > 0
    );
}

fn corpus(paths: impl Iterator<Item = impl AsRef<Path>>) {
    let mut count = Counts::default();
    let mut files = 0usize;
    let mut descriptors = 0usize;
    let mut no_script = 0usize;
    for path in paths {
        let path = path.as_ref();
        let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        files += 1;
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(
            &arena,
            &source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let Ok(view) = descriptor.admitted() else {
            descriptors += 1;
            continue;
        };
        let mut scripts = [view.ordinary(), view.setup()]
            .into_iter()
            .flatten()
            .peekable();
        if scripts.peek().is_none() {
            no_script += 1
        }
        for script in scripts {
            check(
                &arena,
                script.block(),
                ProgramOptions::module(script.lang()),
                &mut count,
            );
        }
    }
    eprintln!("files={files} descriptor_refused={descriptors} scriptless={no_script} {count:?}");
    assert!(count.scripts > 0 && count.printed > 0);
}

#[test]
fn repository_vue_scripts_match_both_original_consumers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for directory in [
        "tests/_fixtures",
        "tests/fixtures",
        "crates",
        "playground",
        "examples",
        "npm",
    ] {
        davinci_test_support::corpus::collect_vue_files(&root.join(directory), &mut files);
    }
    corpus(files.into_iter());
}

#[test]
#[ignore = "NUL-delimited paths to repository or pinned external Vue fixtures"]
fn original_sfc_script_corpus() {
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_PROGRAM_SNAPSHOT_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    corpus(manifest.split_terminator('\0').map(|path| root.join(path)));
}

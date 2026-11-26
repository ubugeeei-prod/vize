//! Independently run the pre-existing script passes on each original Program.
//! Compare the File, available query answers and every refused input.

use super::*;
use crate::framework::vue::{OptionsObserver, OrdinaryObserver, SetupObserver};
use std::{fs, path::PathBuf};

fn input<'p, 'a>(script: ScriptView<'_, 'a>, syntax: &'p EmbedSyntax<'a>) -> ProgramInput<'p, 'a> {
    ProgramInput::checked(
        syntax.admitted_program().unwrap(),
        script.block(),
        script.container_index(),
    )
    .unwrap()
}

#[derive(Debug, Default)]
struct Counts {
    files: usize,
    descriptor: usize,
    syntax: usize,
    incomplete: usize,
    complete: usize,
    bindings: usize,
    macros: usize,
    options_members: usize,
    unclassified: usize,
}

fn compare(source: &str, count: &mut Counts, path: &std::path::Path) {
    count.files += 1;
    let arena = Allocator::default();
    let Some(parsed) = Parsed::new(&arena, source) else {
        count.descriptor += 1;
        return;
    };
    #[cfg(feature = "script-queries")]
    let view = parsed.l1ir.admitted().unwrap();
    #[cfg(feature = "script-queries")]
    view.with_script_programs(&arena, |programs| {
        let produce = || {
            VueL2IrProducer::from_script_programs(&arena, programs, ALL)
                .map(|producer| producer.finish().unwrap())
        };
        compare_file(
            &arena,
            &parsed,
            programs.get(ScriptRole::Ordinary),
            programs.get(ScriptRole::Setup),
            produce,
            count,
            path,
        );
    });
    #[cfg(not(feature = "script-queries"))]
    compare_file(
        &arena,
        &parsed,
        parsed.ordinary.as_ref(),
        parsed.setup.as_ref(),
        || parsed.produce(&arena, ALL),
        count,
        path,
    );
}

fn compare_file<'a>(
    arena: &'a Allocator,
    parsed: &Parsed<'a>,
    ordinary_syntax: Option<&EmbedSyntax<'a>>,
    setup_syntax: Option<&EmbedSyntax<'a>>,
    produce: impl FnOnce() -> Result<VueL2Ir<'a>, VueProgramError>,
    count: &mut Counts,
    path: &std::path::Path,
) {
    let view = parsed.l1ir.admitted().unwrap();
    let source = view.source();
    if [ordinary_syntax, setup_syntax]
        .into_iter()
        .flatten()
        .any(|p| p.admitted_program().is_none())
    {
        let error = produce().err().unwrap();
        assert!(matches!(error, VueProgramError::Missing(_)));
        assert_eq!(parsed.produce(arena, ALL).err(), Some(error));
        count.syntax += 1;
        return;
    }
    let candidate = produce().unwrap();
    let model = candidate.model();
    let mut control = L2IrProducer::new(arena, source).unwrap();
    let ordinary = view
        .ordinary()
        .zip(ordinary_syntax)
        .map(|(script, syntax)| {
            let mut bindings = OrdinaryObserver::new();
            let (_, scope) = control
                .program_observed(input(script, syntax), ProgramScope::Module, &mut bindings)
                .unwrap();
            let mut second_file = L2IrProducer::new(arena, source).unwrap();
            let mut options = OptionsObserver::new();
            second_file
                .program_observed(input(script, syntax), ProgramScope::Module, &mut options)
                .unwrap();
            (scope, bindings.finish(), options.finish())
        });
    let setup = view.setup().zip(setup_syntax).map(|(script, syntax)| {
        let mut observer = SetupObserver::new().with_hoist_static(ordinary.is_none());
        let (_, scope) = control
            .program_observed(input(script, syntax), ProgramScope::Nested, &mut observer)
            .unwrap();
        (scope, observer.finish())
    });
    let control = control.finish().unwrap();
    assert_eq!(candidate.l2ir().issues(), control.issues(), "{path:?}");
    assert_eq!(
        candidate.l2ir().is_complete(),
        control.is_complete(),
        "{path:?}"
    );
    // Include all original lexical rows and their targets, even on refused Files.
    let rows = |l2ir: &L2Ir<'_>| {
        std::format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}{:?}",
            l2ir.units(),
            l2ir.scopes(),
            l2ir.bindings()
                .map(|r| r.declaration())
                .collect::<alloc::vec::Vec<_>>(),
            l2ir.references(),
            l2ir.imports(),
            l2ir.exports(),
            l2ir.issues()
        )
    };
    assert_eq!(rows(candidate.l2ir()), rows(&control), "{path:?}");
    if !control.is_complete() {
        assert_eq!(model.contract(), Err(VueQueryError::IncompleteFile));
        assert_eq!(model.options(), Err(VueQueryError::IncompleteFile));
        for binding in candidate.l2ir().bindings() {
            assert_eq!(
                model.binding_kind(binding),
                Err(VueQueryError::IncompleteFile)
            );
        }
        count.incomplete += 1;
        return;
    }
    count.complete += 1;
    if let Some((_, facts)) = &setup {
        let oracle = facts
            .as_ref()
            .map_err(|e| VueQueryError::SetupIncomplete(*e));
        assert_eq!(
            model.contract(),
            oracle.map(SetupAnalysis::contract),
            "{path:?}"
        );
        assert_eq!(
            model.macro_calls(),
            oracle.map(SetupAnalysis::macro_calls),
            "{path:?}"
        );
        count.macros += model.macro_calls().map_or(0, <[_]>::len);
    } else {
        assert_eq!(
            model.contract(),
            Err(VueQueryError::NotObserved(VueFamily::Setup))
        );
    }
    if let Some((_, _, facts)) = &ordinary {
        let oracle = facts
            .as_ref()
            .map(OptionsAnalysis::component)
            .map_err(|e| VueQueryError::OptionsIncomplete(*e));
        assert_eq!(model.options(), oracle, "{path:?}");
        count.options_members += model
            .options()
            .ok()
            .flatten()
            .map_or(0, |c| c.members.len());
    } else {
        assert_eq!(
            model.options(),
            Err(VueQueryError::NotObserved(VueFamily::Options))
        );
    }
    for binding in candidate.l2ir().bindings() {
        let row = binding.declaration().unwrap();
        let expected = if row.namespace != Namespace::Value {
            Ok(None)
        } else if let Some((scope, facts)) = &setup
            && *scope == row.scope
        {
            facts
                .as_ref()
                .map_err(|e| VueQueryError::SetupIncomplete(*e))
                .and_then(|facts| {
                    if let Some(span) = facts
                        .unclassified()
                        .iter()
                        .find(|s| s.start <= row.span.start && row.span.end <= s.end)
                    {
                        count.unclassified += 1;
                        return Err(VueQueryError::Unclassified(*span));
                    }
                    exact(facts.bindings(), row.span).map(|v| v.map(VueBinding::Setup))
                })
        } else if let Some((scope, facts, _)) = &ordinary
            && *scope == row.scope
        {
            facts
                .as_ref()
                .map_err(|e| VueQueryError::OrdinaryIncomplete(*e))
                .and_then(|facts| {
                    exact(facts.bindings(), row.span).map(|v| v.map(VueBinding::Ordinary))
                })
        } else {
            Ok(None)
        };
        assert_eq!(model.binding_kind(binding), expected, "{path:?} {row:?}");
        count.bindings += usize::from(expected.is_ok_and(|v| v.is_some()));
    }
}

fn exact(rows: &[SetupBinding], span: Span) -> Result<Option<SetupBinding>, VueQueryError> {
    let mut matching = rows.iter().filter(|row| row.name == span);
    let first = matching.next().copied();
    if matching.next().is_some() {
        Err(VueQueryError::Ambiguous(span))
    } else {
        Ok(first)
    }
}

#[test]
fn existing_vue_corpus_agrees_with_the_independent_script_observers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = alloc::vec::Vec::new();
    for tree in [
        "tests/_fixtures",
        "tests/fixtures",
        "crates",
        "playground",
        "examples",
        "npm",
    ] {
        davinci_test_support::corpus::collect_vue_files(&root.join(tree), &mut files);
    }
    let mut count = Counts::default();
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        compare(&source, &mut count, path.strip_prefix(&root).unwrap());
    }
    assert!(count.files > 1000 && count.complete > 0 && count.incomplete > 0 && count.bindings > 0);
    assert_eq!(
        count.files,
        count.descriptor + count.syntax + count.incomplete + count.complete
    );
    std::eprintln!("{count:?}");
}

#[test]
#[ignore = "accepts a NUL-delimited manifest of pinned real project Vue files"]
fn external_vue_corpus() {
    let list = fs::read_to_string(std::env::var_os("VIZE_VUE_QUERY_MANIFEST").unwrap()).unwrap();
    let mut count = Counts::default();
    for path in list.split_terminator('\0').map(PathBuf::from) {
        compare(&fs::read_to_string(&path).unwrap(), &mut count, &path);
    }
    assert!(count.files > 0 && count.complete > 0);
    std::eprintln!("{count:?}");
}

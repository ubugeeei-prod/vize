//! Native `<script setup>` facts: top-level macro calls equal Croquis' `MacroTracker::all_calls()`
//! on every `<script setup>` in the repository corpus (#6844). Croquis is
//! only this test's oracle.

use std::path::{Path, PathBuf};

use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
    cstr,
};
use vize_l1::container::{
    Vue,
    vue::{DescriptorOptions, ScriptRole},
};
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l1::markup::SurfaceParseOptions;
use vize_l2::framework::vue::{MACROS, MacroCall, MacroKind, SetupFactsIncomplete, SetupObserver};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

/// Repository trees holding `.vue` files, relative to the workspace root.
const CORPUS: &[&str] = &[
    "tests/_fixtures",
    "tests/fixtures",
    "crates",
    "playground",
    "examples",
    "npm",
];
const SKIPPED_DIRECTORIES: &[&str] = &["node_modules", "target", "dist", ".git"];

/// Files where Croquis disagrees with Vue, which is the authority. Croquis
/// records no macro in an object-pattern declarator except `defineProps`;
/// Vue processes `defineSlots` there (`const { bottom } = defineSlots<T>()`).
const CROQUIS_DEFECTS: &[&str] =
    &["tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#4326/main.vue"];

/// `(name, start, end, runtime arguments, type arguments)`, script-relative.
type Row = (String, u32, u32, Option<String>, Option<String>);

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if !SKIPPED_DIRECTORIES.iter().any(|skip| name == *skip) {
                collect(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "vue") {
            files.push(path);
        }
    }
}

fn slice(source: &str, base: u32, span: vize_l0::Span) -> Option<String> {
    source
        .get((span.start - base) as usize..(span.end - base) as usize)
        .map(String::from)
}

/// Croquis records Vue's macros and `defineArt` only.
fn croquis_records(kind: MacroKind) -> bool {
    MACROS
        .iter()
        .any(|row| row.kind == kind && (row.builtin || kind == MacroKind::DefineArt))
}

fn native_rows(content: &str, base: u32, calls: &[MacroCall]) -> Vec<Row> {
    calls
        .iter()
        .filter(|call| croquis_records(call.kind))
        .map(|call| {
            let name = slice(content, base, call.callee).unwrap_or_default();
            (
                name,
                call.span.start - base,
                call.span.end - base,
                call.arguments.and_then(|span| slice(content, base, span)),
                call.type_arguments
                    .and_then(|span| slice(content, base, span)),
            )
        })
        .collect()
}

fn croquis_rows(content: &str) -> Vec<Row> {
    vize_croquis::script_parser::parse_script_setup(content)
        .macros
        .all_calls()
        .iter()
        .map(|call| {
            (
                String::from(call.name.as_str()),
                call.start,
                call.end,
                call.runtime_args.as_deref().map(String::from),
                call.type_args.as_deref().map(String::from),
            )
        })
        .collect()
}

/// The native calls of the file's `<script setup>` with its content and
/// start, or `None` when the native levels refuse the file first.
fn observe(source: &str) -> Option<(String, u32, Result<Vec<MacroCall>, SetupFactsIncomplete>)> {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = descriptor.admitted().ok()?;
    let script = view.setup()?;
    assert_eq!(script.role(), ScriptRole::Setup);
    let block = script.block();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(script.lang()),
    );
    let admitted = syntax.admitted_program()?;
    let mut producer = FileProducer::new(&arena, descriptor.source()).ok()?;
    let mut observer = SetupObserver::new();
    producer
        .program_observed(
            ProgramInput::checked(admitted, block, script.container_index()).ok()?,
            ProgramScope::Nested,
            &mut observer,
        )
        .ok()?;
    let calls = observer.finish().map(|facts| facts.macro_calls().to_vec());
    let content = source.get(block.span().start as usize..block.span().end as usize)?;
    Some((String::from(content), block.span().start, calls))
}

fn native(source: &str) -> Option<(String, Result<Vec<Row>, SetupFactsIncomplete>)> {
    let (content, base, calls) = observe(source)?;
    let rows = calls.map(|calls| native_rows(&content, base, &calls));
    Some((content, rows))
}

#[test]
fn every_corpus_setup_matches_croquis() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for tree in CORPUS {
        collect(&root.join(tree), &mut files);
    }
    files.sort();
    let mut compared = 0usize;
    let mut with_calls = 0usize;
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, native)) = native(&source) else {
            continue;
        };
        compared += 1;
        with_calls += usize::from(native.as_ref().is_ok_and(|rows| !rows.is_empty()));
        let oracle = Ok(croquis_rows(&content));
        if native != oracle {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            mismatches.push(cstr!(
                "{relative}\n  native:  {native:?}\n  croquis: {oracle:?}"
            ));
        }
    }
    eprintln!("compared {compared} setup scripts ({with_calls} with macro calls)");
    assert!(compared > 1000, "corpus shrank to {compared} setup scripts");
    let defects = mismatches
        .iter()
        .filter_map(|mismatch| mismatch.split_once('\n').map(|(path, _)| path))
        .collect::<Vec<_>>();
    assert_eq!(defects, CROQUIS_DEFECTS, "{mismatches:#?}");
}

/// `(kind, call text, declarator text)` of each native call.
fn shapes(script: &str) -> Option<Vec<(MacroKind, String, Option<String>)>> {
    let source = cstr!("<script setup lang=\"ts\">{script}</script>");
    let (content, base, calls) = observe(&source)?;
    let rows = calls
        .ok()?
        .iter()
        .map(|call| {
            (
                call.kind,
                slice(&content, base, call.span).unwrap_or_default(),
                call.declarator.and_then(|span| slice(&content, base, span)),
            )
        })
        .collect();
    Some(rows)
}

fn text(value: &str) -> String {
    String::from(value)
}

#[test]
fn with_defaults_records_its_define_props_with_the_same_pattern() {
    assert_eq!(
        shapes("const { a = 1 } = withDefaults(defineProps<{ a?: number }>(), { a: 2 })"),
        Some(vec![
            (
                MacroKind::WithDefaults,
                text("withDefaults(defineProps<{ a?: number }>(), { a: 2 })"),
                Some(text("{ a = 1 }")),
            ),
            (
                MacroKind::DefineProps,
                text("defineProps<{ a?: number }>()"),
                Some(text("{ a = 1 }")),
            ),
        ])
    );
}

#[test]
fn typescript_wrappers_and_parentheses_are_transparent() {
    assert_eq!(
        shapes(
            "const a = defineProps() as any\nconst b = (defineEmits(['x']))!\n\
             const c = defineModel() satisfies object\n(defineExpose({}))"
        ),
        Some(vec![
            (
                MacroKind::DefineProps,
                text("defineProps()"),
                Some(text("a"))
            ),
            (
                MacroKind::DefineEmits,
                text("defineEmits(['x'])"),
                Some(text("b")),
            ),
            (
                MacroKind::DefineModel,
                text("defineModel()"),
                Some(text("c"))
            ),
            (MacroKind::DefineExpose, text("defineExpose({})"), None),
        ])
    );
}

#[test]
fn only_top_level_statements_carry_macros() {
    assert_eq!(
        shapes(
            "declare const a: ReturnType<typeof defineProps>\n\
             function f() { defineEmits() }\n\
             const g = () => defineSlots()\n\
             foo(defineOptions({}))\n\
             definePageMeta({ layout: 'x' })"
        ),
        Some(vec![(
            MacroKind::DefinePageMeta,
            text("definePageMeta({ layout: 'x' })"),
            None,
        )])
    );
}

#[test]
fn one_observer_refuses_a_second_program() {
    let source = "<script>export const a = 1</script>\n<script setup>defineProps()</script>";
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = descriptor.admitted().expect("descriptor");
    let mut producer = FileProducer::new(&arena, descriptor.source()).expect("producer");
    let mut observer = SetupObserver::new();
    for script in [view.ordinary(), view.setup()].into_iter().flatten() {
        let syntax = parse_program_once(
            &arena,
            EmbedSource::authored(source, script.block().span()).expect("block"),
            ProgramOptions::module(script.lang()),
        );
        let input = ProgramInput::checked(
            syntax.admitted_program().expect("program"),
            script.block(),
            script.container_index(),
        )
        .expect("input");
        producer
            .program_observed(input, ProgramScope::Module, &mut observer)
            .expect("unit");
    }
    assert_eq!(observer.finish(), Err(SetupFactsIncomplete::SecondProgram));
}

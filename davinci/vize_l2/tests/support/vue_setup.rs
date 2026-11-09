//! The repository's `<script setup>` corpus observed by `SetupObserver`.

use std::path::{Path, PathBuf};

use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
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
use vize_l2::framework::vue::{SetupFacts, SetupFactsIncomplete, SetupObserver};
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

/// The workspace root and every corpus `.vue` file, sorted.
pub fn corpus() -> (PathBuf, Vec<PathBuf>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for tree in CORPUS {
        collect(&root.join(tree), &mut files);
    }
    files.sort();
    (root, files)
}

pub fn slice(source: &str, base: u32, span: vize_l0::Span) -> Option<String> {
    source
        .get((span.start - base) as usize..(span.end - base) as usize)
        .map(String::from)
}

pub fn text(value: &str) -> String {
    String::from(value)
}

/// The facts of the file's `<script setup>` with its content and start, or
/// `None` when the native levels refuse the file first.
pub fn observe(source: &str) -> Option<(String, u32, Result<SetupFacts, SetupFactsIncomplete>)> {
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
    let facts = observer.finish();
    let content = source.get(block.span().start as usize..block.span().end as usize)?;
    Some((String::from(content), block.span().start, facts))
}

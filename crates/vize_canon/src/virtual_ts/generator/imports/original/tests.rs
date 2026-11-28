use super::super::{ImportNames, collect_identifier_usage, import_parse_count};
use super::*;
use crate::virtual_ts::{
    VirtualTsOptions, VirtualTsOutput, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_original_imports,
};
use davinci_harness::alloc;
#[cfg(not(feature = "plain-script-bridges"))]
use davinci_harness::alloc::CountingAllocator;
#[cfg(not(feature = "plain-script-bridges"))]
use std::alloc::System;
use std::collections::BTreeSet;
use vize_carton::Allocator;
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_l0::{
    SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::SurfaceParseOptions,
};

#[cfg(not(feature = "plain-script-bridges"))]
#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn same(actual: &VirtualTsOutput, expected: &VirtualTsOutput, name: &str) {
    assert_eq!(actual.code, expected.code, "{name}");
    assert_eq!(actual.mapping, expected.mapping, "{name}");
}

fn analyze(script: &str, setup: bool, template: Option<&vize_relief::RootNode<'_>>) -> Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if setup {
        analyzer.analyze_script_setup(script);
    } else {
        analyzer.analyze_script_plain(script);
    }
    if let Some(template) = template {
        analyzer.analyze_template(template);
    }
    analyzer.finish()
}

#[test]
fn original_shared_import_facts_keep_full_documents_and_remove_slice_parses() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/generation-value-import-facts/Imports.vue.txt"
    ));
    let arena = Allocator::new();
    let descriptor = Vue.observe_descriptor(&arena, fixture, options());
    let selected = descriptor.admitted().unwrap();
    let block = selected.setup().unwrap().block();
    let original = AuthoredProgram::parse(&arena, block, SourceType::ts());
    let facts = OriginalImportFacts::new(&original).unwrap();
    let template = selected
        .template()
        .map(|template| vize_armature::parse(&arena, template.block().source()).0);
    let summary = analyze(block.source(), true, template.as_ref());
    let usage = collect_identifier_usage(&original.parsed().program);
    let expected = ImportNames::collect(&summary, Some(block.source()), &usage);
    let actual = ImportNames::from_original(&summary, block.source(), &usage, &facts);
    assert_eq!(actual.values.bindings, expected.values.bindings);
    assert_eq!(
        actual.values.without_script_value_use,
        expected.values.without_script_value_use
    );
    assert_eq!(
        actual
            .values
            .bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<BTreeSet<_>>(),
        ["DefaultValue", "aliased", "namespace"]
            .into_iter()
            .collect()
    );
    let before = import_parse_count(false);
    let expected = generate_virtual_ts_with_offsets(
        &summary,
        Some(block.source()),
        template.as_ref(),
        block.start(),
        selected
            .template()
            .map_or(0, |template| template.block().start()),
        &VirtualTsOptions::default(),
    );
    assert!(import_parse_count(false) > before);
    let before = import_parse_count(false);
    let actual = generate_virtual_ts_with_original_imports(
        &summary,
        block.source(),
        template.as_ref(),
        block.start(),
        selected
            .template()
            .map_or(0, |template| template.block().start()),
        &VirtualTsOptions::default(),
        &facts,
    )
    .unwrap();
    assert_eq!(import_parse_count(false), before);
    same(&actual, &expected, "pinned original");
}

#[test]
fn authored_spelling_and_exact_original_selection_remain_required() {
    let source = r"import { value as \u006cocal } from 'pkg'; import type Type from 'types';";
    let block = SourceRoot::new(source).unwrap().block(source, 0).unwrap();
    let arena = Allocator::new();
    let original = AuthoredProgram::parse(&arena, block, SourceType::ts());
    let facts = OriginalImportFacts::new(&original).unwrap();
    let summary = analyze(source, true, None);
    let usage = collect_identifier_usage(&original.parsed().program);
    let names = ImportNames::from_original(&summary, source, &usage, &facts);
    assert_eq!(names.values.bindings, Default::default());
    assert_eq!(
        facts.value_names(&summary).collect::<Vec<_>>(),
        [r"\u006cocal"]
    );
    let copy = source.to_owned();
    assert_eq!(
        facts.validate(&summary, Some(&copy)),
        Err(OriginalImportError::Source)
    );
    let other = analyze("\nimport Different from 'different';", true, None);
    assert_eq!(
        facts.validate(&other, Some(source)),
        Err(OriginalImportError::StatementSelection)
    );
    let jsx = AuthoredProgram::parse(&arena, block, SourceType::tsx());
    assert!(matches!(
        OriginalImportFacts::new(&jsx),
        Err(OriginalImportError::Grammar)
    ));
    let invalid = "import { broken from 'pkg';";
    let block = SourceRoot::new(invalid).unwrap().block(invalid, 0).unwrap();
    let recovered = AuthoredProgram::parse(&arena, block, SourceType::ts());
    assert!(matches!(
        OriginalImportFacts::new(&recovered),
        Err(OriginalImportError::Syntax(_))
    ));
}

#[derive(Debug, Default)]
struct Coverage {
    files: usize,
    no_descriptor: usize,
    script_selection: usize,
    syntax_refused: usize,
    statement_refused: usize,
    admitted: usize,
    imports: usize,
    bytes: usize,
    invalid: usize,
}

fn compare_original(source: &str, name: &str, counts: &mut Coverage) {
    counts.files += 1;
    let arena = Allocator::new();
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let Ok(selected) = descriptor.admitted() else {
        counts.no_descriptor += 1;
        return;
    };
    let script = match (selected.ordinary(), selected.setup()) {
        (Some(script), None) | (None, Some(script)) => script,
        _ => {
            counts.script_selection += 1;
            return;
        }
    };
    let block = script.block();
    let original = AuthoredProgram::parse(&arena, block, SourceType::ts());
    let Ok(facts) = OriginalImportFacts::new(&original) else {
        counts.syntax_refused += 1;
        return;
    };
    let template = selected
        .template()
        .map(|template| vize_armature::parse(&arena, template.block().source()).0);
    let summary = analyze(
        block.source(),
        selected.setup().is_some(),
        template.as_ref(),
    );
    if facts.validate(&summary, Some(block.source())).is_err() {
        counts.statement_refused += 1;
        return;
    }
    let template_offset = selected
        .template()
        .map_or(0, |template| template.block().start());
    let options = VirtualTsOptions::default();
    let expected = generate_virtual_ts_with_offsets(
        &summary,
        Some(block.source()),
        template.as_ref(),
        block.start(),
        template_offset,
        &options,
    );
    let actual = generate_virtual_ts_with_original_imports(
        &summary,
        block.source(),
        template.as_ref(),
        block.start(),
        template_offset,
        &options,
        &facts,
    )
    .unwrap();
    same(&actual, &expected, name);
    counts.admitted += 1;
    counts.imports += summary.import_statements.len();
    counts.bytes += actual.code.len();
}

#[test]
#[ignore = "full original-source virtual documents and mapping packets"]
fn original_import_documents_on_tracked_sources() {
    let manifest = std::fs::read(std::env::var_os("VIZE_ORIGINAL_IMPORT_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts = Coverage::default();
    for entry in manifest
        .split(|&b| b == 0)
        .filter(|entry| !entry.is_empty())
    {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => compare_original(source, name, &mut counts),
            Err(_) => counts.invalid += 1,
        }
    }
    assert!(counts.admitted > 0 && counts.imports > 0, "{counts:?}");
    eprintln!("original_import_corpus {counts:?}");
}

mod profile;

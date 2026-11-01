//! Complete syntax facts, unchanged late fallbacks, and a consumed whole SFC.

use std::collections::BTreeSet;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{CompactString, FxHashSet};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

use super::super::{
    global_components::GlobalComponentPlan,
    imports::{
        ImportNames, collect_identifier_usage, extract_import_facts_for_test, import_parse_count,
    },
    options_api_support::find_options_api_props,
};
use crate::batch::{ImportRewriter, ImportSourceMap, generate_vue_document_virtual_ts};
use crate::virtual_ts::VirtualTsOptions;

mod reference;

const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/prepared-type-only-import-facts/Imports.vue.txt"
));

fn types(names: &[&str]) -> FxHashSet<CompactString> {
    names.iter().map(|name| CompactString::new(*name)).collect()
}

fn summary(script: &str) -> Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.finish()
}

#[test]
fn whole_import_syntax_facts_match_both_original_extractors() {
    for (source, values, type_names) in [
        ("import 'pkg'", vec![], vec![]),
        ("import Value from 'pkg'", vec!["Value"], vec![]),
        (
            "import Value, * as space from 'pkg'",
            vec!["Value", "space"],
            vec![],
        ),
        (
            "import { type Model, value as local } from 'pkg'",
            vec!["local"],
            vec!["Model"],
        ),
        ("import type Value from 'pkg'", vec![], vec!["Value"]),
        ("import type * as space from 'pkg'", vec![], vec!["space"]),
        (
            "import type DefaultBadge, { Badge, type Props as LocalProps } from 'pkg'",
            vec![],
            vec!["DefaultBadge", "Badge", "LocalProps"],
        ),
        ("import { type as local } from 'pkg'", vec!["local"], vec![]),
        (
            "import { 'string-name' as local, type café as résumé } from 'pkg'",
            vec!["local"],
            vec!["résumé"],
        ),
        (
            r"import { value as \u006cocal, type Model as \u0050rops } from 'pkg'",
            vec![r"\u006cocal"],
            vec![r"\u0050rops"],
        ),
        (
            "import data from './data.json' with { type: 'json' }",
            vec!["data"],
            vec![],
        ),
        ("import Thing = require('pkg')", vec![], vec![]),
    ] {
        let actual = extract_import_facts_for_test(source);
        assert_eq!(actual.0, values, "{source}");
        assert_eq!(actual.1, types(&type_names), "{source}");
        reference::assert_facts(source, actual);
    }
    for source in [
        "import { value as } from 'pkg';",
        "import type { Missing",
        "import { type T, value from 'pkg';",
        "import Value from",
        "import type",
        "const before = 1; import type { T } from 'pkg';",
        "import 'pkg'; import type { T } from 'later';",
    ] {
        reference::assert_facts(source, extract_import_facts_for_test(source));
    }
}

#[test]
fn prepared_type_names_survive_empty_values_and_empty_completed_extraction() {
    for (script, expected) in [
        (
            "import type { Model } from './model'; const props = defineProps<{ value: Model }>();",
            types(&["Model"]),
        ),
        (
            "import Value from './model'; const props = defineProps<{ value: number }>();",
            types(&[]),
        ),
    ] {
        let summary = summary(script);
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
        let usage = collect_identifier_usage(&parsed.program);
        let before = import_parse_count(false);
        let plan = ImportNames::collect(&summary, Some(script), &usage);
        assert_eq!(plan.type_only.prepared, Some(expected.clone()));
        let globals = GlobalComponentPlan::new((&summary, None), false, false, None);
        let actual =
            super::syntactic_type_only_imported_names(&summary, &globals, false, plan.type_only);
        assert_eq!(actual, expected);
        assert_eq!(import_parse_count(false) - before, 1);
    }
}

#[test]
fn macro_absent_gates_and_options_models_fallback_keep_original_parses() {
    for (script, expected, parses) in [
        ("import type { Model } from './model';", types(&[]), 1),
        (
            "import type { Model } from './model'; export default { props: { label: String } };",
            types(&["Model"]),
            2,
        ),
        (
            "import type { Model } from './model'; const model = defineModel<Model>();",
            types(&["Model"]),
            2,
        ),
    ] {
        let summary = summary(script);
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
        let usage = collect_identifier_usage(&parsed.program);
        let before = import_parse_count(false);
        let plan = ImportNames::collect(&summary, Some(script), &usage);
        assert_eq!(plan.type_only.prepared, None);
        let globals = GlobalComponentPlan::new((&summary, None), false, false, None);
        let actual = super::syntactic_type_only_imported_names(
            &summary,
            &globals,
            find_options_api_props(script).is_some(),
            plan.type_only,
        );
        assert_eq!(actual, expected);
        assert_eq!(import_parse_count(false) - before, parses);
    }
}

#[test]
fn pinned_sfc_uses_exact_facts_and_generates_default_and_hoisted_documents() {
    let descriptor = parse_sfc(FIXTURE, SfcParseOptions::default()).unwrap();
    let script = descriptor.script_setup.as_ref().unwrap().content.as_ref();
    let summary = summary(script);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let usage = collect_identifier_usage(&parsed.program);
    let before = import_parse_count(false);
    let plan = ImportNames::collect(&summary, Some(script), &usage);
    assert_eq!(
        plan.values
            .bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<BTreeSet<_>>(),
        ["Badge", "local", "values"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        plan.values
            .without_script_value_use
            .iter()
            .map(|name| name.as_str())
            .collect::<BTreeSet<_>>(),
        ["local"].into_iter().collect::<BTreeSet<_>>()
    );
    let expected = types(&["Payload", "TypeBadge", "TypeSpace", r"\u0050rops"]);
    assert_eq!(plan.type_only.prepared, Some(expected.clone()));
    let globals = GlobalComponentPlan::new((&summary, None), false, false, None);
    assert_eq!(
        super::syntactic_type_only_imported_names(&summary, &globals, false, plan.type_only),
        expected
    );
    assert_eq!(import_parse_count(false) - before, 5);

    for hoist in [false, true] {
        let document = generate_vue_document_virtual_ts(
            std::path::Path::new("src/Imports.vue"),
            FIXTURE,
            &VirtualTsOptions::default(),
            &ImportRewriter::new(),
            hoist,
        )
        .unwrap();
        assert_eq!(document.code, document.pre_rewrite_code);
        assert_eq!(document.import_source_map, ImportSourceMap::default());
        assert_eq!(document.source_type, SourceType::ts());
        assert_eq!(document.virtual_suffix, ".ts");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &document.code, SourceType::ts()).parse();
        assert_eq!(
            parsed.diagnostics.len(),
            0,
            "{hoist}: {:?}",
            parsed.diagnostics
        );
    }
}

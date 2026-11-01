//! The pinned script feeds the real import collector and complete anchor emitter.

use super::{SetupBindingAnchors, emit_setup_binding_anchors};
use crate::virtual_ts::generator::imports::{
    IdentifierUsage, ImportNames, collect_identifier_usage, import_parse_count,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{FxHashSet, String};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generation-value-import-facts/Imports.vue.txt"
));
const LABEL: &str = "Reference setup bindings used by template generation";

fn analyze(script: &str) -> (Croquis, IdentifierUsage) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    (analyzer.finish(), collect_identifier_usage(&parsed.program))
}

fn names(values: &[&str]) -> FxHashSet<String> {
    values.iter().map(|name| String::from(*name)).collect()
}

#[test]
fn pinned_import_facts_keep_whole_sets_and_both_anchor_modes() {
    let descriptor = parse_sfc(FIXTURE, SfcParseOptions::default()).unwrap();
    let script = descriptor.script_setup.as_ref().unwrap().content.as_ref();
    let (summary, usage) = analyze(script);
    let before = import_parse_count(false);
    let imports = ImportNames::collect(&summary, Some(script), &usage).values;
    assert_eq!(import_parse_count(false) - before, 4);
    assert_eq!(
        imports
            .bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        ["DefaultValue", "aliased", "namespace"]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(imports.without_script_value_use, names(&["aliased"]));
    let template = names(&["LocalState", "aliased", "local"]);
    for (preserve, expected) in [
        (
            true,
            "\n  // Reference setup bindings used by template generation\n  void (null as unknown as keyof typeof aliased); void local;\n",
        ),
        (
            false,
            "\n  // Reference setup bindings (used in template/CSS v-bind)\n  void (null as unknown as keyof typeof DefaultValue); void (null as unknown as keyof typeof aliased); void local; void (null as unknown as keyof typeof namespace);\n",
        ),
    ] {
        let mut output = String::default();
        SetupBindingAnchors {
            summary: &summary,
            imports: &imports,
            usage: &usage,
            template_usage_names: &template,
            template_value_usage_names: &template,
            preserve_unused_diagnostics: preserve,
        }
        .emit(&mut output, "test.value_import_names");
        assert_eq!(output.as_str(), expected);
    }
    let mut type_only_output = String::default();
    emit_setup_binding_anchors(
        &mut type_only_output,
        &summary,
        &imports,
        &usage,
        Some(&names(&["aliased", "local"])),
        &names(&["local"]),
        LABEL,
    );
    assert_eq!(
        type_only_output.as_str(),
        "\n  // Reference setup bindings used by template generation\n  void local;\n"
    );
    assert_eq!(imports.without_script_value_use, names(&["aliased"]));
    assert_eq!(import_parse_count(false) - before, 4);
}

#[test]
fn import_facts_own_their_generation_and_preserve_the_binding_filter() {
    let first_script = "import { value as first } from 'one'; const local = 1;";
    let (first_summary, first_usage) = analyze(first_script);
    let first = ImportNames::collect(&first_summary, Some(first_script), &first_usage).values;
    let (second_summary, second_usage) = analyze("import { second } from 'two'; second();");
    let second = ImportNames::collect(
        &second_summary,
        Some("import { second } from 'two'; second();"),
        &second_usage,
    )
    .values;
    assert_eq!(first.without_script_value_use, names(&["first"]));
    assert_eq!(second.without_script_value_use, names(&[]));
    assert_eq!(
        first
            .bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["first"]
    );
    assert_eq!(
        second
            .bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["second"]
    );
    let mut output = String::default();
    emit_setup_binding_anchors(
        &mut output,
        &first_summary,
        &first,
        &first_usage,
        None,
        &names(&["first"]),
        LABEL,
    );
    assert_eq!(
        output.as_str(),
        "\n  // Reference setup bindings used by template generation\n  void (null as unknown as keyof typeof first); void local;\n"
    );

    let empty = ImportNames::collect(&first_summary, None, &first_usage).values;
    assert_eq!(empty.bindings.len(), 0);
    assert_eq!(empty.without_script_value_use, names(&[]));
    let escaped_script = r"import { value as \u006cocal } from 'pkg';";
    let (escaped_summary, escaped_usage) = analyze(escaped_script);
    let escaped =
        ImportNames::collect(&escaped_summary, Some(escaped_script), &escaped_usage).values;
    // The existing collector filters raw authored spelling against decoded binding names.
    assert_eq!(escaped.bindings.len(), 0);
    assert_eq!(escaped.without_script_value_use, names(&[]));
}

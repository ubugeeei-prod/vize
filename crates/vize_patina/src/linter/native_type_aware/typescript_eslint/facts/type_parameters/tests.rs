use corsa::lint::run_default_type_aware_rule_owned;
use serde_json::Value;

use super::enrich;
use crate::linter::native_type_aware::typescript_eslint::tree;

#[test]
fn reports_a_single_use_and_keeps_a_relational_parameter() {
    let source = "function logValue<T>(value: T): void {}\nfunction identity<T>(value: T): T { return value; }\n";
    let mut collected = tree::collect(source, false, &["TSTypeParameterDeclaration"]);
    enrich(source, &mut collected.nodes, &collected.program);
    let reports = collected
        .nodes
        .into_iter()
        .flat_map(|node| {
            run_default_type_aware_rule_owned("no-unnecessary-type-parameters", node)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    assert_eq!(reports.len(), 1, "{reports:#?}");
    assert_eq!(
        reports[0].range.start as usize,
        source.find("logValue<T>").unwrap() + "logValue<".len()
    );
    assert_eq!(
        source.get(reports[0].range.start as usize..reports[0].range.end as usize),
        Some("T")
    );
}

#[test]
fn collects_nested_signatures_but_not_shadowed_type_parameters() {
    let source = concat!(
        "function wrap<T>(value: T): () => T { return () => value; }\n",
        "class Box<T> { get(value: T): T { return value; } }\n",
        "function outer<T>(value: T): <T>(inner: T) => T { return inner => inner; }\n",
    );
    let reports = report_ranges(source);
    assert_eq!(reports.len(), 1, "{reports:#?}");
    assert_eq!(
        reports[0].0,
        source.find("outer<T>").unwrap() + "outer<".len()
    );
}

#[test]
fn excludes_property_keys_and_mapped_type_binders() {
    let source = concat!(
        "function property<T>(value: { T: T }): void {}\n",
        "function mapped<T>(value: { [T in 'x']: T }): void {}\n",
        "function inferred<T>(value: T extends infer T ? T : never): void {}\n",
    );
    let reports = report_ranges(source);
    assert_eq!(reports.len(), 3, "{reports:#?}");
}

#[test]
fn preserves_type_parameters_that_flow_into_inferred_returns() {
    let source = concat!(
        "function identity<T>(value: T) { return value; }\n",
        "const arrow = <T>(value: T) => value;\n",
        "function logValue<T>(value: T) { console.info(value); }\n",
    );
    let reports = report_ranges(source);
    assert_eq!(reports.len(), 1, "{reports:#?}");
    assert_eq!(
        reports[0].0,
        source.find("logValue<T>").unwrap() + "logValue<".len()
    );
}

#[test]
fn recognizes_constructor_type_signatures() {
    let source = "type Factory = new <T>(value: T) => { value: T };\n";
    let mut collected = tree::collect(source, false, &["TSTypeParameterDeclaration"]);
    enrich(source, &mut collected.nodes, &collected.program);
    let declaration = collected.nodes.first().expect("type parameter declaration");
    assert_eq!(
        declaration.field_str("__ownerKind"),
        Some("TSConstructorType")
    );
    let parameter = declaration
        .child_lists
        .get("params")
        .and_then(|params| params.first())
        .expect("type parameter");
    assert_eq!(
        parameter
            .fields
            .get("__typeParameterUsageCount")
            .and_then(Value::as_u64),
        Some(3)
    );
}

#[test]
fn excludes_concise_arrow_function_bodies() {
    let source = "const log = <T>(value: T): void => void (value as T);\n";
    let reports = report_ranges(source);
    assert_eq!(reports.len(), 1, "{reports:#?}");
}

fn report_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut collected = tree::collect(source, false, &["TSTypeParameterDeclaration"]);
    enrich(source, &mut collected.nodes, &collected.program);
    collected
        .nodes
        .into_iter()
        .flat_map(|node| {
            run_default_type_aware_rule_owned("no-unnecessary-type-parameters", node)
                .unwrap_or_default()
        })
        .map(|report| (report.range.start as usize, report.range.end as usize))
        .collect()
}

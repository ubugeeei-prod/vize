use super::{
    CompactString, collect_line_module_spans, collect_named_value_export_starts,
    collect_named_value_exports,
};

#[test]
fn collect_import_span_includes_adjacent_ts_ignore_comment_group() {
    let script = "const before = 1;\n// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";\nconst after = 2;\n";
    let spans = collect_line_module_spans(script);

    assert_eq!(spans.len(), 1);
    assert_eq!(
        &script[spans[0].0 as usize..spans[0].1 as usize],
        "// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";"
    );
}

#[test]
fn collect_import_span_leaves_regular_comments_in_script_body() {
    let script = "// import note\nimport Chart from \"chart.js/auto/auto\";\n";
    let spans = collect_line_module_spans(script);

    assert_eq!(spans.len(), 1);
    assert_eq!(
        &script[spans[0].0 as usize..spans[0].1 as usize],
        "import Chart from \"chart.js/auto/auto\";"
    );
}

#[test]
fn exported_enum_is_lifted_to_module_scope_instead_of_the_value_bridge() {
    let script = "export enum DiffDisplayMode { Hidden = 'hidden' }\nexport type Props = {}\n";
    let spans = collect_line_module_spans(script);

    assert_eq!(spans.len(), 1);
    assert_eq!(
        &script[spans[0].0 as usize..spans[0].1 as usize],
        "export enum DiffDisplayMode { Hidden = 'hidden' }"
    );
    // The value bridge would re-export it value-only and erase its type side.
    assert!(collect_named_value_exports(script).is_empty());
}

#[test]
fn enum_that_cannot_leave_setup_scope_keeps_the_value_bridge() {
    let script = "const base = 1;\nexport enum Level { Low = base }\n";

    assert!(collect_line_module_spans(script).is_empty());
    assert_eq!(
        collect_named_value_exports(script),
        vec![CompactString::new("Level")]
    );
}

#[test]
fn named_value_export_starts_exclude_nested_enum_members() {
    let script = "enum Modes {\n  export = 'export',\n}\nexport const selected = Modes.export;\n";
    let starts = collect_named_value_export_starts(script);

    assert_eq!(starts.len(), 1);
    assert!(starts.contains(&(script.find("export const").unwrap() as u32)));
    assert!(!starts.contains(&(script.find("export =").unwrap() as u32)));
}

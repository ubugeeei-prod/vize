use super::MacroTypeMappings;
use crate::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use std::ops::Range;

fn model_rows(script: Option<&str>, ts: &str, name: &str) -> Vec<(Range<usize>, Range<usize>)> {
    let mut rows = Vec::new();
    let offset = |start| start + 100;
    MacroTypeMappings::new(&mut rows, script, &offset).map_model_symbol(
        ts,
        10..ts.len(),
        (0, script.map_or(10, |text| text.len() as u32)),
        name,
    );
    rows.into_iter()
        .map(|row| {
            assert_eq!(row.sub_spans, vec![]);
            (row.gen_range, row.src_range)
        })
        .collect()
}

#[test]
fn model_literal_and_update_endpoints_own_only_the_retained_name() {
    for authored in ["'saveItem'", "\"saveItem\"", "`saveItem`"] {
        assert_eq!(
            model_rows(Some(authored), "prefix(); \"saveItem\"", "saveItem"),
            vec![(10..20, 101..109), (11..19, 101..109)]
        );
        assert_eq!(
            model_rows(Some(authored), "prefix(); \"update:saveItem\"", "saveItem"),
            vec![(10..27, 101..109), (11..26, 101..109), (18..26, 101..109)]
        );
    }
    assert_eq!(
        model_rows(Some("'save-item'"), "prefix(); \"save-item\"", "save-item"),
        vec![(10..21, 101..110), (11..20, 101..110)]
    );
    assert_eq!(
        model_rows(
            Some("'save-item'"),
            "prefix(); \"update:save-item\"",
            "save-item"
        ),
        vec![(10..28, 101..110), (11..27, 101..110), (18..27, 101..110)]
    );
}

#[test]
fn unverified_names_preserve_the_original_whole_ast_mapping() {
    for (authored, generated, name, expected) in [
        (
            None,
            "prefix(); \"saveItem\"",
            "saveItem",
            (10..20, 100..110),
        ),
        (
            Some("defineModel()"),
            "prefix(); \"update:modelValue\"",
            "modelValue",
            (10..29, 100..113),
        ),
        (
            Some("'save\\u0049tem'"),
            "prefix(); \"saveItem\"",
            "saveItem",
            (10..20, 100..115),
        ),
        (
            Some("'other'"),
            "prefix(); \"saveItem\"",
            "saveItem",
            (10..20, 100..107),
        ),
        (
            Some("'saveItem'"),
            "prefix(); \"delete:saveItem\"",
            "saveItem",
            (10..27, 100..110),
        ),
        (
            Some("'saveItem'"),
            "prefix(); saveItem",
            "saveItem",
            (10..18, 100..110),
        ),
        (Some("''"), "prefix(); \"update:\"", "", (10..19, 100..102)),
    ] {
        assert_eq!(model_rows(authored, generated, name), vec![expected]);
    }
}

#[test]
fn generated_model_prop_and_event_rows_retain_authored_ast_contents() {
    for script in [
        "defineModel<string>('saveItem', { default: '' });",
        "defineModel<string>(\"saveItem\", { default: '' });",
    ] {
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, "<span />");
        let mut analyzer =
            vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script),
            Some(&root),
            100,
            0,
            &VirtualTsOptions::default(),
        );
        let prop_start = output
            .code
            .find("\"saveItem\"?: string;")
            .expect("model prop");
        let event_start = output
            .code
            .find("/* __vize_model_event */ \"update:saveItem\"")
            .expect("model event")
            + "/* __vize_model_event */ ".len();
        let rows: Vec<_> = output
            .mapping
            .spans()
            .iter()
            .filter(|row| {
                (prop_start..prop_start + 10).contains(&row.gen_range.start)
                    || (event_start..event_start + 17).contains(&row.gen_range.start)
            })
            .map(|row| (row.gen_range.clone(), row.src_range.clone()))
            .collect();
        assert_eq!(
            rows,
            vec![
                (prop_start..prop_start + 10, 121..129),
                (prop_start + 1..prop_start + 9, 121..129),
                (event_start..event_start + 17, 121..129),
                (event_start + 1..event_start + 16, 121..129),
                (event_start + 8..event_start + 16, 121..129),
            ]
        );
    }
}

//! Computed component keys preserve existing authored mapping units.

use super::{Lane, mapped_on};

#[test]
fn computed_component_keys_match_every_retained_mapping_segment() {
    for (case, source) in [
        (
            "reference",
            r#"<Child :[propName]="value" @[eventName]="record" />"#,
        ),
        (
            "indexed",
            r#"<Child :[names[selected]]="value" @[events[selected]]="record(value)" />"#,
        ),
        (
            "dynamic",
            r#"<component :is="view" :[is]="value" @[eventName]="record" />"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            (code.clone(), segments.clone()),
            mapped_on(source, Lane::Legacy),
            "{case}: generated code and every decoded segment"
        );
        insta::assert_snapshot!(format!("component_names_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("component_names_{case}_map"), segments);
    }
}

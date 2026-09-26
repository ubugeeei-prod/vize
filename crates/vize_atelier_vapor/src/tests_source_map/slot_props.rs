//! Computed outlet props preserve existing selector, fallback and render anchors.

use super::{Lane, mapped_on};

#[test]
fn computed_slot_props_match_every_retained_mapping_segment() {
    for (case, source) in [
        (
            "reference",
            r#"<slot title="fixed" :[name]="value"><b>fallback</b></slot>"#,
        ),
        (
            "indexed",
            r#"<slot name="header" data-id="fixed" :[keys[index]]="value" :extra="extra"><b>{{ fallback }}</b></slot>"#,
        ),
        (
            "selector",
            r#"<slot :name="selected" :[name]="value"><b>{{ fallback }}</b></slot>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            (code.clone(), segments.clone()),
            mapped_on(source, Lane::Legacy),
            "{case}: generated code and every decoded segment"
        );
        insta::assert_snapshot!(format!("slot_props_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("slot_props_{case}_map"), segments);
    }
}

//! Ordered computed DOM sources preserve every existing mapping segment.

use super::{Lane, mapped_on};

#[test]
fn computed_dom_keys_match_every_retained_mapping_segment() {
    for (case, source) in [
        (
            "reference",
            r#"<div title="fixed" :[name]="value">text</div>"#,
        ),
        (
            "indexed",
            r#"<div class="base" :class="classes" :[keys[index]]="value"></div>"#,
        ),
        (
            "object",
            r#"<div :[name.toLowerCase()]="value" v-bind="attrs" :['data-'+suffix]="extra"></div>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            (code.clone(), segments.clone()),
            mapped_on(source, Lane::Legacy),
            "{case}: generated code and every decoded segment"
        );
        insta::assert_snapshot!(format!("computed_dom_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("computed_dom_{case}_map"), segments);
    }
}

//! Computed models preserve each existing component mapping unit.

use super::{Lane, mapped_on};

#[test]
fn computed_component_models_match_every_retained_mapping_segment() {
    for (case, source) in [
        ("reference", r#"<Child v-model:[name]="value" />"#),
        (
            "indexed",
            r#"<Child v-model:[names[index]].trim="form[field]" />"#,
        ),
        (
            "dynamic",
            r#"<component :is="view" v-model:[name.toLowerCase()].number="form.title" />"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            (code.clone(), segments.clone()),
            mapped_on(source, Lane::Legacy),
            "{case}: generated code and every decoded segment"
        );
        insta::assert_snapshot!(format!("model_names_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("model_names_{case}_map"), segments);
    }
}

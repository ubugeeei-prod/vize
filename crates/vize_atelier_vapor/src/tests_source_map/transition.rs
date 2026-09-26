//! The wrapper, slot root, condition and loop aliases preserve exact maps.

use super::support::{Lane, mapped_on};

#[test]
fn transition_roots_map_equally_on_native_and_retained_lanes() {
    for (case, source) in [
        (
            "conditional",
            r#"<main><Transition :css="false" @enter="enter"><button v-if="show" @click="send">{{ label }}</button></Transition><i>tail</i></main>"#,
        ),
        (
            "group",
            r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in items" :key="item.id" :data-id="item.id">{{ item.text }}</li></TransitionGroup>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            mapped_on(source, Lane::Legacy),
            (code.clone(), segments.clone()),
            "{case}"
        );
        insta::assert_snapshot!(format!("transition_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("transition_{case}_map"), segments);
    }
}

#[test]
fn nested_transition_maps_pin_both_reviewed_numbering_policies() {
    for (case, source) in [
        (
            "nested_static",
            r#"<Transition :css="false"><div><span>{{ label }}</span></div></Transition>"#,
        ),
        (
            "nested_conditional",
            r#"<main><Transition :css="false" @enter="enter"><div v-if="show"><button @click="send">{{ label }}</button></div></Transition><i>tail</i></main>"#,
        ),
        (
            "nested_group",
            r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in items" :key="item.id"><span>{{ item.text }}</span></li></TransitionGroup>"#,
        ),
    ] {
        let (native, native_segments) = mapped_on(source, Lane::Selected);
        let (retained, retained_segments) = mapped_on(source, Lane::Legacy);
        assert_eq!(
            crate::tests_generated_identity::normalized(&native),
            crate::tests_generated_identity::normalized(&retained),
            "{case}"
        );
        insta::assert_snapshot!(format!("transition_{case}_native_code"), native);
        insta::assert_debug_snapshot!(format!("transition_{case}_native_map"), native_segments);
        insta::assert_snapshot!(format!("transition_{case}_retained_code"), retained);
        insta::assert_debug_snapshot!(format!("transition_{case}_retained_map"), retained_segments);
    }
}

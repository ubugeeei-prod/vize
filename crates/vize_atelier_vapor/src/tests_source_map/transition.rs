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

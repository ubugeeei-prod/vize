//! The renderer primitive and ordinary slots keep exact authored mappings.

use super::support::{Lane, mapped_on};

#[test]
fn suspense_default_and_fallback_map_equally_on_both_lanes() {
    for (case, source) in [
        (
            "root",
            r#"<Suspense><template #default><AsyncChild :label="label" /></template><template #fallback><p>{{ waiting }}</p></template></Suspense>"#,
        ),
        (
            "nested",
            r#"<main><Suspense v-if="visible" :timeout="limit"><template #default><AsyncChild /></template><template #fallback><p>{{ waiting }}</p></template></Suspense><i>tail</i></main>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            mapped_on(source, Lane::Legacy),
            (code.clone(), segments.clone()),
            "{case}"
        );
        insta::assert_snapshot!(format!("suspense_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("suspense_{case}_map"), segments);
    }
}

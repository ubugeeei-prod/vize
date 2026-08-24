//! P2-11 named / scoped slot witness: `<template #name>` groups,
//! component-root `v-slot` (shipped keys that `default`), dynamic
//! names, simple scoped params, and `createSlots` (`v-if` / `v-for`
//! slot templates), compared **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    (
        "named_header_text",
        "<Foo><template #header>title</template></Foo>",
    ),
    (
        "named_header_interp",
        "<Foo><template #header>hello {{ msg }}</template></Foo>",
    ),
    (
        "named_then_default",
        "<Foo><template #header>title</template>hello</Foo>",
    ),
    (
        "default_then_named",
        "<Foo>hello<template #header>title</template></Foo>",
    ),
    (
        "named_header_span",
        "<Foo><template #header><span></span></template></Foo>",
    ),
    (
        "hyphenated_slot",
        "<Foo><template #foo-bar>x</template></Foo>",
    ),
    (
        "two_named",
        "<Foo><template #header>title</template><template #footer>end</template></Foo>",
    ),
    ("empty_named", "<Foo><template #header></template></Foo>"),
    ("ws_named", "<Foo><template #header>  </template></Foo>"),
    ("component_v_slot_header", "<Foo v-slot:header>title</Foo>"),
    ("component_v_slot", "<Foo v-slot>title</Foo>"),
    ("component_hash_header", "<Foo #header>title</Foo>"),
    (
        "dynamic_slot_name",
        r#"<Foo><template #[name]>x</template></Foo>"#,
    ),
    (
        "scoped_ident",
        r#"<Foo><template #header="p">x</template></Foo>"#,
    ),
    (
        "scoped_destructure",
        r#"<Foo><template #header="{ foo }">x</template></Foo>"#,
    ),
    (
        "create_slots_if",
        r#"<Foo><template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_for",
        r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#,
    ),
    (
        "create_slots_if_and_static",
        r#"<Foo><template #header v-if="ok">x</template><template #footer>end</template></Foo>"#,
    ),
    (
        "create_slots_default_and_if",
        r#"<Foo>hello<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_else",
        r#"<Foo><template #header v-if="a">x</template><template #header v-else>y</template></Foo>"#,
    ),
    (
        "create_slots_if_else_if",
        r#"<Foo><template #header v-if="a">x</template><template #header v-else-if="b">y</template></Foo>"#,
    ),
    (
        "create_slots_if_span",
        r#"<Foo><template #header v-if="ok"><span></span></template></Foo>"#,
    ),
    (
        "create_slots_scoped",
        r#"<Foo><template #header="p" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_dynamic_name",
        r#"<Foo><template #[name] v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_hyphenated",
        r#"<Foo><template #foo-bar v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_empty",
        r#"<Foo><template #header v-if="ok"></template></Foo>"#,
    ),
    (
        "create_slots_for_aliases",
        r#"<Foo><template v-for="(v, k, i) in n" #header>x</template></Foo>"#,
    ),
    (
        "create_slots_default_interp",
        r#"<Foo>hello {{ msg }}<template #header v-if="ok">x</template></Foo>"#,
    ),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_named_slots_match_the_shipped_dom_lane_byte_for_byte() {
    let mut compared = 0u64;
    let mut skipped_legacy_flag = 0u64;
    if std::env::var(DOM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        skipped_legacy_flag += 1;
    } else {
        let allocator = Allocator::new();
        for (name, src) in BATTERY {
            let old = shipped(src);
            let new = emit_dom_source(&allocator, src)
                .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
                .assembled();
            assert_eq!(
                old.as_str(),
                new.as_str(),
                "{name}: S2 DOM emit diverged from the shipped lane"
            );
            compared += 1;
        }
    }
    assert_eq!(
        (compared, skipped_legacy_flag),
        (BATTERY.len() as u64, 0),
        "a cfg or {DOM_LANE_FLAG}=legacy regression disarmed the dual-run"
    );
}

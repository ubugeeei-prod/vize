//! P2-11 `<template v-if>` / `<template v-for>` fragment witness,
//! compared **byte-for-byte** including helper usage and hoists.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "vif_two_span",
        r#"<template v-if="ok"><span></span><span></span></template>"#,
    ),
    (
        "vif_one_span",
        r#"<template v-if="ok"><span></span></template>"#,
    ),
    ("vif_interp", r#"<template v-if="ok">{{ msg }}</template>"#),
    ("vif_text", r#"<template v-if="ok">hello</template>"#),
    (
        "vif_compound",
        r#"<template v-if="ok">hello {{ msg }}</template>"#,
    ),
    (
        "vif_key",
        r#"<template v-if="ok" key="k"><span></span><p></p></template>"#,
    ),
    (
        "vif_else",
        r#"<template v-if="ok"><span>a</span><span>b</span></template><div v-else>no</div>"#,
    ),
    (
        "vif_nested",
        r#"<div><template v-if="ok"><span></span><p></p></template></div>"#,
    ),
    (
        "vif_comp",
        r#"<template v-if="ok"><Foo /><Bar /></template>"#,
    ),
    (
        "vif_text_span",
        r#"<template v-if="ok">hello<span></span></template>"#,
    ),
    (
        "vif_unwrap_dyn",
        r#"<template v-if="ok"><span>{{ msg }}</span></template>"#,
    ),
    (
        "vif_unwrap_comp",
        r#"<template v-if="ok"><Foo /></template>"#,
    ),
    ("empty_tpl_vif", r#"<template v-if="ok"></template>"#),
    (
        "vif_for",
        r#"<template v-if="ok"><div v-for="i in n">{{ i }}</div></template>"#,
    ),
    (
        "vfor_two",
        r#"<template v-for="item in list"><span></span><span></span></template>"#,
    ),
    (
        "vfor_keyed",
        r#"<template v-for="item in list" :key="item"><span></span><span></span></template>"#,
    ),
    (
        "vfor_one",
        r#"<template v-for="item in list"><span></span></template>"#,
    ),
    (
        "vfor_one_keyed",
        r#"<template v-for="item in list" :key="item"><span>{{ item }}</span></template>"#,
    ),
    (
        "vfor_numeric",
        r#"<template v-for="n in 3"><span></span><span></span></template>"#,
    ),
    (
        "vfor_interp",
        r#"<template v-for="item in list">{{ item }}</template>"#,
    ),
    (
        "vfor_unwrap_dyn",
        r#"<template v-for="item in list"><span>{{ item }}</span></template>"#,
    ),
];

#[test]
fn s2_template_wrapper_fragments_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}

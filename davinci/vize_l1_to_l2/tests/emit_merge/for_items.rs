use super::*;

// Full live development oracle, including helper order and complete render body.
// The unchanged 3 whole Theme SFCs remain in the mandatory inline/module corpus.
#[test]
fn lone_for_item_spreads_keep_complete_shipped_output() {
    for source in [
        r#"<div><button v-for="$attrs in items" v-bind="$attrs" /></div>"#,
        r#"<div v-for="item in items" v-bind="$attrs" />"#,
        r#"<div v-for="$attrs in items" v-bind="$attrs" />"#,
        r#"<template v-for="item in items"><button v-bind="obj" /></template>"#,
        r#"<div v-for="item in 3" v-bind="obj" />"#,
        r#"<Foo v-for="item in items" v-bind="obj" />"#,
        r#"<div v-for="item in items" v-on="handlers" />"#,
        r#"<Foo v-for="item in items" v-on="handlers" />"#,
    ] {
        assert_shipped_parity(source);
    }
}

// Loop scope alone must not change an ordinary child's lone-spread selection.
#[test]
fn ordinary_children_of_for_items_keep_complete_shipped_output() {
    for source in [
        r#"<div v-for="item in items"><button v-bind="obj" /></div>"#,
        r#"<template v-for="item in items"><div><button v-bind="obj" /></div></template>"#,
    ] {
        assert_shipped_parity(source);
    }
}

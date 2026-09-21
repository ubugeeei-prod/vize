//! Native S3 object/range loops and root-level control flow, with written-out
//! expectations checked against both published runtimes.

use super::native_control::element;
use crate::assert_backends;
use serde_json::{Value, json};

#[test]
fn native_object_and_range_loops_render_in_order() {
    let span = |name: &str, position: &str, value: &str| {
        element(
            "span",
            json!({"data-name": name, "title": position}),
            json!([value]),
        )
    };
    let range: std::vec::Vec<_> = ["1", "2", "3"]
        .iter()
        .map(|n| element("b", json!({}), json!([n])))
        .collect();
    let main = |mut children: std::vec::Vec<Value>| {
        children.extend(range.iter().cloned());
        json!({"tree": [element("main", json!({}), Value::Array(children))], "events": []})
    };
    let actual = assert_backends(
        // Upstream beta.10 skips key/index refresh on unkeyed updates (rc.6 fixes
        // it in `updateAt`); keyed moves observe the compiler contract here.
        r#"<main><span v-for="(value, name, position) in entries" :key="name" :data-name="name" :title="position">{{ value }}</span><b v-for="n in 3">{{ n }}</b></main>"#,
        json!({"entries": {"x": "X", "y": "Y"}}),
        json!([{"patch": {"entries": {"x": "X2", "z": "Z"}}}]),
    );
    assert_eq!(
        actual,
        json!([
            main(vec![span("x", "0", "X"), span("y", "1", "Y")]),
            main(vec![span("x", "0", "X2"), span("z", "1", "Z")]),
            {"tree": [], "events": []}
        ])
    );
}

#[test]
fn native_root_branch_chains_and_loops_mount_as_fragments() {
    let view = |root: Value| json!({"tree": [root], "events": []});
    let actual = assert_backends(
        r#"<b v-if="a" :title="label">{{ label }}</b><i v-else-if="b">B</i><span v-else>none</span>"#,
        json!({"a": true, "b": false, "label": "L"}),
        json!([{"patch": {"a": false}}, {"patch": {"b": true}}, {"patch": {"a": true, "label": "M"}}]),
    );
    assert_eq!(
        actual,
        json!([
            view(element("b", json!({"title": "L"}), json!(["L"]))),
            view(element("span", json!({}), json!(["none"]))),
            view(element("i", json!({}), json!(["B"]))),
            view(element("b", json!({"title": "M"}), json!(["M"]))),
            {"tree": [], "events": []}
        ])
    );

    let items = |labels: &[&str]| {
        let items: std::vec::Vec<_> = labels
            .iter()
            .map(|label| element("b", json!({}), json!([label])))
            .collect();
        json!({"tree": items, "events": []})
    };
    let actual = assert_backends(
        r#"<b v-for="item in items" :key="item">{{ item }}</b>"#,
        json!({"items": ["p", "q"]}),
        json!([{"patch": {"items": ["q", "r"]}}, {"patch": {"items": []}}]),
    );
    assert_eq!(
        actual,
        json!([items(&["p", "q"]), items(&["q", "r"]), items(&[]), {"tree": [], "events": []}])
    );
}

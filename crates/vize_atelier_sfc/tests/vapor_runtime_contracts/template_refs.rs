//! #7882 loop-native refs, with unchanged ordinary scalar refs.

use super::super::{Value, json, trace};

const UPSTREAM: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/upstream.vue.txt"
);
const REMOVAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/removal.vue.txt"
);
const CALLBACK: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/callback.vue.txt"
);
const SINGLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/single.vue.txt"
);
const NESTED: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/nested.vue.txt"
);
const LOOP_ONLY: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/loop-only.vue.txt"
);

fn element(tag: &str, attributes: Value, children: Value) -> Value {
    json!({"tag": tag, "attributes": attributes, "children": children})
}

fn snapshot(tree: Value, refs: Value, elements: usize) -> Value {
    json!({"tree": tree, "namespaces": vec!["http://www.w3.org/1999/xhtml"; elements],
        "events": [], "exposed": {"referencedText": refs}})
}

fn list(items: &[&str]) -> Value {
    let children: Vec<Value> = items
        .iter()
        .map(|item| element("li", json!({"data-plant": "ref-item"}), json!([item])))
        .collect();
    snapshot(
        json!([element("ul", json!({}), json!(children))]),
        json!(items.join(",")),
        items.len() + 1,
    )
}

fn assert_modes(
    name: &str,
    source: &str,
    steps: Value,
    expected: Value,
    readers: bool,
    backends: &[&str],
) {
    for &backend in backends {
        for production in [false, true] {
            for separate in [false, true] {
                if !production && !separate {
                    continue;
                }
                let mut request = json!({
                    "production": production, "separateTemplate": separate,
                    "proofName": format!("{name}-{backend}-{production}-{separate}"),
                    "steps": steps
                });
                if readers {
                    request["exposedReads"] = json!(["referencedText"]);
                }
                assert_eq!(
                    trace(source, backend, request),
                    expected,
                    "{name}/{backend}/production={production}/separate={separate}"
                );
            }
        }
    }
}

#[test]
fn upstream_loop_ref_arrays_mount_and_append() {
    assert_modes(
        "upstream",
        UPSTREAM,
        json!([{"call": "add"}]),
        json!([
            list(&["a", "b"]),
            list(&["a", "b", "c"]),
            snapshot(json!([]), json!(""), 0)
        ]),
        true,
        &["vdom", "vapor"],
    );
}

#[test]
fn loop_ref_arrays_remove_and_clear_without_stale_nodes() {
    assert_modes(
        "removal",
        REMOVAL,
        json!([{"call": "add"}, {"call": "remove"}, {"call": "clear"}]),
        json!([
            list(&["a", "b"]),
            list(&["a", "b", "c"]),
            list(&["b", "c"]),
            list(&[]),
            snapshot(json!([]), json!(""), 0)
        ]),
        true,
        &["vdom", "vapor"],
    );
}

#[test]
fn callback_loop_refs_receive_actual_nodes_and_cleanup() {
    assert_modes(
        "callback",
        CALLBACK,
        json!([{"call": "add"}]),
        json!([
            list(&["a", "b"]),
            list(&["a", "b", "c"]),
            snapshot(json!([]), json!(""), 0)
        ]),
        true,
        &["vdom", "vapor"],
    );
}

#[test]
fn loop_free_refs_remain_scalar_and_clear_on_unmount() {
    assert_modes(
        "single",
        SINGLE,
        json!([]),
        json!([
            snapshot(
                json!([element("p", json!({}), json!(["single"]))]),
                json!("single"),
                1
            ),
            snapshot(json!([]), Value::Null, 0)
        ]),
        true,
        &["vdom", "vapor"],
    );
}

#[test]
fn transparent_nested_loop_refs_keep_all_descendant_nodes() {
    let nested = |items: &[&str]| {
        let children: Vec<Value> = items
            .iter()
            .map(|item| element("span", json!({}), json!([item])))
            .collect();
        let mut children = children;
        children.push(element("aside", json!({}), json!(["single"])));
        snapshot(
            json!([element("main", json!({}), json!(children))]),
            json!(format!("{}|single", items.join(","))),
            items.len() + 2,
        )
    };
    assert_modes(
        "nested",
        NESTED,
        json!([{"call": "add"}, {"call": "clear"}]),
        json!([
            nested(&["a", "b", "c"]),
            nested(&["a", "b", "c", "d"]),
            nested(&[]),
            snapshot(json!([]), json!("|null"), 0)
        ]),
        true,
        &["vdom", "vapor"],
    );
}

#[test]
fn original_numeric_loop_clause_has_three_refs_after_mount() {
    let tree = json!([
        element("span", json!({}), json!(["1"])),
        element("span", json!({}), json!(["2"])),
        element("span", json!({}), json!(["3"])),
        element("p", json!({}), json!(["3"]))
    ]);
    assert_modes(
        "numeric-loop",
        LOOP_ONLY,
        json!([]),
        json!([
            {"tree": tree, "namespaces": vec!["http://www.w3.org/1999/xhtml"; 4], "events": []},
            {"tree": [], "namespaces": [], "events": []}
        ]),
        false,
        &["vapor"],
    );
}

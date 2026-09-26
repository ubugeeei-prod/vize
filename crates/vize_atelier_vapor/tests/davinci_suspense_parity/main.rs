//! Real async setup, fallback, resolution and pending teardown under rc.9.
//! Retained compilation is isolated from the native-only walk probe binaries.

#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "tests assert by panicking and compare std-string fixtures"
)]

mod trace;

use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

const SOURCE: &str = r#"<main data-id="root"><Suspense v-if="visible" :timeout="limit" @pending="record('pending')" @fallback="record('fallback')" @resolve="record('resolve')"><template #default><AsyncChild :label="label" @send="record" /></template><template #fallback><p data-id="fallback">{{ waiting }}</p></template></Suspense><i data-id="tail">tail</i></main>"#;
const CHILD: &str =
    r#"<button data-id="resolved" @click="send('send', label)">{{ label }}</button>"#;

fn view(branch: Option<(&str, &str, u32)>, events: Value) -> Value {
    let mut children = vec![];
    let mut identities = vec![json!(["root", 0])];
    if let Some((name, text, id)) = branch {
        let mut child = json!({
            "tag": if name == "resolved" { "button" } else { "p" },
            "attributes": {"data-id": name}, "children": [text],
        });
        if name == "resolved" {
            child
                .as_object_mut()
                .unwrap()
                .insert("disabled".to_owned(), json!(false));
        }
        children.push(child);
        identities.push(json!([name, id]));
    }
    children.push(json!({"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]}));
    identities.push(json!(["tail", 2]));
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": children}],
        "events": events, "identities": identities,
    })
}

fn empty(events: Value) -> Value {
    json!({"tree": [], "events": events, "identities": []})
}

fn expected(scenario: &str) -> Vec<Value> {
    let pending = json!(["setup", "pending", "fallback"]);
    let mut frames = vec![
        view(Some(("fallback", "waiting", 1)), pending.clone()),
        view(Some(("fallback", "still waiting", 1)), pending),
    ];
    match scenario {
        "resolve" => {
            frames.push(view(
                Some(("resolved", "B", 3)),
                json!(["setup", "pending", "fallback", "resolve", "mounted",]),
            ));
            frames.push(view(
                Some(("resolved", "C", 3)),
                json!(["setup", "pending", "fallback", "resolve", "mounted", "C",]),
            ));
            let removed = json!([
                "setup",
                "pending",
                "fallback",
                "resolve",
                "mounted",
                "C",
                "disposed",
                "unmounted",
            ]);
            frames.push(view(None, removed.clone()));
            frames.push(empty(removed));
        }
        "remove-pending" => {
            let removed = json!(["setup", "pending", "fallback", "disposed", "unmounted"]);
            frames.push(view(None, removed.clone()));
            frames.push(view(None, removed.clone()));
            frames.push(empty(removed));
        }
        "unmount-pending" => {
            let removed = json!(["setup", "pending", "fallback", "disposed", "unmounted"]);
            frames.push(empty(removed.clone()));
            frames.push(empty(removed));
        }
        _ => panic!("unknown scenario"),
    }
    frames
}

#[test]
fn async_suspense_lifetimes_match_native_retained_and_official_vapor() {
    let allocator = Allocator::new();
    let options = VaporCompilerOptions {
        prefix_identifiers: true,
        ..Default::default()
    };
    let before = WalkCounts::snapshot();
    let parent = compile_vapor(&allocator, SOURCE, options.clone());
    let child = compile_vapor(&allocator, CHILD, options.clone());
    for result in [&parent, &child] {
        assert!(
            result.error_messages.is_empty(),
            "{:?}",
            result.error_messages
        );
    }
    assert_eq!(
        WalkCounts::snapshot().since(before).total_walks(),
        0,
        "native S3"
    );
    let retained = compile_vapor(
        &allocator,
        SOURCE,
        VaporCompilerOptions {
            davinci_retained_lane: true,
            ..options
        },
    );
    assert!(
        retained.error_messages.is_empty(),
        "{:?}",
        retained.error_messages
    );
    for scenario in ["resolve", "remove-pending", "unmount-pending"] {
        for (lane, code) in [("native", &parent.code), ("retained", &retained.code)] {
            let result =
                trace::trace(json!({"code": code, "childCode": child.code, "scenario": scenario}));
            assert_eq!(result, expected(scenario), "{scenario}: {lane}");
        }
        let official =
            trace::trace(json!({"source": SOURCE, "childSource": CHILD, "scenario": scenario}));
        assert_eq!(official, expected(scenario), "{scenario}: official");
    }
}

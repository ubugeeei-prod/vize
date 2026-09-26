//! Published rc.9 JS hooks, controlled completion and keyed identities.
//! Retained compilation lives outside all native-only process-global probes.

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
use vize_atelier_core::{expr_parse_probe, walk_probe::WalkCounts};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, cstr};

const HOOKS: &str = r#":css="false" @before-enter="beforeEnter" @enter="enter" @after-enter="afterEnter" @enter-cancelled="enterCancelled" @before-leave="beforeLeave" @leave="leave" @after-leave="afterLeave" @leave-cancelled="leaveCancelled""#;

#[test]
fn controlled_transition_hooks_match_native_retained_and_official_runtime() {
    for (scenario, wrapper, child, golden) in [
        (
            "static",
            "Transition",
            r#"<button data-id="child" @click="send">{{ label }}</button>"#,
            include_str!("__snapshots__/static.json"),
        ),
        (
            "single",
            "Transition",
            r#"<button v-if="show" data-id="child" @click="send">{{ label }}</button>"#,
            include_str!("__snapshots__/single.json"),
        ),
        (
            "cancel",
            "Transition",
            r#"<button v-if="show" data-id="child" @click="send">{{ label }}</button>"#,
            include_str!("__snapshots__/cancel.json"),
        ),
        (
            "group",
            "TransitionGroup",
            r#"<li v-for="item in items" :key="item.id" :data-id="item.id">{{ item.text }}</li>"#,
            include_str!("__snapshots__/group.json"),
        ),
        (
            "group-div",
            "TransitionGroup",
            r#"<li v-for="item in items" :key="item.id" :data-id="item.id">{{ item.text }}</li>"#,
            include_str!("__snapshots__/group-div.json"),
        ),
    ] {
        let tag = if scenario == "group-div" {
            r#"tag="div" "#
        } else if scenario == "group" {
            r#"tag="ul" "#
        } else {
            ""
        };
        let source = cstr!(
            "<main data-id=\"root\"><{wrapper} {tag}{HOOKS}>{child}</{wrapper}><i data-id=\"tail\">tail</i></main>"
        );
        let expected: Vec<Value> = serde_json::from_str(golden).unwrap();
        let official = trace::trace(json!({"source": source, "scenario": scenario}));
        assert_eq!(
            official, expected,
            "{scenario}: official rc.9 contract changed"
        );
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let before = WalkCounts::snapshot();
        let parses = expr_parse_probe::expr_parse_count();
        let native = compile_vapor(&allocator, &source, options.clone());
        assert!(
            native.error_messages.is_empty(),
            "{:?}",
            native.error_messages
        );
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "native {scenario}"
        );
        assert_eq!(
            expr_parse_probe::expr_parse_count() - parses,
            0,
            "native {scenario}"
        );
        let retained = compile_vapor(
            &allocator,
            &source,
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
        for (lane, code) in [("native", native.code), ("retained", retained.code)] {
            let result = trace::trace(json!({"code": code, "scenario": scenario}));
            assert_eq!(result, expected, "{scenario}: {lane}");
        }
    }
}

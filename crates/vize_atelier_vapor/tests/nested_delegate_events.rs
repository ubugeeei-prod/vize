#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn registrations(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result
        .code
        .lines()
        .filter(|line| line.starts_with("_delegateEvents("))
        .map(str::to_owned)
        .collect()
}

#[test]
fn delegated_events_are_registered_from_every_nested_emitted_block() {
    for source in [
        r#"<button v-if="ready" @click="save">save</button>"#,
        r#"<button v-if="ready">ready</button><button v-else @click="save">save</button>"#,
        r#"<button v-for="item in items" :key="item" @click="save">{{ item }}</button>"#,
        r#"<slot name="body"><button @click="save">save</button></slot>"#,
        r#"<Widget><button @click="save">save</button></Widget>"#,
        r#"<Widget><template #body><button @click="save">save</button></template></Widget>"#,
        r#"<section v-if="ready"><button v-for="item in items" :key="item" @click="save">{{ item }}</button></section>"#,
    ] {
        assert_eq!(
            registrations(source),
            [r#"_delegateEvents("click")"#],
            "{source}"
        );
    }
}

#[test]
fn nested_registrations_are_deduplicated_and_sorted_with_root_events() {
    assert_eq!(
        registrations(
            r#"<button @click="save">root</button><button v-if="ready" @keydown="save">ready</button><button v-else @click="save">save</button><slot><button @keydown="save">slot</button></slot>"#
        ),
        [
            r#"_delegateEvents("click")"#,
            r#"_delegateEvents("keydown")"#
        ]
    );
}

#[test]
fn non_delegated_and_dynamic_nested_events_keep_the_direct_listener_path() {
    for event in [
        "click.once",
        "click.capture",
        "click.passive",
        "focus",
        "[event]",
    ] {
        let source = format!(r#"<button v-if="ready" @{event}="save">save</button>"#);
        assert_eq!(registrations(&source), Vec::<String>::new(), "{source}");
    }
}

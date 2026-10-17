//! Shared parent guards must not erase the exclusions of a nested branch chain.
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::expect_used, reason = "the authored fixtures must be readable")]

#[path = "support/script_block_project.rs"]
mod project;

fn fixture(file: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/nested-branch-chain");
    std::fs::read_to_string(root.join(format!("{file}.txt")))
        .expect("the authored fixture should be readable")
}

fn check(child: &str) -> Vec<String> {
    project::check(&[
        ("src/Command.vue", &fixture("Command.vue")),
        ("src/Notification.vue", &fixture("Notification.vue")),
        ("src/User.vue", &fixture("User.vue")),
        ("src/Child.vue", child),
        ("src/Paginator.vue", &fixture("Paginator.vue")),
        ("src/types.ts", &fixture("types.ts")),
    ])
}

#[test]
fn nested_else_keeps_v_for_slot_and_nullable_prop_narrowing() {
    assert_eq!(check(&fixture("Child.vue")), Vec::<String>::new());
}

#[test]
fn nested_else_keeps_real_prop_type_errors() {
    // Complete rows from the same fixtures checked by the Vue TypeScript oracle.
    for (before, after, expected) in [
        (
            "cmd?: string",
            "cmd?: number",
            vec![
                "src/Command.vue(12,24): error TS2322: Type 'string' is not assignable to type 'number'.",
            ],
        ),
        (
            "notification?: Notification",
            "notification?: number",
            vec![
                "src/Notification.vue(12,24): error TS2322: Type 'Notification' is not assignable to type 'number'.",
                "src/Notification.vue(17,24): error TS2322: Type 'Notification' is not assignable to type 'number'.",
            ],
        ),
        (
            "user?: User",
            "user?: number",
            vec![
                "src/User.vue(9,30): error TS2322: Type 'User' is not assignable to type 'number'.",
            ],
        ),
    ] {
        let child = fixture("Child.vue").replace(before, after);
        assert_eq!(check(&child), expected, "{before} -> {after}");
    }
}

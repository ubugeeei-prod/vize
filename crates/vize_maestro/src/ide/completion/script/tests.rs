use super::infer_reactive_value_type;
use vize_croquis::reactivity::ReactiveKind;

fn infer(script: &str, name: &str, kind: ReactiveKind) -> Option<String> {
    infer_reactive_value_type(script, name, kind)
}

#[test]
fn pins_angle_and_paren_inference_behavior() {
    // Angle-bracket generic form.
    assert_eq!(
        infer("const count = ref<number>()", "count", ReactiveKind::Ref).as_deref(),
        Some("number"),
    );
    assert_eq!(
        infer("const item = ref<Foo>()", "item", ReactiveKind::Ref).as_deref(),
        Some("Foo"),
    );
    // Parenthesized initializer form, `let` keyword.
    assert_eq!(
        infer("let name = ref(\"x\")", "name", ReactiveKind::Ref).as_deref(),
        Some("string"),
    );
    assert_eq!(
        infer("const n = ref(0)", "n", ReactiveKind::Ref).as_deref(),
        Some("number"),
    );
    // Computed arrow body inference.
    assert_eq!(
        infer(
            "const total = computed(() => 1 + 2)",
            "total",
            ReactiveKind::Computed
        )
        .as_deref(),
        Some("number"),
    );
    // shallowRef / toRef callee mapping.
    assert_eq!(
        infer(
            "const flag = shallowRef(true)",
            "flag",
            ReactiveKind::ShallowRef
        )
        .as_deref(),
        Some("boolean"),
    );
}

#[test]
fn receiver_extractor_matches_cursor_context_on_chains() {
    use super::context::member_access_receiver;
    use crate::ide::cursor_context::CursorContext;

    // Chained member access must not silently truncate to the leaf
    // identifier. `CursorContext::detect` exposes the full receiver
    // (`count.value`), and the completion-side extractor used to disagree
    // (yielding just `value`). Pin the fix from issue #751 by asserting
    // both extractors agree.
    for src in ["count.value.", "obj.foo.", "a.b.c.", "count.", "foo[0]."] {
        let from_completion = member_access_receiver(src, src.len());
        let from_cursor = match CursorContext::detect(src, src.len()) {
            CursorContext::MemberAccess { receiver, .. } => Some(receiver),
            _ => None,
        };
        assert_eq!(
            from_completion, from_cursor,
            "receiver mismatch for {src:?}",
        );
    }

    // Specifically pin the chained `count.value.` shape — used to truncate
    // to `value`.
    assert_eq!(
        member_access_receiver("count.value.", "count.value.".len()),
        Some("count.value"),
    );

    // No trailing dot → not a member-access context.
    assert_eq!(member_access_receiver("count", 5), None);
}

#[test]
fn pins_negative_and_non_inferrable_cases() {
    // Reactive kind has no Ref/ComputedRef wrapper.
    assert_eq!(
        infer("const obj = reactive({})", "obj", ReactiveKind::Reactive),
        None,
    );
    // Name not declared in the script.
    assert_eq!(
        infer("const a = ref(1)", "missing", ReactiveKind::Ref),
        None
    );
    // Callee mismatch: a `ref` declaration is not matched when asked as toRef.
    assert_eq!(infer("const t = ref(1)", "t", ReactiveKind::ToRef), None);
    // Prefix-substring guard: `countX` must not match a query for `count`.
    assert_eq!(
        infer("const countX = ref(1)", "count", ReactiveKind::Ref),
        None
    );
    // A space between callee and `<` defeats the heuristic (preserved).
    assert_eq!(infer("const s = ref <Foo>()", "s", ReactiveKind::Ref), None);
}

#[test]
fn member_positions_suppress_extras_identifier_positions_keep_them() {
    // Bare dot, dot with a typed prefix, optional chaining, and whitespace
    // after the dot are member positions (#3933).
    let cases = [
        ("rootContext.", 12, true),
        ("rootContext.fil", 15, true),
        ("rootContext?.fil", 16, true),
        ("rootContext.  ", 14, true),
        // Identifier positions keep the extras.
        ("const x = fil", 13, false),
        ("", 0, false),
        ("rootContext", 11, false),
        // An in-progress decimal literal is not member access, matching
        // `complete_script`'s receiver classification.
        ("1.", 2, false),
        ("const n = 42.", 13, false),
    ];
    for (content, offset, expected) in cases {
        assert_eq!(
            super::completes_a_member(content, offset),
            expected,
            "content={content:?} offset={offset}"
        );
    }
}

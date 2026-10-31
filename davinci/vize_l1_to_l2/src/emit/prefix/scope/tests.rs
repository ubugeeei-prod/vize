use super::PrefixScope;
use crate::emit::options::{BindingKind, BindingTable, ReactiveRead};

#[test]
fn for_and_slot_locals_shadow_ref_metadata_and_reactive_reads_until_scope_exit() {
    let table = BindingTable::new(
        [
            ("visibility", BindingKind::SetupRef),
            ("maybe", BindingKind::SetupMaybeRef),
            ("mutable", BindingKind::SetupLet),
            ("tracked", BindingKind::SetupMaybeRef),
        ],
        [],
        true,
    )
    .with_reactive_reads([
        ("visibility", ReactiveRead::Value),
        ("tracked", ReactiveRead::Value),
    ]);
    let mut scope = PrefixScope::new(Some(&table), true, false, true);
    let expected = [
        ("visibility", (true, false)),
        ("maybe", (false, true)),
        ("mutable", (false, true)),
        ("tracked", (true, false)),
    ];
    for (name, read) in expected {
        assert_eq!((scope.is_ref_binding(name), scope.needs_unref(name)), read);
    }
    let mark = scope.mark();
    scope.push_for([Some("visibility"), Some("maybe"), Some("mutable")]);
    scope.push_slot("{ tracked }");
    for (name, _) in expected {
        assert_eq!(scope.identifier_prefix(name), None);
        assert_eq!(
            (scope.is_ref_binding(name), scope.needs_unref(name)),
            (false, false)
        );
    }
    scope.pop(mark);
    for (name, read) in expected {
        assert_eq!((scope.is_ref_binding(name), scope.needs_unref(name)), read);
    }
}

#[test]
fn implicit_event_parameter_shadows_a_same_named_setup_ref_until_scope_exit() {
    let table = BindingTable::new([("$event", BindingKind::SetupRef)], [], true)
        .with_reactive_reads([("$event", ReactiveRead::Value)]);
    let mut scope = PrefixScope::new(Some(&table), true, false, true);
    assert!(scope.is_ref_binding("$event"));
    let mark = scope.mark();
    scope.push_event();
    assert_eq!(scope.identifier_prefix("$event"), None);
    assert!(!scope.is_ref_binding("$event"));
    assert!(!scope.needs_unref("$event"));
    scope.pop(mark);
    assert!(scope.is_ref_binding("$event"));
}

#[test]
fn identifier_prefix_follows_the_binding_table_in_non_inline_mode() {
    let table = BindingTable::new(
        [
            ("count", BindingKind::SetupRef),
            ("title", BindingKind::Props),
            ("label", BindingKind::PropsAliased),
            ("d", BindingKind::Data),
            ("method", BindingKind::Options),
            ("$slots", BindingKind::VueGlobal),
        ],
        [],
        true,
    );
    let mut scope = PrefixScope::new(Some(&table), true, false, false);
    assert_eq!(scope.identifier_prefix("count"), Some("$setup."));
    assert_eq!(scope.identifier_prefix("title"), Some("$props."));
    assert_eq!(scope.identifier_prefix("label"), Some("$props."));
    assert_eq!(scope.identifier_prefix("d"), Some("$data."));
    assert_eq!(scope.identifier_prefix("method"), Some("$options."));
    assert_eq!(scope.identifier_prefix("$slots"), Some("_ctx."));
    assert_eq!(scope.identifier_prefix("other"), Some("_ctx."));
    assert_eq!(scope.identifier_prefix("Math"), None);
    scope.push_for([Some("count"), None, None]);
    assert_eq!(scope.identifier_prefix("count"), None);
}

#[test]
fn destructured_for_values_shadow_props_inside_nested_slots() {
    let table = BindingTable::new([("version", BindingKind::Props)], [], true);
    let mut scope = PrefixScope::new(Some(&table), true, true, true);
    assert_eq!(scope.identifier_prefix("version"), Some("__props."));
    scope.push_for([Some("[dep, version]"), None, None]);
    assert_eq!(scope.identifier_prefix("version"), None);
    assert!(scope.is_slot_param("version"));
}

#[test]
fn default_codegen_scope_membership_reads_retained_patterns() {
    let mut scope = PrefixScope::new(None, false, false, false);
    scope.push_pattern("{ item, meta: { index } }: ListItem<Item>");
    assert!(scope.has_codegen_scope_params());
    assert!(scope.is_codegen_scope_param("item"));
    assert!(scope.is_codegen_scope_param("index"));
    assert!(!scope.is_codegen_scope_param("meta"));
    assert!(!scope.is_codegen_scope_param("other"));
}

#[test]
fn inline_reads_setup_bindings_off_the_closure() {
    let table = BindingTable::new(
        [
            ("count", BindingKind::SetupRef),
            ("msg", BindingKind::SetupLet),
            ("title", BindingKind::Props),
            ("handler", BindingKind::SetupConst),
        ],
        [],
        true,
    );
    let scope = PrefixScope::new(Some(&table), true, false, true);
    // A setup binding is read straight from the closure, so the
    // prefix is `None` — which is what lets the collector reach the
    // `.value` / `_unref` decisions below.
    assert_eq!(scope.identifier_prefix("count"), None);
    assert_eq!(scope.identifier_prefix("msg"), None);
    assert_eq!(scope.identifier_prefix("handler"), None);
    assert_eq!(scope.identifier_prefix("title"), Some("__props."));
    assert_eq!(scope.identifier_prefix("other"), Some("_ctx."));
    assert_eq!(
        (
            scope.is_ref_binding("count"),
            scope.is_ref_binding("msg"),
            scope.needs_unref("msg"),
            scope.needs_unref("count")
        ),
        (true, false, true, false)
    );
    assert!(scope.reads_constant_binding("handler"));
    assert!(!scope.reads_constant_binding("count"));
}

#[test]
fn without_a_table_every_free_name_is_ctx() {
    let scope = PrefixScope::new(None, true, false, false);
    assert_eq!(scope.identifier_prefix("count"), Some("_ctx."));
    assert_eq!(scope.codegen_prefix("count"), "_ctx.");
}

#[test]
fn reactivity_facts_decide_the_transform_reads_but_not_the_codegen_reads() {
    // `total` is a `computed` the analyser classed as a maybe-ref, `state`
    // a demoted `reactive` now bound with `let`, `count` a plain ref, and
    // `nested` a tracked name the binding table does not know.
    let table = BindingTable::new(
        [
            ("total", BindingKind::SetupMaybeRef),
            ("state", BindingKind::SetupLet),
            ("count", BindingKind::SetupRef),
            ("draft", BindingKind::SetupLet),
        ],
        [],
        true,
    )
    .with_reactive_reads([
        ("total", ReactiveRead::Direct),
        ("total", ReactiveRead::Value),
        ("state", ReactiveRead::Direct),
        ("nested", ReactiveRead::Value),
    ]);
    let mut inline = PrefixScope::new(Some(&table), true, false, true);
    let transform = |name| (inline.is_ref_binding(name), inline.needs_unref(name));
    let codegen = |name| {
        (
            inline.codegen_is_ref_binding(name),
            inline.codegen_needs_unref(name),
        )
    };
    assert_eq!(transform("total"), (true, false));
    assert_eq!(codegen("total"), (false, true));
    assert_eq!(transform("state"), (false, false));
    assert_eq!(codegen("state"), (false, true));
    assert_eq!(transform("count"), (true, false));
    assert_eq!(codegen("count"), (true, false));
    assert_eq!(transform("draft"), (false, true));
    assert_eq!(codegen("draft"), (false, true));
    assert_eq!(transform("nested"), (true, false));
    assert_eq!(codegen("nested"), (false, false));

    inline.enable_vdom_setup_writes();
    assert_eq!(
        inline.guarded_setup_write_kind("total"),
        None,
        "authored handlers do not guard maybe-ref writes"
    );
    inline.set_model_maybe_ref_write(true);
    assert_eq!(
        inline.guarded_setup_write_kind("total"),
        Some(BindingKind::SetupMaybeRef)
    );
    assert_eq!(
        inline.guarded_setup_write_kind("state"),
        Some(BindingKind::SetupLet)
    );
    let direct_maybe_ref = BindingTable::new([("state", BindingKind::SetupMaybeRef)], [], true)
        .with_reactive_reads([("state", ReactiveRead::Direct)]);
    let mut direct_maybe_ref_scope = PrefixScope::new(Some(&direct_maybe_ref), true, false, true);
    direct_maybe_ref_scope.enable_vdom_setup_writes();
    direct_maybe_ref_scope.set_model_maybe_ref_write(true);
    assert_eq!(
        direct_maybe_ref_scope.guarded_setup_write_kind("state"),
        None
    );
    assert_eq!(
        inline.guarded_setup_write_kind("draft"),
        Some(BindingKind::SetupLet)
    );

    let module = PrefixScope::new(Some(&table), true, false, false);
    for name in ["total", "state", "count", "draft", "nested"] {
        assert_eq!(
            (module.is_ref_binding(name), module.needs_unref(name)),
            (false, false),
            "{name}: non-inline render functions read through the proxies"
        );
    }
}

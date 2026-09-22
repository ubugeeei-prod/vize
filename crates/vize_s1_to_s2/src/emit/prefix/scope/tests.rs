use super::PrefixScope;
use crate::emit::options::{BindingKind, BindingTable, ReactiveRead};

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
    let inline = PrefixScope::new(Some(&table), true, false, true);
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

    let module = PrefixScope::new(Some(&table), true, false, false);
    for name in ["total", "state", "count", "draft", "nested"] {
        assert_eq!(
            (module.is_ref_binding(name), module.needs_unref(name)),
            (false, false),
            "{name}: non-inline render functions read through the proxies"
        );
    }
}

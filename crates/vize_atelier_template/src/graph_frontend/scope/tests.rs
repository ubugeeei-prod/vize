use super::{pattern_bindings, strip_local_scope_prefixes};

#[test]
fn extracts_nested_patterns_and_strips_only_lexical_bindings() {
    let scope = pattern_bindings("[{ id: dep = fallback }, version, ...rest]");
    assert!(scope.contains("dep"));
    assert!(scope.contains("version"));
    assert!(scope.contains("rest"));
    assert!(!scope.contains("id"));
    assert_eq!(
        strip_local_scope_prefixes(
            &[scope],
            "_ctx.route(_ctx.dep, _ctx.version, _ctx.external)",
        ),
        "_ctx.route(dep, version, _ctx.external)",
    );
}

#[test]
fn rewrites_only_javascript_references_without_corrupting_literals() {
    let scope = pattern_bindings("dep, 項目");
    assert_eq!(
        strip_local_scope_prefixes(
            &[scope],
            r#"_ctx.route(_ctx.dep, _ctx.項目, `日本語 _ctx.dep ${_ctx.dep}`, "_ctx.dep", /_ctx\.dep/, /* _ctx.dep */ _ctx.external)"#,
        ),
        r#"_ctx.route(dep, 項目, `日本語 _ctx.dep ${dep}`, "_ctx.dep", /_ctx\.dep/, /* _ctx.dep */ _ctx.external)"#,
    );
}

use super::ScopeChain;

#[test]
fn test_bindings_visible_at_returns_empty_outside_any_scope() {
    let chain = ScopeChain::new();
    // Root scope uses the default span (0..0), so any non-zero offset finds
    // no containing scope and returns nothing.
    let visible = chain.bindings_visible_at(42);
    assert!(visible.is_empty(), "expected empty, got {visible:?}");
}

use super::{Buf, Helper};

#[test]
fn helper_preamble_uses_final_body_order_within_one_rank() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    buf.push("_normalizeClass(cls); _normalizeStyle(style)");

    assert_eq!(
        buf.preamble(),
        "const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle } = Vue\n"
    );
}

#[test]
fn helper_preamble_uses_final_hoist_order_within_one_rank() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    buf.push_hoist("_normalizeClass(cls)".into());
    buf.push_hoist("_normalizeStyle(style)".into());

    assert_eq!(
        buf.preamble(),
        concat!(
            "const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle } = Vue\n",
            "\n",
            "const _hoisted_1 = _normalizeClass(cls)\n",
            "const _hoisted_2 = _normalizeStyle(style)\n"
        )
    );
}

#[test]
fn helper_preamble_keeps_preferred_before_body_order() {
    let mut buf = Buf::new();
    buf.prefer(Helper::WithDirectives);
    buf.use_helper(Helper::WithKeys);
    buf.use_helper(Helper::WithDirectives);
    buf.use_helper(Helper::WithModifiers);
    buf.push("_withDirectives(node, [_withModifiers(handler), _withKeys(handler)])");

    assert_eq!(
        buf.preamble(),
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
    );
}

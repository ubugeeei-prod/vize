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
fn helper_preamble_uses_hoists_before_the_body() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::NormalizeClass);
    buf.use_helper(Helper::NormalizeStyle);
    buf.push("_normalizeClass(cls)");
    buf.push_hoist("_normalizeStyle(style)".into());

    assert_eq!(
        buf.preamble(),
        concat!(
            "const { normalizeStyle: _normalizeStyle, normalizeClass: _normalizeClass } = Vue\n",
            "\n",
            "const _hoisted_1 = _normalizeStyle(style)\n"
        )
    );
}

#[test]
fn helper_preamble_ignores_alias_shaped_authored_text() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    buf.push(
        "['_normalizeStyle()', value._normalizeStyle(), é_normalizeStyle(), /* _normalizeStyle() */ _normalizeClass(cls), _normalizeStyle(style)]",
    );

    assert_eq!(
        buf.preamble(),
        "const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle } = Vue\n"
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
        "const { withDirectives: _withDirectives, withKeys: _withKeys, withModifiers: _withModifiers } = Vue\n"
    );
}

#[test]
fn helper_preamble_keeps_preferred_directives_before_modifier_body_order() {
    let mut buf = Buf::new();
    buf.prefer(Helper::WithDirectives);
    buf.use_helper(Helper::WithModifiers);
    buf.use_helper(Helper::WithDirectives);
    buf.push("_withModifiers(handler, [\"stop\"]); _withDirectives(node, [])");

    assert_eq!(
        buf.preamble(),
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers } = Vue\n"
    );
}

#[test]
fn helper_preamble_keeps_unpreferred_rank_two_in_first_use_order() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::WithKeys);
    buf.use_helper(Helper::WithModifiers);
    buf.push("_withModifiers(handler, [\"stop\"]); _withKeys(handler, [\"enter\"])");

    assert_eq!(
        buf.preamble(),
        "const { withKeys: _withKeys, withModifiers: _withModifiers } = Vue\n"
    );
}

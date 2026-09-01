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
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
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
        "const { withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
    );
}

#[test]
fn helper_preamble_orders_codegen_only_directives_by_final_rank_two_use() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::WithKeys);
    buf.use_helper(Helper::WithDirectives);
    buf.use_helper(Helper::WithModifiers);
    buf.push("_withDirectives(node, { onClick: _withModifiers(handler, [\"stop\"]), onKeydown: _withKeys(handler, [\"enter\"]) })");

    assert_eq!(
        buf.preamble(),
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
    );
}

#[test]
fn helper_preamble_orders_create_slots_before_v_show_for_textful_directive_slots() {
    let mut buf = Buf::new();
    buf.use_helper(Helper::ResolveDirective);
    buf.use_helper(Helper::CreateText);
    buf.use_helper(Helper::VShow);
    buf.use_helper(Helper::CreateSlots);
    buf.push("_createSlots(slots, []); [_vShow, visible]");

    assert_eq!(
        buf.preamble(),
        "const { resolveDirective: _resolveDirective, createTextVNode: _createTextVNode, createSlots: _createSlots, vShow: _vShow } = Vue\n"
    );

    let mut buf = Buf::new();
    buf.use_helper(Helper::VShow);
    buf.use_helper(Helper::CreateSlots);
    buf.push("[_vShow, visible]; _createSlots(slots, [])");

    assert_eq!(
        buf.preamble(),
        "const { vShow: _vShow, createSlots: _createSlots } = Vue\n"
    );
}

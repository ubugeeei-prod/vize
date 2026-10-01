use super::super::options::DomEmitOptions;
use super::{Buf, Helper};

#[test]
fn newline_preserves_indentation_bytes_at_chunk_boundaries() {
    for (levels, spaces) in [
        (0, 0),
        (1, 2),
        (31, 62),
        (32, 64),
        (33, 66),
        (63, 126),
        (64, 128),
        (127, 254),
        (128, 256),
        (129, 258),
        (1024, 2048),
    ] {
        let mut buf = Buf::new(false);
        buf.push("é界");
        for _ in 0..levels {
            buf.indent();
        }
        buf.newline();
        assert_eq!(&buf.code[..6], "é界\n");
        assert_eq!(buf.code.len(), 6 + spaces);
        assert!(buf.code.as_bytes()[6..].iter().all(|byte| *byte == b' '));
        assert_eq!(buf.indent_width(), spaces);
    }
}

#[test]
fn newline_keeps_multiline_body_and_generated_byte_offsets() {
    let mut buf = Buf::new(false);
    buf.push("render(){");
    buf.indent();
    buf.newline();
    let assets_start = buf.code.len();
    buf.push("const 名 = 1;");
    buf.newline();
    let assets_end = buf.code.len();
    buf.push("return ");
    let return_start = buf.code.len();
    buf.push("['é', '界']");
    let return_end = buf.code.len();
    buf.deindent();
    buf.newline();
    buf.push("}");

    assert_eq!(
        buf.code,
        "render(){\n  const 名 = 1;\n  return ['é', '界']\n}"
    );
    assert_eq!(
        (assets_start, assets_end, return_start, return_end),
        (12, 29, 36, 49)
    );
    assert_eq!(&buf.code[return_start..return_end], "['é', '界']");
}

#[test]
fn newline_keeps_empty_and_zero_indent_lines() {
    let mut buf = Buf::new(false);
    buf.newline();
    buf.indent();
    buf.deindent();
    buf.deindent();
    buf.newline();
    assert_eq!(buf.code, "\n\n");
    assert_eq!(buf.indent_width(), 0);
}

#[test]
fn multiline_indentation_keeps_helper_import_membership_and_order() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    for _ in 0..33 {
        buf.indent();
    }
    buf.newline();
    buf.push("_normalizeClass(cls)");
    buf.newline();
    buf.push("_normalizeStyle(style)");
    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle } = Vue\n"
    );
}

#[test]
fn helper_preamble_uses_final_body_order_within_one_rank() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    buf.push("_normalizeClass(cls); _normalizeStyle(style)");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle } = Vue\n"
    );
}

#[test]
fn helper_order_recomputes_after_body_changes() {
    let mut buf = Buf::new(false);
    for helper in [
        Helper::MergeProps,
        Helper::GuardReactiveProps,
        Helper::NormalizeProps,
    ] {
        buf.use_helper(helper);
    }
    buf.push("_mergeProps(_guardReactiveProps(props), extra)");
    assert!(buf.ordered_helpers().into_iter().map(Helper::alias).eq([
        "_mergeProps",
        "_guardReactiveProps",
        "_normalizeProps"
    ]));
    buf.push_hoist("_normalizeProps(props)".into());
    assert!(buf.ordered_helpers().into_iter().map(Helper::alias).eq([
        "_normalizeProps",
        "_guardReactiveProps",
        "_mergeProps"
    ]));
}

#[test]
fn helper_preamble_uses_final_hoist_order_within_one_rank() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    buf.push_hoist("_normalizeClass(cls)".into());
    buf.push_hoist("_normalizeStyle(style)".into());

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
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
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::NormalizeClass);
    buf.use_helper(Helper::NormalizeStyle);
    buf.push("_normalizeClass(cls)");
    buf.push_hoist("_normalizeStyle(style)".into());

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        concat!(
            "const { normalizeStyle: _normalizeStyle, normalizeClass: _normalizeClass } = Vue\n",
            "\n",
            "const _hoisted_1 = _normalizeStyle(style)\n"
        )
    );
}

#[test]
fn helper_preamble_ignores_alias_shaped_authored_text() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::NormalizeStyle);
    buf.use_helper(Helper::NormalizeClass);
    buf.push(
        "['_normalizeStyle()', value._normalizeStyle(), é_normalizeStyle(), /* _normalizeStyle() */ _normalizeClass(cls), _normalizeStyle(style)]",
    );

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle } = Vue\n"
    );
}

#[test]
fn helper_preamble_keeps_preferred_before_body_order() {
    let mut buf = Buf::new(false);
    buf.prefer(Helper::WithDirectives);
    buf.use_helper(Helper::WithKeys);
    buf.use_helper(Helper::WithDirectives);
    buf.use_helper(Helper::WithModifiers);
    buf.push("_withDirectives(node, [_withModifiers(handler), _withKeys(handler)])");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
    );
}

#[test]
fn helper_preamble_keeps_preferred_directives_before_modifier_body_order() {
    let mut buf = Buf::new(false);
    buf.prefer(Helper::WithDirectives);
    buf.use_helper(Helper::WithModifiers);
    buf.use_helper(Helper::WithDirectives);
    buf.push("_withModifiers(handler, [\"stop\"]); _withDirectives(node, [])");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers } = Vue\n"
    );
}

#[test]
fn helper_preamble_keeps_unpreferred_rank_two_in_first_use_order() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::WithKeys);
    buf.use_helper(Helper::WithModifiers);
    buf.push("_withModifiers(handler, [\"stop\"]); _withKeys(handler, [\"enter\"])");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
    );
}

#[test]
fn helper_preamble_orders_codegen_only_directives_by_final_rank_two_use() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::WithKeys);
    buf.use_helper(Helper::WithDirectives);
    buf.use_helper(Helper::WithModifiers);
    buf.push("_withDirectives(node, { onClick: _withModifiers(handler, [\"stop\"]), onKeydown: _withKeys(handler, [\"enter\"]) })");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { withDirectives: _withDirectives, withModifiers: _withModifiers, withKeys: _withKeys } = Vue\n"
    );
}

#[test]
fn helper_preamble_orders_create_slots_before_v_show_for_textful_directive_slots() {
    let mut buf = Buf::new(false);
    buf.use_helper(Helper::ResolveDirective);
    buf.use_helper(Helper::CreateText);
    buf.use_helper(Helper::VShow);
    buf.use_helper(Helper::CreateSlots);
    buf.push("_createSlots(slots, []); [_vShow, visible]");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { resolveDirective: _resolveDirective, createTextVNode: _createTextVNode, createSlots: _createSlots, vShow: _vShow } = Vue\n"
    );

    let mut buf = Buf::new(false);
    buf.use_helper(Helper::VShow);
    buf.use_helper(Helper::CreateSlots);
    buf.push("[_vShow, visible]; _createSlots(slots, [])");

    assert_eq!(
        buf.preamble(&DomEmitOptions::DEFAULT),
        "const { vShow: _vShow, createSlots: _createSlots } = Vue\n"
    );
}

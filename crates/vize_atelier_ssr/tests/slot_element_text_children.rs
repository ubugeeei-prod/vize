//! #11610: an element's single text run is a scalar VNode child. Slot roots
//! and mixed element children still need arrays of VNodes.

#![cfg(feature = "legacy-differential")]

use vize_atelier_ssr::differential::compare_ssr_lanes;
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// The complete slot fallback, or the complete root VNode render call.
fn vnode_or_fallback(code: &str) -> &str {
    if let Some(start) = code.find("return [") {
        return code[start..].split('\n').next().unwrap_or_default();
    }
    code.lines()
        .find(|line| line.starts_with("  _ssrRenderVNode("))
        .unwrap_or_default()
}

fn check_both_lanes(source: &str, expected: &str) {
    let pair = compare_ssr_lanes(
        source,
        &SsrCompilerOptions::default(),
        &SsrCompilerExperimentalOptions::default(),
    );
    assert_eq!(pair.lane, "s4", "{source}");
    assert!(
        pair.selected.errors.is_empty(),
        "{source}: {:?}",
        pair.selected.errors
    );
    assert!(
        pair.legacy.errors.is_empty(),
        "{source}: {:?}",
        pair.legacy.errors
    );
    assert_eq!(
        pair.selected.result.code, pair.legacy.result.code,
        "{source}"
    );
    for (lane, code) in [
        ("s4", pair.selected.result.code),
        ("legacy", pair.legacy.result.code),
    ] {
        assert_eq!(
            vnode_or_fallback(&code),
            expected,
            "{lane}: {source}\n{code}"
        );
    }
}

#[test]
fn an_elements_only_text_run_is_a_scalar_on_both_lanes() {
    for (source, expected) in [
        (
            "<Comp><div>{{ count }} tail</div></Comp>",
            r#"return [_createElementVNode("div", null, _toDisplayString(_ctx.count) + " tail", 1 /* TEXT */)]"#,
        ),
        (
            "<Comp><p>{{ a }}{{ b }}</p></Comp>",
            r#"return [_createElementVNode("p", null, _toDisplayString(_ctx.a) + _toDisplayString(_ctx.b), 1 /* TEXT */)]"#,
        ),
        (
            r#"<component :is="view"><div>{{ count }}</div></component>"#,
            r#"  _ssrRenderVNode(_push, _createVNode(_resolveDynamicComponent(_ctx.view), _attrs, { default: _withCtx(() => [_createElementVNode("div", null, _toDisplayString(_ctx.count), 1 /* TEXT */)]), _: 1 }), _parent)"#,
        ),
    ] {
        check_both_lanes(source, expected);
    }
}

#[test]
fn static_text_mixed_children_and_slot_roots_keep_their_shapes() {
    for (source, expected) in [
        (
            "<Comp><div>plain</div></Comp>",
            r#"return [_createElementVNode("div", null, "plain")]"#,
        ),
        (
            "<Comp><div>{{ count }}<i></i>tail</div></Comp>",
            r#"return [_createElementVNode("div", null, [_createTextVNode(_toDisplayString(_ctx.count)), _createElementVNode("i", null, null), _createTextVNode("tail")])]"#,
        ),
        (
            "<Comp>{{ count }} tail</Comp>",
            r#"return [_createTextVNode(_toDisplayString(_ctx.count) + " tail")]"#,
        ),
    ] {
        check_both_lanes(source, expected);
    }
}

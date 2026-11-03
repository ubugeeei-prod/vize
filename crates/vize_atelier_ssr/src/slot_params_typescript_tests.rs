//! `v-slot` props patterns on both SSR emitters: type-erased under `is_ts`
//! by the expression lane's parameter-list erasure, authored bytes otherwise.

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_l0::{Allocator, String, cstr};

fn compile(source: &str, is_ts: bool, lane: SsrLane) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_on_lane(
        &allocator,
        source,
        SsrCompilerOptions {
            is_ts,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
        lane,
    );
    assert!(errors.is_empty(), "{source}: {errors:?}");
    cstr!("{}\n{}", result.preamble, result.code)
}

/// The output both emitters agree on.
fn both_lanes(source: &str, is_ts: bool) -> String {
    let selected = compile(source, is_ts, SsrLane::Selected);
    assert_eq!(
        compile(source, is_ts, SsrLane::LegacyOnly),
        selected,
        "{source} (is_ts: {is_ts})"
    );
    selected
}

/// `<Card>` with one static slot `name` taking `params` and reading `read`.
fn card(name: &str, params: &str, read: &str) -> String {
    cstr!(
        "import {{ ssrInterpolate as _ssrInterpolate, ssrRenderComponent as _ssrRenderComponent }} from \"@vue/server-renderer\"\nimport {{ createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx }} from \"vue\"\n\nfunction ssrRender(_ctx, _push, _parent, _attrs) {{\n  _push(_ssrRenderComponent(_resolveComponent(\"Card\"), _attrs, {{\n    {name}: _withCtx(({params}, _push, _parent, _scopeId) => {{\n      if (_push) {{\n        _push(`${{_ssrInterpolate({read})}}`)\n      }} else {{\n        return [_createTextVNode(_toDisplayString({read}))]\n      }}\n    }}),\n    _: 1\n  }}, _parent))\n}}\n"
    )
}

#[test]
fn typed_slot_params_leave_as_javascript_on_both_lanes() {
    let cases = [
        (
            r#"<Card #default="{ row }: { row: Row }">{{ row }}</Card>"#,
            card("default", "{ row }", "row"),
        ),
        (
            r#"<Card v-slot="props: Props">{{ props }}</Card>"#,
            card("default", "props", "props"),
        ),
        (
            r#"<Card><template #item="{ row }: { row: Row }">{{ row }}</template></Card>"#,
            card("item", "{ row }", "row"),
        ),
        (
            r#"<Card #default="{ row = 0 as number }: { row?: number }">{{ row }}</Card>"#,
            card("default", "{ row = 0 }", "row"),
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(both_lanes(source, true), expected, "{source}");
    }
    // A `createSlots` entry reads the same pattern.
    assert_eq!(
        both_lanes(
            r#"<Card><template v-if="ok" #item="{ row }: { row: Row }">{{ row }}</template></Card>"#,
            true
        ),
        "import { ssrInterpolate as _ssrInterpolate, ssrRenderComponent as _ssrRenderComponent } from \"@vue/server-renderer\"\nimport { createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx } from \"vue\"\n\nfunction ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(_ssrRenderComponent(_resolveComponent(\"Card\"), _attrs, _createSlots({\n    _: 2 /* DYNAMIC */\n  }, [\n    (_ctx.ok) ? {\n      name: \"item\",\n      fn: _withCtx(({ row }, _push, _parent, _scopeId) => {\n        if (_push) {\n          _push(`${_ssrInterpolate(row)}`)\n        } else {\n          return [_createTextVNode(_toDisplayString(row))]\n        }\n      }),\n      key: \"0\"\n    } : undefined\n  ]), _parent))\n}\n"
    );
}

/// Negative control: JavaScript params keep their authored bytes in either
/// mode. Erasure is decided by the parse, never by the text scan, which
/// fires on these strings and used to reprint `'x as y'` as `"x as y"`.
#[test]
fn javascript_slot_params_keep_their_bytes() {
    for is_ts in [false, true] {
        assert_eq!(
            both_lanes(
                r#"<Card #default="{ label = 'a as b' }">{{ label }}</Card>"#,
                is_ts
            ),
            card("default", "{ label = 'a as b' }", "label"),
            "is_ts: {is_ts}"
        );
        assert_eq!(
            both_lanes(r#"<Card #default="{ 'x as y': z }">{{ z }}</Card>"#, is_ts),
            card("default", "{ 'x as y': z }", "z"),
            "is_ts: {is_ts}"
        );
    }
}

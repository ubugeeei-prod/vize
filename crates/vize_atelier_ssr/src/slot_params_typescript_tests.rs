//! `v-slot` props patterns on both SSR emitters: type-erased under `is_ts`
//! by the expression lane's parameter-list erasure, authored bytes otherwise.

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::options::{
    BindingMetadata, BindingType, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_l0::{Allocator, String, cstr};

fn compile_with_bindings(
    source: &str,
    is_ts: bool,
    lane: SsrLane,
    binding_metadata: Option<BindingMetadata>,
) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_on_lane(
        &allocator,
        source,
        SsrCompilerOptions {
            is_ts,
            binding_metadata,
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

fn compile(source: &str, is_ts: bool, lane: SsrLane) -> String {
    compile_with_bindings(source, is_ts, lane, None)
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

fn both_lanes_with_bindings(source: &str, bindings: &[(&str, BindingType)]) -> String {
    let metadata = || {
        let mut metadata = BindingMetadata::default();
        for (name, kind) in bindings {
            metadata.bindings.insert((*name).into(), *kind);
        }
        metadata.is_script_setup = true;
        metadata
    };
    let selected = compile_with_bindings(source, false, SsrLane::Selected, Some(metadata()));
    assert_eq!(
        compile_with_bindings(source, false, SsrLane::LegacyOnly, Some(metadata())),
        selected,
        "{source}"
    );
    selected
}

/// `<Card>` with one static slot `name` taking `params` and reading `read`.
fn card(name: &str, params: &str, read: &str) -> String {
    cstr!(
        "import {{ ssrInterpolate as _ssrInterpolate, ssrRenderComponent as _ssrRenderComponent }} from \"@vue/server-renderer\"\nimport {{ createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx }} from \"vue\"\n\nfunction ssrRender(_ctx, _push, _parent, _attrs) {{\n  _push(_ssrRenderComponent(_resolveComponent(\"Card\"), _attrs, {{\n    {name}: _withCtx(({params}, _push, _parent, _scopeId) => {{\n      if (_push) {{\n        _push(`${{_ssrInterpolate({read})}}`)\n      }} else {{\n        return [_createTextVNode(_toDisplayString({read}))]\n      }}\n    }}),\n    _: 1\n  }}, _parent))\n}}\n"
    )
}

fn card_with_bindings(name: &str, params: &str, read: &str) -> String {
    card(name, params, read).replacen(
        "function ssrRender(_ctx, _push, _parent, _attrs)",
        "function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options)",
        1,
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

/// Negative control: TS-looking JavaScript keeps its authored bytes in
/// either mode, as it always did.
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
    }
}

#[test]
fn slot_param_defaults_prefix_free_identifiers_on_both_lanes() {
    for is_ts in [false, true] {
        assert_eq!(
            both_lanes(
                r#"<Card #default="{ ok = done!==1 }">{{ ok }}</Card>"#,
                is_ts
            ),
            card("default", "{ ok = _ctx.done!==1 }", "ok"),
            "is_ts: {is_ts}"
        );
        assert_eq!(
            both_lanes(
                r#"<Card #default="{ ok = function f(n) { return n ? f(n - 1) : done } }">{{ ok }}</Card>"#,
                is_ts
            ),
            card(
                "default",
                "{ ok = function f(n) { return n ? f(n - 1) : _ctx.done } }",
                "ok"
            ),
            "named function default (is_ts: {is_ts})"
        );
    }

    assert_eq!(
        both_lanes(
            r#"<Card #default="{ row = fallback, active = row.active, n = Math.max(seed, 1), map = item => item.id, config = { value } }: SlotProps">{{ row }}</Card>"#,
            true
        ),
        card(
            "default",
            "{ row = _ctx.fallback, active = row.active, n = Math.max(_ctx.seed, 1), map = (item) => item.id, config = { value: _ctx.value } }",
            "row"
        )
    );
}

#[test]
fn slot_param_defaults_use_binding_specific_render_prefixes() {
    let output = both_lanes_with_bindings(
        r#"<Card #default="{ value = setupValue, title = propValue, data = dataValue, option = optionValue, other = fallback }">{{ value }}</Card>"#,
        &[
            ("setupValue", BindingType::SetupConst),
            ("propValue", BindingType::Props),
            ("dataValue", BindingType::Data),
            ("optionValue", BindingType::Options),
        ],
    );
    assert_eq!(
        output,
        card_with_bindings(
            "default",
            "{ value = $setup.setupValue, title = $props.propValue, data = $data.dataValue, option = $options.optionValue, other = _ctx.fallback }",
            "value"
        )
    );
}

#[test]
fn slot_param_defaults_keep_outer_template_locals_and_authored_strings() {
    for lane in [SsrLane::Selected, SsrLane::LegacyOnly] {
        let output = compile(
            r#"<Card v-for="row in rows" #default="{ item = row, text = '_ctx.row' }">{{ item }}</Card>"#,
            false,
            lane,
        );
        let signature = output
            .lines()
            .find(|line| line.trim_start().starts_with("default: _withCtx(("))
            .map(str::trim);
        assert_eq!(
            signature,
            Some(
                "default: _withCtx(({ item = row, text = '_ctx.row' }, _push, _parent, _scopeId) => {"
            ),
            "{lane:?}: {output}"
        );
    }
}

/// Erasure is decided by the parse, never by the text scan. The scan fires
/// on these string keys, and the old strip, run whatever `is_ts` was,
/// reprinted the JavaScript (`{ "x as y": z }`).
#[test]
fn scan_tripping_string_keys_are_not_reprinted() {
    for is_ts in [false, true] {
        for (params, read) in [("{ 'x as y': z }", "z"), ("{ 'a!': b }", "b")] {
            assert_eq!(
                both_lanes(
                    &cstr!("<Card #default=\"{params}\">{{{{ {read} }}}}</Card>"),
                    is_ts
                ),
                card("default", params, read),
                "is_ts: {is_ts}"
            );
        }
    }
}

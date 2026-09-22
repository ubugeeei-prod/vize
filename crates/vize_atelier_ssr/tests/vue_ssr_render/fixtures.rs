//! Render-test fixtures for the `fix(ssr)!` Vue 3.5 alignment: one per
//! changed shape, with the pinned output of the historical legacy walker
//! (`legacy`, captured before the fix) and the aligned output (`expected`).

pub struct Fixture {
    pub name: &'static str,
    pub template: &'static str,
    /// The `_ctx` state, as JSON.
    pub data: &'static str,
    /// Names exposed from `setup()` (`$setup.x` under binding metadata).
    pub setup: &'static [&'static str],
    /// Compiler binding metadata: `(name, vue binding type)`.
    pub bindings: &'static [(&'static str, &'static str)],
    /// Fallthrough attrs the parent passes, as JSON.
    pub attrs: &'static str,
    /// The pre-fix SSR module (preamble + render function).
    pub legacy: &'static str,
    /// The aligned SSR module this crate must emit.
    pub expected: &'static str,
}

pub const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "directive-value-arg-modifiers",
        template: r#"<div><p v-focus:a.b="x" class="a">t</p></div>"#,
        data: r#"{"x":"on"}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "focus"))} class="a">t</p></div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
import { resolveDirective as _resolveDirective, mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_mergeProps({ class: "a" }, _ssrGetDirectiveProps(_ctx, _resolveDirective("focus"), _ctx.x, "a", { b: true })))}>t</p></div>`)
}
"#,
    },
    Fixture {
        name: "directive-owns-empty-content",
        template: r#"<div><p v-content></p><p v-markup></p></div>"#,
        data: "{}",
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "content"))}></p><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "markup"))}></p></div>`)
}
"#,
        expected: r#"import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
import { resolveDirective as _resolveDirective } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  let _temp0, _temp1
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_temp0 = _ssrGetDirectiveProps(_ctx, _resolveDirective("content")))}>${("textContent" in _temp0) ? _ssrInterpolate(_temp0.textContent) : _temp0.innerHTML ?? ''}</p><p${_ssrRenderAttrs(_temp1 = _ssrGetDirectiveProps(_ctx, _resolveDirective("markup")))}>${("textContent" in _temp1) ? _ssrInterpolate(_temp1.textContent) : _temp1.innerHTML ?? ''}</p></div>`)
}
"#,
    },
    Fixture {
        name: "directive-owns-textarea-value",
        template: r#"<div><textarea v-val>fallback</textarea></div>"#,
        data: "{}",
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><textarea${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "val"))}>fallback</textarea></div>`)
}
"#,
        expected: r#"import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
import { resolveDirective as _resolveDirective } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  let _temp0
  _push(`<div${_ssrRenderAttrs(_attrs)}><textarea${_ssrRenderAttrs(_temp0 = _ssrGetDirectiveProps(_ctx, _resolveDirective("val")), "textarea")}>${_ssrInterpolate(("value" in _temp0) ? _temp0.value : "fallback")}</textarea></div>`)
}
"#,
    },
    Fixture {
        name: "directive-on-fallthrough-root",
        template: r#"<section class="r" v-focus="x">x</section>"#,
        data: r#"{"x":1}"#,
        setup: &[],
        bindings: &[],
        attrs: r#"{"data-parent":"p"}"#,
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
import { mergeProps as _mergeProps, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<section${_ssrRenderAttrs(_mergeProps(_normalizeProps(_guardReactiveProps(_ssrGetDirectiveProps(_ctx, _directives, "focus"))), { class: "r" }, _attrs))}>x</section>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
import { resolveDirective as _resolveDirective, mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<section${_ssrRenderAttrs(_mergeProps({ class: "r" }, _attrs, _ssrGetDirectiveProps(_ctx, _resolveDirective("focus"), _ctx.x)))}>x</section>`)
}
"#,
    },
    Fixture {
        name: "directive-from-setup-binding",
        template: r#"<div><p v-focus="x">t</p></div>"#,
        data: r#"{"x":"s"}"#,
        setup: &["vFocus", "x"],
        bindings: &[("vFocus", "setup-const"), ("x", "setup-ref")],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "focus"))}>t</p></div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, $setup["vFocus"], $setup.x))}>t</p></div>`)
}
"#,
    },
    Fixture {
        name: "builtin-once-cloak-memo",
        template: r#"<div><p v-once>{{ a }}</p><p v-cloak>{{ b }}</p><p v-memo="[c]">{{ c }}</p></div>"#,
        data: r#"{"a":1,"b":2,"c":3}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs, ssrGetDirectiveProps as _ssrGetDirectiveProps } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "once"))}>${_ssrInterpolate(_ctx.a)}</p><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "cloak"))}>${_ssrInterpolate(_ctx.b)}</p><p${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, "memo"))}>${_ssrInterpolate(_ctx.c)}</p></div>`)
}
"#,
        expected: r#"import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p>${_ssrInterpolate(_ctx.a)}</p><p>${_ssrInterpolate(_ctx.b)}</p><p>${_ssrInterpolate(_ctx.c)}</p></div>`)
}
"#,
    },
    Fixture {
        name: "inline-camel-bind",
        template: r#"<div><svg :view-box.camel="vb"></svg></div>"#,
        data: r#"{"vb":"0 0 1 1"}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrRenderAttr as _ssrRenderAttr } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><svg${_ssrRenderAttr("view-box", _ctx.vb)}></svg></div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrRenderAttr as _ssrRenderAttr } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><svg${_ssrRenderAttr("viewBox", _ctx.vb)}></svg></div>`)
}
"#,
    },
    Fixture {
        name: "inline-dynamic-key-bind",
        template: r#"<div><p :[k]="v" class="a">t</p></div>"#,
        data: r#"{"k":"title","v":"hi"}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_ctx.v)} class="a">t</p></div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs({ [_ctx.k || ""]: _ctx.v, class: "a" })}>t</p></div>`)
}
"#,
    },
    Fixture {
        name: "inline-spread-collision",
        template: r#"<div><p class="a" id="first" v-bind="obj">t</p></div>"#,
        data: r#"{"obj":{"id":"second","class":"b"}}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p class="a" id="first"${_ssrRenderAttrs(_ctx.obj)}>t</p></div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p${_ssrRenderAttrs(_mergeProps({ id: "first", class: "a" }, _ctx.obj))}>t</p></div>`)
}
"#,
    },
    Fixture {
        name: "root-spread-source-order",
        template: r#"<div id="a" v-bind="obj">x</div>"#,
        data: r#"{"obj":{"id":"b"}}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps(_normalizeProps(_guardReactiveProps(_ctx.obj)), { id: "a" }, _attrs))}>x</div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps({ id: "a" }, _ctx.obj, _attrs))}>x</div>`)
}
"#,
    },
    Fixture {
        name: "root-v-show-after-attrs",
        template: r#"<div v-show="s" class="a">x</div>"#,
        data: r#"{"s":false}"#,
        setup: &[],
        bindings: &[],
        attrs: r#"{"style":"color:red"}"#,
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps({ class: "a", style: ((_ctx.s) ? null : { display: "none" }) }, _attrs))}>x</div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps({ class: "a" }, _attrs, { style: ((_ctx.s) ? null : { display: "none" }) }))}>x</div>`)
}
"#,
    },
    Fixture {
        name: "outlet-dynamic-key",
        template: r#"<div><slot :[k]="v" /></div>"#,
        data: r#"{"k":"foo","v":1}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderSlot as _ssrRenderSlot, ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}>`)
  _ssrRenderSlot(_ctx.$slots, "default", { [k || ""]: _ctx.v }, null, _push, _parent)
  _push(`</div>`)
}
"#,
        expected: r#"import { ssrRenderSlot as _ssrRenderSlot, ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}>`)
  _ssrRenderSlot(_ctx.$slots, "default", { [_ctx.k || ""]: _ctx.v }, null, _push, _parent)
  _push(`</div>`)
}
"#,
    },
    Fixture {
        name: "component-dynamic-key-in-loop",
        template: r#"<div><Foo v-for="k in ks" :[k]="1" /></div>"#,
        data: r#"{"ks":["data-a","data-b"]}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderComponent as _ssrRenderComponent, ssrRenderAttrs as _ssrRenderAttrs, ssrRenderList as _ssrRenderList } from "@vue/server-renderer"
import { resolveComponent as _resolveComponent, normalizeProps as _normalizeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}>`)
  _push(`<!--[-->`)
  _ssrRenderList(_ctx.ks, (k) => {
    _push(_ssrRenderComponent(_resolveComponent("Foo"), _normalizeProps({ [_ctx.k || ""]: 1 }), null, _parent))
  })
  _push(`<!--]-->`)
  _push(`</div>`)
}
"#,
        expected: r#"import { ssrRenderComponent as _ssrRenderComponent, ssrRenderAttrs as _ssrRenderAttrs, ssrRenderList as _ssrRenderList } from "@vue/server-renderer"
import { resolveComponent as _resolveComponent, normalizeProps as _normalizeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}>`)
  _push(`<!--[-->`)
  _ssrRenderList(_ctx.ks, (k) => {
    _push(_ssrRenderComponent(_resolveComponent("Foo"), _normalizeProps({ [k || ""]: 1 }), null, _parent))
  })
  _push(`<!--]-->`)
  _push(`</div>`)
}
"#,
    },
    Fixture {
        name: "input-spread-model",
        template: r#"<div><input v-bind="o" v-model="m"></div>"#,
        data: r#"{"o":{"type":"checkbox"},"m":true}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrRenderAttr as _ssrRenderAttr } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><input${_ssrRenderAttrs(_ctx.o)}${_ssrRenderAttr("value", _ctx.m)}></div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs, ssrGetDynamicModelProps as _ssrGetDynamicModelProps } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><input${_ssrRenderAttrs(_mergeProps(_ctx.o, _ssrGetDynamicModelProps(_ctx.o, _ctx.m)))}></div>`)
}
"#,
    },
    Fixture {
        name: "merged-prop-attr-binds",
        template: r#"<div :title.prop="t" :data-x.attr="d">x</div>"#,
        data: r#"{"t":"tip","d":"dx"}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps({ ".title": _ctx.t, "^data-x": _ctx.d }, _attrs))}>x</div>`)
}
"#,
        expected: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
import { mergeProps as _mergeProps } from "vue"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps({ title: _ctx.t, "data-x": _ctx.d }, _attrs))}>x</div>`)
}
"#,
    },
    Fixture {
        name: "merged-textarea-from-spread",
        template: r#"<div><textarea v-bind="o">fallback</textarea></div>"#,
        data: r#"{"o":{"value":"from spread","rows":"2"}}"#,
        setup: &[],
        bindings: &[],
        attrs: "null",
        legacy: r#"import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><textarea${_ssrRenderAttrs(_ctx.o)}>fallback</textarea></div>`)
}
"#,
        expected: r#"import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"
function ssrRender(_ctx, _push, _parent, _attrs) {
  let _temp0
  _push(`<div${_ssrRenderAttrs(_attrs)}><textarea${_ssrRenderAttrs(_temp0 = _ctx.o, "textarea")}>${_ssrInterpolate(("value" in _temp0) ? _temp0.value : "fallback")}</textarea></div>`)
}
"#,
    },
];

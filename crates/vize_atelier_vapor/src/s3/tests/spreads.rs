//! `v-bind`/`v-on` objects (`native/validate/spread.rs`). Element objects take
//! upstream's ordered `setDynamicProps` sources, which the retained lane does
//! not emit (see the P3-6 record); every other object shape matches the
//! retained lane byte for byte.

use super::{generated, lowered_source, options};
use crate::s3::{
    LegacyReason, VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained,
};
use vize_carton::Allocator;
use vize_s3::operand::OperandRole;

fn compile(source: &str, davinci_retained_lane: bool) -> vize_carton::String {
    let allocator = Allocator::new();
    let options = crate::VaporCompilerOptions {
        prefix_identifiers: true,
        davinci_retained_lane,
        ..Default::default()
    };
    crate::compile_vapor(&allocator, source, options).code
}

fn accepted(source: &str) {
    let allocator = Allocator::new();
    let status = lower_source_for_vapor(&allocator, source, options());
    assert!(
        matches!(status, VaporS3BridgeStatus::Accepted(_)),
        "{source}: {status:?}"
    );
}

/// Static attributes, `:prop`s and the object become upstream's sources in
/// authored order; `class`/`style` merge within a group, and the template
/// keeps no attribute the runtime merges.
#[test]
fn element_objects_emit_upstream_ordered_sources() {
    for (source, expected) in [
        (
            r#"<div class="a" :class="c" v-bind="o" style="color:red" :style="s" data-x="1" :aria-label="l" disabled></div>"#,
            "const t0 = _template(\"<div></div>\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  _renderEffect(() => _setDynamicProps(n0, [{ class: [\"a\", _ctx.c] }, _ctx.o, { style: [\"color:red\", _ctx.s], \"data-x\": \"1\", \"aria-label\": _ctx.l, disabled: \"\" }]))\n  return n0\n}",
        ),
        (
            r#"<div class="a" v-bind="o" :class="c" id="z"></div>"#,
            "const t0 = _template(\"<div></div>\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  _renderEffect(() => _setDynamicProps(n0, [{ class: \"a\" }, _ctx.o, { class: _ctx.c, id: \"z\" }]))\n  return n0\n}",
        ),
        (
            r#"<div :id="i" v-bind="o"><span v-bind="p">{{ t }}</span></div>"#,
            "const t0 = _template(\"<div><span> </span></div>\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  const n1 = _child(n0)\n  const x1 = _txt(n1)\n  _renderEffect(() => {\n    _setDynamicProps(n0, [{ id: _ctx.i }, _ctx.o])\n    _setDynamicProps(n1, [_ctx.p])\n    _setText(x1, _toDisplayString(_ctx.t))\n  })\n  return n0\n}",
        ),
    ] {
        accepted(source);
        let code = compile(source, false);
        let body = code
            .split_once('\n')
            .map_or("", |(_, body)| body)
            .trim_end();
        assert_eq!(body, expected, "{source}");
    }
}

/// Component objects (`$` sources in authored order) and element listener
/// objects are the retained lane's shapes.
#[test]
fn component_and_listener_objects_match_the_retained_lane() {
    for source in [
        r#"<MyComponent v-bind="props" :extra="value" @update="onUpdate" />"#,
        r#"<div><MyComp class="a" :class="c" v-bind="o" /></div>"#,
        r#"<component :is="view" id="x" v-bind="attrs" />"#,
        r#"<button v-on="handlers"></button>"#,
        r#"<button :title="t" v-on="h">x {{ y }}</button>"#,
    ] {
        accepted(source);
        assert_eq!(compile(source, false), compile(source, true), "{source}");
    }
}

/// A component `v-on` object normalizes its keys with `toHandlers`, as
/// upstream does; the retained lane drops the object.
#[test]
fn component_listener_objects_normalize_handler_keys() {
    let source = r#"<MyComp v-on="h" title="t" />"#;
    accepted(source);
    assert_eq!(
        compile(source, false).as_str(),
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, toHandlers as _toHandlers } from 'vue';\n\nexport function render(_ctx) {\n  const _component_MyComp = _resolveComponent(\"MyComp\")\n  const n0 = _createComponentWithFallback(_component_MyComp, { $: [\n    () => (_toHandlers(_ctx.h)),\n    { title: () => (\"t\") }\n  ] }, null, true)\n  return n0\n}\n"
    );
    assert_eq!(
        compile(source, true).as_str(),
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback } from 'vue';\n\nexport function render(_ctx) {\n  const _component_MyComp = _resolveComponent(\"MyComp\")\n  const n0 = _createComponentWithFallback(_component_MyComp, { title: () => (\"t\") }, null, true)\n  return n0\n}\n"
    );
}

#[test]
fn object_payload_mutations_drive_generation() {
    let allocator = Allocator::new();
    let source = r#"<div title="t" v-bind="o"><MyComp v-bind="p" /></div>"#;
    let mut s3 = lowered_source(&allocator, source);
    for operand in &mut s3.program.operands {
        match (operand.role, operand.value.text) {
            (OperandRole::Value, "o") => operand.value.text = "q",
            (OperandRole::Value, "p") => operand.value.text = "r",
            (OperandRole::Attribute, "t") => operand.value.text = "u",
            _ => {}
        }
    }
    let code = generated(admit(s3, &Retained::new(&allocator)), &allocator);
    let render = code.split_once("export").map_or("", |(_, render)| render);
    assert_eq!(
        render,
        " function render(_ctx) {\n  const n1 = t0()\n  const n2 = _child(n1)\n  const _component_MyComp = _resolveComponent(\"MyComp\")\n  _setInsertionState(n1, n2, true)\n  const n0 = _createComponentWithFallback(_component_MyComp, { $: [() => (_ctx.r)] }, null, true)\n  _renderEffect(() => _setDynamicProps(n1, [{ title: \"u\" }, _ctx.q]))\n  return n1\n}\n"
    );
}

#[test]
fn unsupported_object_shapes_select_exact_legacy_reasons() {
    use LegacyReason::{Binding, Component, ExpressionOrEncoding};
    for (source, reason) in [
        // Named listeners beside an object: their order differs by runtime.
        (r#"<div v-bind="o" @click="go"></div>"#, Binding),
        (r#"<button v-on="h" @click="go"></button>"#, Binding),
        (r#"<div v-bind="o" v-on="h"></div>"#, Binding),
        (r#"<div v-bind="o" v-show="s"></div>"#, Binding),
        (r#"<input v-bind="o" v-model="m">"#, Binding),
        (r#"<div id="a" v-bind="o" :id="b"></div>"#, Binding),
        (r#"<div v-bind.prop="o"></div>"#, Binding),
        (r#"<slot v-bind="o" />"#, Component),
        (r#"<div v-bind="$attrs"></div>"#, ExpressionOrEncoding),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Legacy(actual) if actual == reason),
            "{source}: expected {reason:?}, got {status:?}"
        );
    }
}

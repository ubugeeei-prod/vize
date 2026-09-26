//! Exact span-carrying Vapor source maps (Davinci P3-9).
//!
//! Every test pins the full generated code and every decoded segment, rendered
//! as `generated text -> authored text [name]`, for both the legacy lowering
//! lane and the native L3 lane. The decoder is local on purpose: it must not
//! share code with the encoder it checks.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

mod component_names;
mod computed_dom;
mod model_names;
mod slot_props;
mod structural_slots;
mod suspense;
mod transition;

mod support;

use super::{VaporCompilerOptions, compile_vapor};
use support::{Lane, mapped, mapped_on};
use vize_carton::Allocator;

#[test]
fn source_map_disabled_by_default_yields_none() {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        "<div>{{ msg }}</div>",
        VaporCompilerOptions::default(),
    );

    assert!(result.error_messages.is_empty(), "Expected no errors");
    assert!(result.map.is_none());
}

#[test]
fn both_lanes_map_templates_branches_and_loops_byte_exactly() {
    let source =
        "<ul>\n<li v-if=\"ok\">Y</li><li v-else>N</li><i v-for=\"x in xs\">{{ x }}</i>\n</ul>";
    // The native L3 lane admits this template; binding metadata forces the
    // legacy lane. Both must emit the same code with the same map.
    let (code, segments) = mapped(source);
    assert_eq!(
        mapped_on(source, Lane::Legacy),
        (code.clone(), segments.clone())
    );
    assert_eq!(
        code,
        "import { txt as _txt, toDisplayString as _toDisplayString, setText as _setText, setInsertionState as _setInsertionState, renderEffect as _renderEffect, createIf as _createIf, createFor as _createFor, template as _template } from 'vue';\nconst t0 = _template(\"<li>Y</li>\", true)\nconst t1 = _template(\"<li>N</li>\", true)\nconst t2 = _template(\"<i> </i>\")\nconst t3 = _template(\"<ul></ul>\", true)\n\nexport function render(_ctx) {\n  const n0 = t3()\n  _setInsertionState(n0)\n  const n1 = _createIf(() => (_ctx.ok), () => {\n    const n3 = t0()\n    return n3\n  }, () => {\n    const n5 = t1()\n    return n5\n  })\n  _setInsertionState(n0, 1)\n  const n6 = _createFor(() => (_ctx.xs), (_for_item0) => {\n    const n8 = t2()\n    const x8 = _txt(n8)\n    _renderEffect(() => _setText(x8, _toDisplayString(_for_item0.value)))\n    return n8\n  })\n  return n0\n}\n"
    );
    assert_eq!(
        segments,
        [
            r#"1:23 "li>Y</li" -> "li v-if=""#,
            r#"1:26 "Y</li>\"," -> "Y</li><l""#,
            r#"2:23 "li>N</li" -> "li v-els""#,
            r#"2:26 "N</li>\"," -> "N</li><i""#,
            r#"3:23 "i> </i>\"" -> "i v-for=""#,
            r#"4:23 "ul></ul>" -> "ul>⏎<li ""#,
            r#"6:0 "export f" -> "<ul>⏎<li""#,
            r#"9:13 "_createI" -> "<li v-if""#,
            r#"9:30 "_ctx.ok)" -> "ok\">Y</l" [ok]"#,
            r#"12:5 "() => {⏎" -> "<li v-el""#,
            r#"17:13 "_createF" -> "<i v-for""#,
            r#"17:31 "_ctx.xs)" -> "xs\">{{ x" [xs]"#,
            r#"20:54 "_for_ite" -> "x }}</i>""#,
        ]
    );
}

#[test]
fn native_lane_maps_templates_bindings_and_text_byte_exactly() {
    let source = "<main class=\"shell\"><span :title=\"hint\">{{ status }}</span></main>";
    let (code, segments) = mapped(source);
    assert_eq!(
        code,
        "import { child as _child, txt as _txt, toDisplayString as _toDisplayString, setText as _setText, setProp as _setProp, renderEffect as _renderEffect, template as _template } from 'vue';\nconst t0 = _template(\"<main class=\\\"shell\\\"><span> </span></main>\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  const n1 = _child(n0)\n  const x1 = _txt(n1)\n  _renderEffect(() => {\n    _setProp(n1, \"title\", _ctx.hint)\n    _setText(x1, _toDisplayString(_ctx.status))\n  })\n  return n0\n}\n"
    );
    assert_eq!(
        segments,
        [
            r#"1:23 "main cla" -> "main cla""#,
            r#"1:28 "class=\\\"" -> "class=\"s""#,
            r#"1:36 "shell\\\">" -> "shell\"><""#,
            r#"1:45 "span> </" -> "span :ti""#,
            r#"3:0 "export f" -> "<main cl""#,
            r#"8:18 "title\", " -> "title=\"h""#,
            r#"8:26 "_ctx.hin" -> "hint\">{{" [hint]"#,
            r#"9:34 "_ctx.sta" -> "status }" [status]"#,
        ]
    );
}

/// Template carriers, compound text and `v-model` admit on the native lane and
/// must keep the legacy lane's map. Component and slot props still carry no
/// native spans (P3-9).
#[test]
fn both_lanes_map_template_carriers_the_same() {
    for source in [
        r#"<div><template v-if="ok"><b>{{ a }}</b><i>x</i></template><span v-else>no</span></div>"#,
        r#"<ul><template v-for="item in items" :key="item.id"><li>{{ item.a }}</li><li>{{ item.b }}</li></template></ul>"#,
        r#"<template v-if="ok"><b>{{ a }}</b><i>x</i></template><template v-else>none</template>"#,
        r#"<main><template v-if="on">on {{ n }}</template><template v-else>off</template></main>"#,
        r#"<main><template v-for="x in xs">item {{ x }}</template></main>"#,
        r#"<ul><li v-if="a > b">Y</li><li v-else-if="c < d">N</li></ul>"#,
        r#"<input v-model="name">"#,
    ] {
        assert_eq!(
            mapped_on(source, Lane::Selected),
            mapped_on(source, Lane::Legacy),
            "{source}"
        );
    }
}

#[test]
fn computed_slot_names_keep_their_authored_mapping_units() {
    for (case, source) in [
        (
            "template",
            r#"<MyComp><template #[names[selected]]="{ item }"><b>{{ item }}</b></template></MyComp>"#,
        ),
        (
            "component",
            r#"<MyComp v-slot:[names[selected]]="{ item }"><b>{{ item }}</b></MyComp>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        insta::assert_snapshot!(format!("computed_slot_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("computed_slot_{case}_map"), segments);
    }
}

#[test]
fn computed_events_keep_name_and_handler_anchors_in_both_lanes() {
    for (case, source) in [
        ("reference", r#"<button @[eventName]="save">go</button>"#),
        (
            "member",
            r#"<button v-on:[names[selected]].once.capture.passive="save">go</button>"#,
        ),
        (
            "compound",
            r#"<button @[enabled?first:second].enter.stop="save">go</button>"#,
        ),
        (
            "call",
            r#"<button @[eventName.toLowerCase()].right="save">go</button>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            mapped_on(source, Lane::Legacy),
            (code.clone(), segments.clone()),
            "{source}"
        );
        insta::assert_snapshot!(format!("computed_event_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("computed_event_{case}_map"), segments);
    }
}

#[test]
fn teleport_props_and_contents_keep_authored_mapping_units() {
    for (case, source) in [
        (
            "static",
            r#"<Teleport to="body"><div>content</div></Teleport>"#,
        ),
        (
            "reactive",
            r#"<Teleport :to="target" :disabled="disabled" defer><span>{{ label }}</span></Teleport>"#,
        ),
        (
            "indexed",
            r#"<Teleport :to="targets[selected]" :defer="deferred"><span v-if="visible">{{ value }}</span><span v-else>fallback</span></Teleport>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        let (legacy_code, legacy_segments) = mapped_on(source, Lane::Legacy);
        assert_eq!(code, legacy_code, "{case}: generated code");
        assert_eq!(segments, legacy_segments, "{case}: authored spans");
        insta::assert_snapshot!(format!("teleport_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("teleport_{case}_map"), segments);
    }
}

#[test]
fn keep_alive_props_and_child_keep_authored_mapping_units() {
    for (case, source) in [
        (
            "static",
            r#"<KeepAlive include="First" max="2"><MyComp /></KeepAlive>"#,
        ),
        (
            "dynamic",
            r#"<KeepAlive :include="names" :max="limit"><component :is="views[selected]" :label="label" /></KeepAlive>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            (code.clone(), segments.clone()),
            mapped_on(source, Lane::Legacy),
            "{case}: code and every decoded segment"
        );
        insta::assert_snapshot!(format!("keep_alive_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("keep_alive_{case}_map"), segments);
    }
}

#[test]
fn select_models_keep_authored_option_maps() {
    for (case, source) in [
        (
            "single",
            r#"<select v-model="selected"><option value="a">A</option><option value="b">B</option></select>"#,
        ),
        (
            "bound",
            r#"<select multiple v-model="selected"><option :value="first">{{ label }}</option><option :value="second">B</option></select>"#,
        ),
        (
            "computed",
            r#"<select v-model="form[key]"><option value="a">A</option></select>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        insta::assert_snapshot!(format!("select_model_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("select_model_{case}_map"), segments);
    }
}

#[test]
fn once_event_handlers_keep_authored_mapping_units() {
    for (case, source) in [
        (
            "reference",
            r#"<button v-once :title="label" @click="save">{{ label }}</button>"#,
        ),
        (
            "callback",
            r#"<button v-once @click="(event) => record(event.type)">{{ label }}</button>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            (code.clone(), segments.clone()),
            mapped_on(source, Lane::Legacy),
            "{case}: code and every decoded segment"
        );
        insta::assert_snapshot!(format!("once_event_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("once_event_{case}_map"), segments);
    }
}

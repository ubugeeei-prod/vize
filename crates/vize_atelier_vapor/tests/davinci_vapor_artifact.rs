//! Production routing evidence in its own process: the monotone walk/reparse
//! probes cannot be mixed with concurrently running compiler tests.

use vize_atelier_core::{TemplateSyntaxMode, expr_parse_probe, walk_probe::WalkCounts};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor,
    compile_vapor_with_diagnostics, compile_vapor_with_sfc_context,
};
use vize_carton::Allocator;

#[test]
fn accepted_artifacts_bypass_legacy_walks_and_unsupported_inputs_keep_them() {
    let accepted = [
        r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#,
        r#"<section><span>fixed</span><div :title="state.title"><b>{{ state.label }}</b></div><button @click="actions.save">{{ label }}</button></section>"#,
        "<div><span>static</span>{{ label }}<br><b>tail</b></div>",
        "<div><section><span>static 雪</span></section><hr></div>",
        "<div>hello {{ name }}!<span>{{ a }}{{ b }}</span>tail {{ end }}</div>",
        "<main><b>fixed</b>{{ a }} / {{ b }}<span :title=\"name\">next</span>{{ end }}</main>",
        "<main @keydown.enter=\"save\"><button @click.stop=\"save\">{{ label }}</button></main>",
        "<div @focus=\"save\" @change.once=\"save\" @custom-event.capture=\"save\"></div>",
        "<div v-if=\"ready\">{{ label }}</div>",
        "<div v-if=\"a\">A</div><div v-else-if=\"b\">B</div><div v-else>C</div>",
        "<div><span v-if=\"a\">A</span><b v-else>{{ label }}</b><i>tail</i></div>",
        "<ul><li v-for=\"item in items\" :key=\"item.id\" :title=\"item.title\">{{ item.label }}</li></ul>",
        "<span v-for=\"(item, name, position) in items\">{{ position }}</span>",
        "<section><span>head</span><div v-for=\"(row, i) in rows\" :key=\"row.id\" @click=\"save\">{{ i }}: {{ row.label }}<b v-if=\"row.ok\">ok</b></div><span>tail {{ n }}</span></section>",
        "<main><section v-if=\"open\"><b>title</b><span v-for=\"n in 3\">{{ n }}</span></section></main>",
        // The mounted `native_control` scenarios, pinned to the native lane.
        r#"<main data-id="root"><button v-if="mode" data-id="yes" :title="label" @click="save">{{ label }}</button><span v-else-if="alt" data-id="alt">{{ label }}</span><i v-else data-id="none">none</i><b data-id="tail">{{ tail }}</b></main>"#,
        r#"<main data-id="root"><button v-for="(item, position) in items" :key="item.id" :data-id="item.id" :title="position" @click="save">{{ item.label }}</button><span data-id="tail">{{ item }}</span></main>"#,
        r#"<main data-id="root"><button v-for="(item, position) in items" :data-id="item.id" :title="position" @click="save">{{ item.label }}</button><span data-id="tail">{{ item }}</span></main>"#,
        r#"<main data-id="root"><section v-if="open" data-id="panel"><b data-id="title">{{ title }}</b><span v-for="row in rows" :key="row.id" :data-id="row.id">{{ row.label }}</span><ul data-id="cells"><li v-for="cell in cells" :key="cell" :data-id="cell">{{ cell }}</li></ul></section><i data-id="tail">tail</i></main>"#,
        r#"<main><span v-for="(value, name, position) in entries" :key="name" :data-name="name" :title="position">{{ value }}</span><b v-for="n in 3">{{ n }}</b></main>"#,
        r#"<b v-if="a" :title="label">{{ label }}</b><i v-else-if="b">B</i><span v-else>none</span>"#,
        r#"<b v-for="item in items" :key="item">{{ item }}</b>"#,
        // The `vapor_native_pair/control_flow` bench fixture.
        r#"<main><section v-if="open"><b>{{ title }}</b><span v-for="row in rows" :key="row.id" :title="row.title">{{ row.label }}</span></section><i v-else>closed</i><ul><li v-for="(cell, i) in cells" @click="save">{{ i }}: {{ cell }}</li></ul></main>"#,
    ];
    for source in accepted {
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let options = VaporCompilerOptions {
                prefix_identifiers,
                ..Default::default()
            };
            let before = WalkCounts::snapshot();
            let parses = expr_parse_probe::expr_parse_count();
            let result = compile_vapor(&allocator, source, options.clone());
            assert!(
                result.error_messages.is_empty(),
                "{source}: {:?}",
                result.error_messages
            );
            assert_eq!(
                WalkCounts::snapshot().since(before).total_walks(),
                0,
                "{source}"
            );
            assert_eq!(expr_parse_probe::expr_parse_count() - parses, 0, "{source}");
            let before = WalkCounts::snapshot();
            let legacy = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    binding_metadata: Some(Default::default()),
                    ..options
                },
            );
            assert!(legacy.error_messages.is_empty());
            assert_eq!(
                WalkCounts::snapshot().since(before).total_walks(),
                2,
                "{source}"
            );
            assert_eq!(
                result.templates, legacy.templates,
                "template payload changed: {source}"
            );
        }
    }
    let allocator = Allocator::new();
    let source =
        "<main><button :disabled=\"locked\">{{ label }}</button><span>static</span></main>";
    let before = WalkCounts::snapshot();
    let (scoped, diagnostics) = compile_vapor_with_sfc_context(
        &allocator,
        source,
        VaporCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        Default::default(),
        VaporCompilerExperimentalOptions {
            source_map: true,
            source_map_filename: Some("Native.vue".into()),
            ..Default::default()
        },
        Some("data-v-native"),
    );
    assert!(diagnostics.is_empty());
    assert_eq!(WalkCounts::snapshot().since(before).total_walks(), 0);
    assert_eq!(
        scoped.templates[0],
        "<main data-v-native><button data-v-native> </button><span data-v-native>static</span></main>"
    );
    let map: serde_json::Value = serde_json::from_str(scoped.map.as_deref().unwrap()).unwrap();
    assert_eq!(map["sources"], serde_json::json!(["Native.vue"]));
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    for source in [
        "<div>{{ value + 1 }}</div>",
        "<template v-if=\"ok\"><b>{{ value }}</b></template>",
        "<button @click=\"save()\">{{ label }}</button>",
        "<div v-pre>{{ raw }}</div>",
    ] {
        let allocator = Allocator::new();
        let before = WalkCounts::snapshot();
        let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
        assert!(
            result.error_messages.is_empty(),
            "{source}: {:?}",
            result.error_messages
        );
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            2,
            "{source}"
        );
    }
    for handler in ["$event", "$event.target"] {
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let source = vize_carton::cstr!(r#"<button @click="{handler}">Click</button>"#);
            let result = compile_vapor(
                &allocator,
                &source,
                VaporCompilerOptions {
                    prefix_identifiers,
                    ..Default::default()
                },
            );
            let expected = vize_carton::cstr!(
                r#"import {{ createInvoker as _createInvoker, delegateEvents as _delegateEvents, template as _template }} from 'vue';
const t0 = _template("<button>Click</button>", true)
_delegateEvents("click")

export function render(_ctx) {{
  const n0 = t0()
  n0.$evtclick = _createInvoker(e => _ctx.{handler}(e))
  return n0
}}
"#
            );
            assert_eq!(result.code, expected, "{handler}");
        }
    }
    for source in ["<div>", "<div v-if />"] {
        let allocator = Allocator::new();
        let (result, diagnostics) =
            compile_vapor_with_diagnostics(&allocator, source, VaporCompilerOptions::default());
        assert!(
            !result.error_messages.is_empty() || !diagnostics.is_empty(),
            "{source}"
        );
        if diagnostics
            .iter()
            .any(|diagnostic| !diagnostic.is_recoverable())
        {
            assert!(
                result.code.is_empty(),
                "fatal diagnostics must not emit: {source}"
            );
        }
    }
    // With prefixing disabled, the established tolerant API preserves invalid
    // expressions. It must terminate. With prefixing enabled, core expression
    // validation reports them and the compiler must not emit code.
    for expression in ["(", "x +", "[x +", "{value: x +}"] {
        let source = vize_carton::cstr!("<button :disabled=\"{expression}\"></button>");
        let allocator = Allocator::new();
        let tolerant = compile_vapor(&allocator, &source, VaporCompilerOptions::default());
        let expected = vize_carton::cstr!(
            r#"import {{ setProp as _setProp, renderEffect as _renderEffect, template as _template }} from 'vue';
const t0 = _template("<button></button>", true)

export function render(_ctx) {{
  const n0 = t0()
  _renderEffect(() => _setProp(n0, "disabled", {expression}))
  return n0
}}
"#
        );
        assert_eq!(tolerant.code, expected, "{expression}");
        let (strict, diagnostics) = compile_vapor_with_diagnostics(
            &allocator,
            &source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(!diagnostics.is_empty(), "{expression}");
        assert!(!strict.error_messages.is_empty(), "{expression}");
        assert!(strict.code.is_empty(), "{expression}");
    }
}

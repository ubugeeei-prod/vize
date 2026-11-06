//! Under `is_ts` the Vapor generator type-erases every authored expression
//! it parses, so render code is JavaScript on both lanes, in the positions
//! the DOM lanes cover (vize_atelier_dom `l2_is_ts.rs`).

use vize_atelier_core::options::{
    BindingMetadata, BindingType, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_sfc_context,
};
use vize_carton::{Allocator, FxHashMap, String, cstr};

const POSITIONS: &[(&str, &str)] = &[
    (
        "return_type",
        r#"<div @click="(e): void => pick(e)"></div>"#,
    ),
    (
        "typed_function_expression",
        r#"<div @click="function (e: Event) { pick(e) }"></div>"#,
    ),
    (
        "this_param_and_return_type",
        r#"<div :fn="function (this: Window, x: number): number { return x + count }"></div>"#,
    ),
    ("instantiation", "<div>{{ pick<Item> }}</div>"),
    (
        "callback_return_type",
        "<div>{{ list.map((x): number => x * 2) }}</div>",
    ),
    (
        "typed_declaration_in_body",
        "<div>{{ (() => { const y: number = count; return y })() }}</div>",
    ),
    ("angle_assertion", "<div>{{ <any>count }}</div>"),
    (
        "optional_param",
        "<div>{{ ((x?: number) => x)(count) }}</div>",
    ),
    (
        "typed_statements",
        r#"<div @click="const n: number = count; pick(n)"></div>"#,
    ),
    (
        "typed_for_aliases",
        r#"<li v-for="(item: Item, i: number) in list" :key="i">{{ item }}</li>"#,
    ),
    (
        "typed_slot_props",
        r#"<Card #default="{ row }: { row: Row }">{{ row }}</Card>"#,
    ),
    (
        "typed_slot_param",
        r#"<Card v-slot="props: Props">{{ props }}</Card>"#,
    ),
    ("interp_as", "<div>{{ count as number }}</div>"),
    ("non_null", "<div>{{ maybe!.name }}</div>"),
    ("bind_as", r#"<div :id="raw as string"></div>"#),
    (
        "handler_typed_param",
        r#"<div @click="(e: MouseEvent) => pick(e)"></div>"#,
    ),
    (
        "handler_as_argument",
        r#"<div @click="pick(count as number)"></div>"#,
    ),
    ("model_as", r#"<input v-model="(raw as any).x">"#),
    (
        "modifiers_satisfies",
        r#"<div @click.stop="pick(count satisfies number)" @keyup.enter="(e: KeyboardEvent): void => pick(e.key)"></div>"#,
    ),
    (
        "structural_slot_for",
        r#"<Card><template v-for="(item: Item, i: number) in list" #[item]="{ row }: { row: Row }">{{ row }}{{ i }}</template></Card>"#,
    ),
    (
        "slot_default_as",
        r#"<Card #default="{ row = count as number }: { row?: number }">{{ row }}</Card>"#,
    ),
    (
        "for_default_as",
        r#"<li v-for="({ id = count as number }: Row, i: number) in list" :key="id">{{ id }}</li>"#,
    ),
    (
        "for_alias_reads",
        r#"<li v-for="(item: Item) in list" :key="(item as Item).id" @click="pick(item as Item)">{{ (item as Item).id }}</li>"#,
    ),
    (
        "component_show_dynamic_arg",
        r#"<component :is="(pick as any)" v-show="count as boolean" :[key!]="1" />"#,
    ),
    (
        "html_text_class",
        r#"<div v-html="(count as any)" v-text="count!" :class="{ a: count! }"></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let mut bindings = FxHashMap::default();
    for (name, kind) in [
        ("count", BindingType::SetupRef),
        ("raw", BindingType::SetupLet),
        ("list", BindingType::SetupConst),
        ("maybe", BindingType::SetupRef),
        ("key", BindingType::SetupRef),
        ("pick", BindingType::SetupConst),
        ("Card", BindingType::SetupConst),
    ] {
        bindings.insert(name.into(), kind);
    }
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    }
}

/// The SFC's Vapor compile: `(error messages, code)`. `retained` pins the
/// legacy lane; otherwise the native lane is selected where it admits.
fn compile(src: &str, is_ts: bool, retained: bool) -> (Vec<String>, String) {
    let allocator = Allocator::new();
    let (result, _) = compile_vapor_with_sfc_context(
        &allocator,
        src,
        VaporCompilerOptions {
            binding_metadata: Some(metadata()),
            is_ts,
            davinci_retained_lane: retained,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
        None,
    );
    (result.error_messages, result.code)
}

fn javascript_diagnostics(code: &str) -> Vec<String> {
    let allocator = oxc_allocator::Allocator::default();
    oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs())
        .parse()
        .diagnostics
        .iter()
        .map(|diagnostic| cstr!("{diagnostic}"))
        .collect()
}

/// The `render` function, from its signature to its closing brace.
fn render(code: &str) -> String {
    let mut out = String::default();
    let mut inside = false;
    for line in code.lines() {
        inside |= line == "export function render(_ctx) {";
        if inside {
            out.push_str(line);
            out.push('\n');
        }
        if inside && line == "}" {
            break;
        }
    }
    out
}

#[test]
fn every_authored_position_leaves_as_javascript_on_both_lanes() {
    for retained in [false, true] {
        for (name, src) in POSITIONS {
            let (errors, code) = compile(src, true, retained);
            assert_eq!(
                errors,
                Vec::<String>::new(),
                "{name} (retained: {retained})"
            );
            assert_eq!(
                javascript_diagnostics(&code),
                Vec::<String>::new(),
                "{name} (retained: {retained}):\n{code}"
            );
        }
    }
}

/// Erasure runs before identifier resolution, so a type is never resolved
/// as a value (`e: _ctx.Event`, `as _ctx.Item` before this lane erased).
#[test]
fn erased_expressions_resolve_as_javascript() {
    let cases: [(&str, &str); 6] = [
        (
            r#"<div @click="function (e: Event) { pick(e) }"></div>"#,
            "export function render(_ctx) {\n  const n0 = t0()\n  _on(n0, \"click\", function(e) {\n\t_ctx.pick(e);\n})\n  return n0\n}\n",
        ),
        (
            "<div>{{ list.map((x): number => x * 2) }}</div>",
            "export function render(_ctx) {\n  const n0 = t0()\n  const x0 = _txt(n0)\n  _renderEffect(() => _setText(x0, _toDisplayString(_ctx.list.map((x) => x * 2))))\n  return n0\n}\n",
        ),
        (
            r#"<div @click="const n: number = count; pick(n)"></div>"#,
            "export function render(_ctx) {\n  const n0 = t0()\n  _on(n0, \"click\", ($event) => {\n\tconst n = _ctx.count;\n\t_ctx.pick(n);\n})\n  return n0\n}\n",
        ),
        (
            r#"<li v-for="(item: Item) in list" :key="(item as Item).id" @click="pick(item as Item)">{{ (item as Item).id }}</li>"#,
            "export function render(_ctx) {\n  const n0 = _createFor(() => (_ctx.list), (_for_item0) => {\n    const n2 = t0()\n    const x2 = _txt(n2)\n    _on(n2, \"click\", ($event) => _ctx.pick(_for_item0.value))\n    _renderEffect(() => _setText(x2, _toDisplayString(_for_item0.value.id)))\n    return n2\n  }, (item) => (item.id), 8 /* IS_SINGLE_NODE */)\n  return n0\n}\n",
        ),
        (
            r#"<Card #default="{ row = count as number }: { row?: number }">{{ row }}</Card>"#,
            "export function render(_ctx) {\n  const _component_Card = _ctx.Card\n  const n1 = _createComponentWithFallback(_component_Card, null, {\n    \"default\": (_slotProps0) => {\n      const n0 = t0()\n      _renderEffect(() => _setText(n0, _toDisplayString(_getDefaultValue(_slotProps0.row, () => (_ctx.count)))))\n      return n0\n    }\n  }, true)\n  return n1\n}\n",
        ),
        (
            r#"<div v-html="(count as any)" v-text="count!" :class="{ a: count! }"></div>"#,
            "export function render(_ctx) {\n  const n0 = t0()\n  _renderEffect(() => {\n    _setHtml(n0, _ctx.count)\n    _setElementText(n0, _toDisplayString(_ctx.count))\n    _setClass(n0, { a: _ctx.count })\n  })\n  return n0\n}\n",
        ),
    ];
    for retained in [false, true] {
        for (src, expected) in cases {
            assert_eq!(
                render(&compile(src, true, retained).1),
                expected,
                "{src} (retained: {retained})"
            );
        }
    }
}

/// Negative control: without `is_ts` the lane erases nothing, so a
/// JavaScript source keeps its bytes.
#[test]
fn without_is_ts_the_authored_bytes_are_kept() {
    for retained in [false, true] {
        assert_eq!(
            render(&compile("<div>{{ count as number }}</div>", false, retained).1),
            "export function render(_ctx) {\n  const n0 = t0()\n  const x0 = _txt(n0)\n  _renderEffect(() => _setText(x0, _toDisplayString(_ctx.count as number)))\n  return n0\n}\n",
            "retained: {retained}"
        );
    }
}

/// Invalid text is rejected by the native lane before it can reach a module;
/// the explicitly retained compatibility lane keeps its historical bytes.
#[test]
fn text_that_is_not_typescript_is_not_erased() {
    let expected = Vec::from([String::from(
        "Error parsing JavaScript expression: invalid expression",
    )]);
    for is_ts in [false, true] {
        let (errors, code) = compile("<div>{{ count as }}</div>", is_ts, false);
        assert_eq!(errors, expected, "is_ts: {is_ts}");
        assert_eq!(code, "", "is_ts: {is_ts}");
    }
    let (_, erased) = compile("<div>{{ count as }}</div>", true, true);
    let (_, authored) = compile("<div>{{ count as }}</div>", false, true);
    assert_eq!(erased, authored);
    assert_eq!(javascript_diagnostics(&erased).len(), 1, "{erased}");
}

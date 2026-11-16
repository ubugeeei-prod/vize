use super::{Code, CompileOptions, Diagnostic, EmitIssue, Target, compile};
use vize_l2::lang::tsrx::TsrxIssueKind;

const OPTIONS: CompileOptions<'static> = CompileOptions {
    filename: "App.tsrx",
    target: Target::Dom,
    source_map: false,
    erase_types: false,
};

fn diagnostics(source: &str, options: &CompileOptions<'_>) -> Vec<Diagnostic> {
    compile(source, options).unwrap_err().diagnostics
}

#[test]
fn syntax_errors_carry_the_reference_code_and_offset() {
    // tests/_fixtures/tsrx/errors/code-block-return.tsrx: TSRX2001 at 62..74.
    let source = include_str!("../../../../../tests/_fixtures/tsrx/errors/code-block-return.tsrx");
    let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
    assert_eq!(diagnostic.code, Code::Syntax("TSRX2001".into()));
    assert_eq!((diagnostic.start, diagnostic.end), (62, 74));
}

#[test]
fn unsupported_constructs_are_refused_at_their_span() {
    let source = "export function Pick({ mode }: { mode: string }) @{\n\t@switch (mode) { @case 'a': { <b /> } }\n}\n";
    let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
    assert_eq!(diagnostic.code, Code::Lowering(TsrxIssueKind::Switch));
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("@switch (mode) { @case 'a': { <b /> } }")
    );

    // A destructured item without a key or an index has no value to key by.
    let source = "export function List({ items }: { items: { a: string }[] }) @{\n\t@for (const { a } of items) {\n\t\t<li>{a}</li>\n\t}\n}\n";
    let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
    assert_eq!(diagnostic.code, Code::Emit(EmitIssue::Dom));
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("@{\n\t@for (const { a } of items) {\n\t\t<li>{a}</li>\n\t}\n}")
    );
}

#[test]
fn dom_host_loops_outside_the_emitted_shapes_are_refused() {
    for body in [
        "@for (const item of items; key item) {\n\t\t<li>{item}</li>\n\t} @empty {\n\t\t<li>none</li>\n\t}",
        "@for (const item: string of items; key item) {\n\t\t<li>{item}</li>\n\t}",
        "@for (const item of items; key item) {\n\t\t<li key={item}>{item}</li>\n\t}",
        "@for (const item of items) {\n\t\t@if (item) {\n\t\t\t<b>{item}</b>\n\t\t}\n\t}",
    ] {
        let source =
            format!("export function List({{ items }}: {{ items: string[] }}) @{{\n\t{body}\n}}\n");
        let [diagnostic] = diagnostics(&source, &OPTIONS).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Emit(EmitIssue::Dom), "{body}");
    }
}

#[test]
fn unsupported_targets_and_options_are_each_refused_over_the_module() {
    let source = "export function App() @{\n\t<p class=\"x\">hi</p>\n}\n";
    let whole = (0, source.len() as u32);
    let refused = |options: &CompileOptions<'_>| -> Vec<_> {
        diagnostics(source, options)
            .into_iter()
            .map(|diagnostic| (diagnostic.code, (diagnostic.start, diagnostic.end)))
            .collect()
    };
    for target in [Target::Ssr, Target::Vapor] {
        let options = CompileOptions { target, ..OPTIONS };
        assert_eq!(refused(&options), [(Code::Target(target), whole)]);
    }
    let options = CompileOptions {
        source_map: true,
        erase_types: true,
        ..OPTIONS
    };
    let mut expected = vec![(Code::Emit(EmitIssue::SourceMap), whole)];
    if !cfg!(feature = "typescript") {
        expected.push((Code::Emit(EmitIssue::TypeErasure), whole));
    }
    assert_eq!(refused(&options), expected);
    let options = CompileOptions {
        target: Target::Ssr,
        source_map: true,
        ..OPTIONS
    };
    assert_eq!(
        refused(&options),
        [
            (Code::Target(Target::Ssr), whole),
            (Code::Emit(EmitIssue::SourceMap), whole),
        ]
    );
}

#[test]
fn dom_markup_outside_a_component_body_is_refused_at_its_region() {
    for (source, region) in [
        (
            "export const view = () => <p class=\"x\">hi</p>;\n",
            "<p class=\"x\">hi</p>",
        ),
        ("export function view() {\n\treturn <p />;\n}\n", "<p />"),
        (
            "export function App() {\n\tconst x = <i />;\n\treturn <p />;\n}\n",
            "<i />",
        ),
        ("export function App() {\n\treturn [<p />];\n}\n", "<p />"),
        ("export function view() @{\n\t<p />\n}\n", "@{\n\t<p />\n}"),
        (
            "export default function () @{\n\t<p />\n}\n",
            "@{\n\t<p />\n}",
        ),
        (
            "export function App() @{\n\tconst x = <i />;\n\t<p />\n}\n",
            "<i />",
        ),
    ] {
        let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Emit(EmitIssue::OutsideComponent));
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some(region)
        );
    }
}

#[test]
fn dom_component_parameters_outside_the_props_profile_are_refused() {
    for (source, issue) in [
        (
            "export function A(a, b) @{\n\t<p />\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ a = 1 }) @{\n\t<p />\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ a, ...rest }) @{\n\t<p />\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ a: b }) @{\n\t<p>{b}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A(props) @{\n\t<p {...props} />\n}\n",
            EmitIssue::PropsValue,
        ),
        (
            "export function A({ a }) @{\n\t<p>{[1].map((a) => a)}</p>\n}\n",
            EmitIssue::ShadowedProp,
        ),
        (
            "export async function A() @{\n\t<p />\n}\n",
            EmitIssue::AsyncComponent,
        ),
        (
            "export const A = async () => <p />;\n",
            EmitIssue::AsyncComponent,
        ),
        (
            "export function A() @{\n\tconst _ctx = 1;\n\t<p>{_ctx}</p>\n}\n",
            EmitIssue::GeneratedName,
        ),
    ] {
        let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Emit(issue), "{source}");
    }
}

#[test]
fn dom_components_keep_host_code_and_read_destructured_props_through_the_instance() {
    let source = "const unit = \"px\";\nexport function Box({ size }: { size: number }) @{\n\tconst label = size + unit;\n\t<div title={label}>{size}</div>\n}\n";
    let code = compile(source, &OPTIONS).unwrap().code;
    assert_eq!(
        code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode } from \"vue\";\nconst unit = \"px\";\nexport const Box = /* @__PURE__ */ _defineComponent(function Box(__props) {\n\tconst label = __props.size + unit;\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(\"div\", { title: label }, [\n    __props.size\n  ], 8 /* PROPS */, [\"title\"]))\n};\n}, { props: [\"size\"] });\n"
    );
}

#[cfg(feature = "typescript")]
#[test]
fn erased_modules_print_javascript_and_refuse_what_the_erasure_cannot_keep() {
    let erase = CompileOptions {
        erase_types: true,
        ..OPTIONS
    };
    let source = "import type { Size } from \"./size\";\nconst unit: string = \"px\";\nexport function Box({ size }: { size: Size }) @{\n\tconst label = (size as number) + unit;\n\t<div title={label}>{size}</div>\n}\n";
    assert_eq!(
        compile(source, &erase).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode } from \"vue\";\nconst unit = \"px\";\nexport const Box = /* @__PURE__ */ _defineComponent(function Box(__props) {\n  const label = __props.size + unit;\n  return (_ctx, _cache) => {\n    return _createElementVNode(\"div\", { title: label }, [__props.size], 8, [\"title\"]);\n  };\n}, { props: [\"size\"] });\n"
    );

    for (source, message) in [
        (
            "const tab = \"\\t\";\nexport function A() @{\n\t<p>{tab}</p>\n}\n",
            "type erasure of TSRX modules does not keep a tab character inside a literal yet",
        ),
        (
            "let a = 1;\nlet a = 2;\nexport function A() @{\n\t<p>{a}</p>\n}\n",
            "type erasure of the compiled TSRX module failed (semantic): Identifier `a` has already been declared",
        ),
    ] {
        let [diagnostic] = diagnostics(source, &erase).try_into().unwrap();
        assert_eq!(
            diagnostic.code,
            Code::Emit(EmitIssue::TypeErasure),
            "{source}"
        );
        assert_eq!(diagnostic.message.as_str(), message, "{source}");
        assert_eq!((diagnostic.start, diagnostic.end), (0, source.len() as u32));
    }
}

#[test]
fn dom_components_returning_markup_read_setup_once_and_render_the_returned_value() {
    let source = "const unit = \"px\";\nexport function Box({ size }: { size: number }) {\n\tconst label = size + unit;\n\tconst pick = () => {\n\t\treturn label;\n\t};\n\treturn (\n\t\t<div title={pick()}>{size}</div>\n\t);\n}\nexport const Tag = ({ text }: { text: string }) => <b>{text}</b>;\n";
    let code = compile(source, &OPTIONS).unwrap().code;
    assert_eq!(
        code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode } from \"vue\";\nconst unit = \"px\";\nexport const Box = /* @__PURE__ */ _defineComponent(function Box(__props) {\n\tconst label = __props.size + unit;\n\tconst pick = () => {\n\t\treturn label;\n\t};\n\treturn (\n\t\t(_ctx, _cache) => {\n  return (_createElementVNode(\"div\", { title: pick() }, [\n    __props.size\n  ], 8 /* PROPS */, [\"title\"]))\n}\n\t);\n}, { props: [\"size\"] });\nexport const Tag = /* @__PURE__ */ _defineComponent((__props) => (_ctx, _cache) => {\n  return (_createElementVNode(\"b\", null, [\n    __props.text\n  ]))\n}, { props: [\"text\"] });\n"
    );
}

#[test]
fn dom_setup_returns_before_the_render_are_refused() {
    for (source, early) in [
        (
            "export function A({ x }: { x: boolean }) @{\n\tif (!x) {\n\t\treturn null;\n\t}\n\t<p />\n}\n",
            "return null;",
        ),
        (
            "export function A({ x }: { x: boolean }) {\n\tif (x) return null;\n\treturn <p />;\n}\n",
            "return null;",
        ),
    ] {
        let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
        assert_eq!(
            diagnostic.code,
            Code::Emit(EmitIssue::EarlyReturn),
            "{source}"
        );
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some(early)
        );
    }
}

#[test]
fn dom_component_children_are_its_default_slot() {
    let source = "function Child() @{\n\t<p />\n}\nexport function A({ x }: { x: string }) @{\n\t<Child>hello {x}<b>bold</b></Child>\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, createTextVNode as _createTextVNode, withCtx as _withCtx } from \"vue\";\nconst Child = /* @__PURE__ */ _defineComponent(function Child() {\n\treturn (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"p\"))\n};\n});\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_createVNode(Child, null, {\n    default: _withCtx(() => [\n      _createTextVNode(\"hello \"),\n      __props.x,\n      _createElementVNode(\"b\", null, \"bold\")\n    ]),\n    _: 2 /* DYNAMIC */\n  }, 1024 /* DYNAMIC_SLOTS */))\n};\n}, { props: [\"x\"] });\n"
    );
}

#[test]
fn dom_markup_expressions_are_erased_as_typescript() {
    let source = "export function A({ size }: { size: number }) @{\n\t<p title={size as unknown as string} onClick={(e: Event) => e}>{size!}</p>\n}\n";
    let compiled = compile(source, &OPTIONS);
    if cfg!(feature = "typescript") {
        assert_eq!(
            compiled.unwrap().code.as_str(),
            "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(\"p\", {\n    title: __props.size,\n    onClick: (e) => e\n  }, [\n    __props.size\n  ], 8 /* PROPS */, [\"title\", \"onClick\"]))\n};\n}, { props: [\"size\"] });\n"
        );
    } else {
        let [diagnostic] = compiled.unwrap_err().diagnostics.try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Emit(EmitIssue::Dom));
    }
}

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
        // An `@empty` region of several nodes, or on a loop that is a branch.
        "@for (const item of items; key item) {\n\t\t<li>{item}</li>\n\t} @empty {\n\t\t<><li>none</li><li>yet</li></>\n\t}",
        "@if (items) {\n\t\t@for (const item of items; key item) {\n\t\t\t<li>{item}</li>\n\t\t} @empty {\n\t\t\t<li>none</li>\n\t\t}\n\t}",
        "@for await (const item of items; key item) {\n\t\t<li>{item}</li>\n\t}",
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
        // A value that holds markup: a call, and `.map()` passes only an
        // item and an index.
        (
            "export function App({ n }: { n: number }) @{\n\t<p>{f(n, <i />)}</p>\n}\n",
            "<i />",
        ),
        (
            "export function App({ xs }: { xs: string[] }) @{\n\t<p>{xs.map((x, i, all) => <i>{x}</i>)}</p>\n}\n",
            "<i>{x}</i>",
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
fn dom_conditionals_over_nested_markup_render_its_branches_with_the_props_it_reads() {
    let source = "export function A({ ok, n, label }: { ok: boolean; n: number; label: string }) @{\n\t<>\n\t\t{ok ? <b title={label}>{n}</b> : n > 1 ? <i /> : null}\n\t\t{n === 0 && <s>none</s>}\n\t</>\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, Fragment as _Fragment, createCommentVNode as _createCommentVNode } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(_Fragment, null, [\n    (__props.ok)\n      ? (_createElementVNode(\"b\", {\n        key: 0,\n        title: __props.label\n      }, [\n        __props.n\n      ], 8 /* PROPS */, [\"title\"]))\n      : (__props.n > 1)\n        ? (_createElementVNode(\"i\", { key: 1 }))\n      : _createCommentVNode(\"v-if\", true),\n    (__props.n === 0)\n      ? (_createElementVNode(\"s\", { key: 2 }, \"none\"))\n      : _createCommentVNode(\"v-if\", true)\n  ], 64 /* STABLE_FRAGMENT */))\n};\n}, { props: [\"ok\", \"n\", \"label\"] });\n"
    );

    // A loop in nested markup shadows a prop inside its body only.
    let source = "export function A({ ok, x }: { ok: boolean; x: string[] }) @{\n\t<p>{ok ? <ul title={x}>@for (const x of [1]) { <li>{x}</li> }</ul> : null}</p>\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, Fragment as _Fragment, createCommentVNode as _createCommentVNode, renderList as _renderList } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(\"p\", null, [\n    (__props.ok)\n      ? (_createElementVNode(\"ul\", {\n        key: 0,\n        title: __props.x\n      }, [\n        (_createElementVNode(_Fragment, null, _renderList([1], (x) => {\n          return (_createElementVNode(\"li\", { key: x }, [\n            x\n          ]))\n        }), 128 /* KEYED_FRAGMENT */))\n      ], 8 /* PROPS */, [\"title\"]))\n      : _createCommentVNode(\"v-if\", true)\n  ]))\n};\n}, { props: [\"ok\", \"x\"] });\n"
    );

    // A conditional as a branch's only node is a keyed fragment, as Vue
    // wraps `<template v-if>` around a nested `v-if`.
    let source = "export function A({ a, b }: { a: boolean; b: boolean }) @{\n\t<p>{a ? (b ? <i /> : <s />) : null}</p>\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, openBlock as _openBlock, createElementBlock as _createElementBlock, Fragment as _Fragment, createCommentVNode as _createCommentVNode } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"p\", null, [\n    (__props.a)\n      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [\n        (__props.b)\n          ? (_openBlock(), _createElementBlock(\"i\", { key: 0 }))\n          : (_openBlock(), _createElementBlock(\"s\", { key: 1 }))\n      ], 64 /* STABLE_FRAGMENT */))\n      : _createCommentVNode(\"v-if\", true)\n  ]))\n};\n}, { props: [\"a\", \"b\"] });\n"
    );

    // An empty branch has nothing to key and stays refused.
    let source = "export function A({ a }: { a: boolean }) @{\n\t<p>{a ? <></> : null}</p>\n}\n";
    let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
    assert_eq!(diagnostic.code, Code::Emit(EmitIssue::Dom));
}

#[test]
fn dom_value_ands_render_a_falsy_left_as_the_child_and_key_their_region() {
    // `n` is evaluated once and is the child when falsy; the region takes the
    // next branch key after the sibling conditional's.
    let source = "export function A({ ok, n, label }: { ok: boolean; n: number; label: string }) @{\n\t<p>\n\t\t{ok ? <b /> : null}\n\t\t{n && <i title={label}>{n}</i>}\n\t</p>\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(\"p\", null, [\n    (__props.ok)\n      ? (_createElementVNode(\"b\", { key: 0 }))\n      : _createCommentVNode(\"v-if\", true),\n    (__props.n)\n      && (_createElementVNode(\"i\", {\n        key: 1,\n        title: __props.label\n      }, [\n        __props.n\n      ], 8 /* PROPS */, [\"title\"]))\n  ]))\n};\n}, { props: [\"ok\", \"n\", \"label\"] });\n"
    );

    // A value `&&` as a branch's only node, or over a conditional, is a
    // keyed fragment like a nested conditional.
    let source = "export function A({ ok, n }: { ok: boolean; n: number }) @{\n\t<p>{ok ? (n && <i />) : null}{n && (ok ? <i /> : <s />)}</p>\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, Fragment as _Fragment, createCommentVNode as _createCommentVNode } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) {\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(\"p\", null, [\n    (__props.ok)\n      ? (_createElementVNode(_Fragment, { key: 0 }, [\n        (__props.n)\n          && (_createElementVNode(\"i\", { key: 0 }))\n      ], 64 /* STABLE_FRAGMENT */))\n      : _createCommentVNode(\"v-if\", true),\n    (__props.n)\n      && (_createElementVNode(_Fragment, { key: 1 }, [\n        (__props.ok)\n          ? (_createElementVNode(\"i\", { key: 0 }))\n          : (_createElementVNode(\"s\", { key: 1 }))\n      ], 64 /* STABLE_FRAGMENT */))\n  ]))\n};\n}, { props: [\"ok\", \"n\"] });\n"
    );
}

#[test]
fn dom_component_parameters_outside_the_props_profile_are_refused() {
    for (source, issue) in [
        (
            "export function A(a, b) @{\n\t<p />\n}\n",
            EmitIssue::Parameters,
        ),
        // The reference drops these defaults.
        (
            "export function A({ a = -1 }) @{\n\t<p>{a}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ a = f() }) @{\n\t<p>{a}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        // Options are evaluated outside the component.
        (
            "export function A({ a, b = () => a }) @{\n\t<p>{b()}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ a: { b } }) @{\n\t<p>{b}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ 'a': a }) @{\n\t<p>{a}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export function A({ a: b, b: c }) @{\n\t<p>{b}{c}</p>\n}\n",
            EmitIssue::Parameters,
        ),
        (
            "export const A = ({ a, ...rest }) => <p {...rest}>{a}</p>;\n",
            EmitIssue::Parameters,
        ),
        // No setup to bind the props and attributes in.
        (
            "export const A = (props) => <p {...props} />;\n",
            EmitIssue::PropsValue,
        ),
        // A destructured prop as a component tag (#13280).
        (
            "export function A({ Icon }) @{\n\t<Icon />\n}\n",
            EmitIssue::PropsValue,
        ),
        // A declaration in the component's own scope; nested scopes may shadow.
        (
            "export function A({ a }) @{\n\tif (a) {\n\t\tvar a = 1;\n\t}\n\t<p>{a}</p>\n}\n",
            EmitIssue::ShadowedProp,
        ),
        (
            "export function A({ a }) @{\n\tfunction a() {}\n\t<p>{a}</p>\n}\n",
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

    // A generated name is reported where the host first declares or reads it.
    let source =
        "export function A({ n }: { n: number }) @{\n\tconst _cache = n;\n\t<p>{_cache}</p>\n}\n";
    let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
    assert_eq!(diagnostic.code, Code::Emit(EmitIssue::GeneratedName));
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("_cache")
    );
}

#[test]
fn dom_prop_defaults_renames_and_rest_follow_the_reference_props_options() {
    let source = "export function A({ a = 'x'!, b: c = undefined, d = null, e = {}, ...rest }: Props) {\n\tconst f = { c };\n\treturn <p {...rest}>{a}{c}{d}{e}{f.c}</p>;\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "import { defineComponent as _defineComponent, useAttrs as _useAttrs, createElementVNode as _createElementVNode, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A(__props) { const rest = _useAttrs();\n\tconst f = { c: __props.b };\n\treturn (_ctx, _cache) => {\n  return (_createElementVNode(\"p\", _normalizeProps(_guardReactiveProps(rest)), [\n    __props.a,\n    __props.b,\n    __props.d,\n    __props.e,\n    f.c\n  ], 16 /* FULL_PROPS */))\n};\n}, { inheritAttrs: false, props: { \"a\": { required: true, type: String, default: 'x' }, \"b\": { default: undefined }, \"d\": { default: null }, \"e\": { type: Object, default: () => ({}) } } });\n"
    );
}

#[test]
fn dom_setup_reads_a_nested_scope_shadows_stay_local() {
    let source = "export function A({ a, xs }: { a: string; xs: string[] }) @{\n\tconst f = (a: string) => a;\n\tfunction g(b = a) {\n\t\tvar a = b;\n\t\treturn a;\n\t}\n\tlet n = 0;\n\tfor (const a of xs) {\n\t\tn += a.length;\n\t}\n\ttry {\n\t\tn += a.length;\n\t} catch (a) {\n\t\tn = a;\n\t}\n\t{\n\t\tclass a {}\n\t\tn += new a() ? 1 : 0;\n\t}\n\tconst o = { a, m(a: string) { return a; } };\n\t<p>{f(a)}{g()}{n}{o.a}{xs.map((a) => a)}</p>\n}\n";
    let code = compile(source, &OPTIONS).unwrap().code;
    let setup = code
        .split("return (_ctx, _cache)")
        .next()
        .and_then(|head| head.split("function A(__props) {").nth(1))
        .unwrap();
    assert_eq!(
        setup,
        "\n\tconst f = (a: string) => a;\n\tfunction g(b = __props.a) {\n\t\tvar a = b;\n\t\treturn a;\n\t}\n\tlet n = 0;\n\tfor (const a of __props.xs) {\n\t\tn += a.length;\n\t}\n\ttry {\n\t\tn += __props.a.length;\n\t} catch (a) {\n\t\tn = a;\n\t}\n\t{\n\t\tclass a {}\n\t\tn += new a() ? 1 : 0;\n\t}\n\tconst o = { a: __props.a, m(a: string) { return a; } };\n\t"
    );
}

#[test]
fn dom_hashbangs_and_directive_prologues_stay_before_the_runtime_import() {
    let source = "#!/usr/bin/env node\n'use client'; // keep\n\"use strict\";\nexport function A() @{\n\t<p />\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "#!/usr/bin/env node\n'use client'; // keep\n\"use strict\";\nimport { defineComponent as _defineComponent, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\";\nexport const A = /* @__PURE__ */ _defineComponent(function A() {\n\treturn (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"p\"))\n};\n});\n"
    );
    let source = "#!/usr/bin/env node";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "#!/usr/bin/env node\nimport { defineComponent as _defineComponent } from \"vue\";\n"
    );
    // A statement on the directive's line starts after the import.
    let source = "'use client'; export function A() @{\n\t<p />\n}\n";
    assert_eq!(
        compile(source, &OPTIONS).unwrap().code.as_str(),
        "'use client';\nimport { defineComponent as _defineComponent, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\";\n export const A = /* @__PURE__ */ _defineComponent(function A() {\n\treturn (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"p\"))\n};\n});\n"
    );
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

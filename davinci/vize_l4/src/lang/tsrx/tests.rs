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

    // `@for` lowers onto a host loop, which the DOM emitter refuses.
    let source = "export function List({ items }: { items: string[] }) @{\n\t@for (const item of items) {\n\t\t<li>{item}</li>\n\t}\n}\n";
    let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
    assert_eq!(diagnostic.code, Code::Emit(EmitIssue::Dom));
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("@{\n\t@for (const item of items) {\n\t\t<li>{item}</li>\n\t}\n}")
    );
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
    assert_eq!(
        refused(&options),
        [
            (Code::Emit(EmitIssue::SourceMap), whole),
            (Code::Emit(EmitIssue::TypeErasure), whole),
        ]
    );
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
            "export const App = () => <p class=\"x\">hi</p>;\n",
            "<p class=\"x\">hi</p>",
        ),
        ("export function view() @{\n\t<p />\n}\n", "@{\n\t<p />\n}"),
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

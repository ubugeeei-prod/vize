//! Scope retention and explicit unsupported module diagnostics.

use super::compile;
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx};
use vize_l0::Allocator;

#[test]
fn lexical_component_bindings_respect_shadowing_and_leave_unbound_globals() {
    let module = compile(
        "import Child from './Child'; export const App = (Child: any) => <Child/>; export const Global = () => <Unknown/>;",
    );
    let allocator = Allocator::new();
    let parsed = vize_atelier_jsx::parse_module(allocator.as_oxc(), &module, JsxLang::Tsx);
    let mut calls = ComponentCalls {
        source: &module,
        calls: Vec::new(),
    };
    calls.visit_program(&parsed.program);
    assert_eq!(
        calls.calls,
        [
            ("_resolveDynamicComponent", "Child"),
            ("_resolveComponent", "\"Unknown\""),
        ]
    );
}

#[test]
fn authored_runtime_helper_bindings_are_diagnosed_without_emitting_broken_modules() {
    for source in [
        "const _openBlock = 1; export const App = () => <p/>;",
        "export default (_openBlock: number) => <p/>;",
    ] {
        let arena = Allocator::new();
        let out = compile_jsx(&arena, source, JsxLang::Tsx, &JsxCompileConfig::default());
        assert!(out.has_errors(), "{source}");
        assert_eq!(out.diagnostics.len(), 1);
        let diagnostic = out.diagnostics.first().expect("helper collision");
        assert_eq!(
            diagnostic.message.as_str(),
            "JSX authored binding `_openBlock` shadows a generated runtime helper; rename the binding"
        );
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some("_openBlock")
        );
        assert!(out.module_code().is_empty());
    }
    let module = compile(
        "// _openBlock\nconst metadata = { _openBlock: 'safe' }; export default () => <p>{metadata._openBlock}</p>;",
    );
    let allocator = Allocator::new();
    let parsed = vize_atelier_jsx::parse_module(allocator.as_oxc(), &module, JsxLang::Tsx);
    let declaration = parsed.program.body.iter().find_map(|statement| {
        let oxc_ast::ast::Statement::VariableDeclaration(declaration) = statement else {
            return None;
        };
        declaration.declarations.iter().find_map(|item| {
            if item
                .id
                .get_identifier_name()
                .is_some_and(|id| id.as_str() == "metadata")
            {
                let span = item.init.as_ref()?.span();
                module.get(span.start as usize..span.end as usize)
            } else {
                None
            }
        })
    });
    assert_eq!(declaration, Some("{ _openBlock: 'safe' }"));
}

#[test]
fn standalone_vapor_and_ssr_reject_authored_bindings_and_exports_they_would_drop() {
    for source in [
        "import { ref } from 'vue'; const App = () => <p/>;",
        "export default () => <p/>;",
        "const App = (props: {label:string}) => <p>{props.label}</p>;",
        "const App = () => { const label = 'value'; return <p>{label}</p>; };",
    ] {
        for ssr in [false, true] {
            let arena = Allocator::new();
            let config = JsxCompileConfig {
                ssr,
                default_mode: vize_atelier_jsx::JsxOutputMode::Vapor,
                ..Default::default()
            };
            let out = compile_jsx(&arena, source, JsxLang::Tsx, &config);
            assert!(out.has_errors(), "{source} (ssr={ssr})");
            assert_eq!(
                out.diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>(),
                [
                    "Vapor/SSR authored module preservation is not supported for imports, exports or captured setup bindings; use VDOM output or consume the per-component renderer"
                ]
            );
            assert!(out.module_code().is_empty());
            assert!(
                !out.components[0].code().is_empty(),
                "per-component backend remains available"
            );
        }
    }
    for ssr in [false, true] {
        let arena = Allocator::new();
        let out = compile_jsx(
            &arena,
            "const App = () => <p>static</p>;",
            JsxLang::Tsx,
            &JsxCompileConfig {
                ssr,
                default_mode: vize_atelier_jsx::JsxOutputMode::Vapor,
                ..Default::default()
            },
        );
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        assert!(!out.module_code().is_empty());
    }
}

#[test]
fn renderer_parameters_do_not_silently_shadow_authored_context_references() {
    for source in [
        "export const App = (props: any, _ctx: any) => <p>{_ctx.attrs.title}</p>;",
        "export default () => <p>{_ctx.attrs.title}</p>;",
    ] {
        let arena = Allocator::new();
        let out = compile_jsx(&arena, source, JsxLang::Tsx, &JsxCompileConfig::default());
        assert!(out.has_errors());
        assert_eq!(out.diagnostics.len(), 1);
        let diagnostic = out.diagnostics.first().expect("renderer capture");
        assert_eq!(
            diagnostic.message.as_str(),
            "JSX authored reference `_ctx` is shadowed by a generated renderer binding; rename the reference"
        );
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some("_ctx")
        );
        assert!(out.module_code().is_empty());
    }
}

struct ComponentCalls<'a> {
    source: &'a str,
    calls: Vec<(&'a str, &'a str)>,
}

impl<'a> Visit<'a> for ComponentCalls<'_> {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        if let oxc_ast::ast::Expression::Identifier(identifier) = &call.callee {
            if matches!(
                identifier.name.as_str(),
                "_resolveDynamicComponent" | "_resolveComponent"
            ) {
                if let Some(argument) = call.arguments.first() {
                    let callee = identifier.span;
                    let argument = argument.span();
                    self.calls.push((
                        self.source
                            .get(callee.start as usize..callee.end as usize)
                            .expect("callee"),
                        self.source
                            .get(argument.start as usize..argument.end as usize)
                            .expect("argument"),
                    ));
                }
            }
        }
        oxc_ast_visit::walk::walk_call_expression(self, call);
    }
}

#[test]
fn block_setup_reports_lexical_contexts_it_cannot_retain() {
    for source in [
        "export const App = () => { return <p>{this.label}</p>; };",
        "function outer() { const App = () => { return <p>{arguments[0]}</p>; }; return App; }",
        "function Outer() { const App = () => { return <p>{new.target}</p>; }; return App; }",
        "export const App = () => { function factory() { const Inner = () => { return <p>{this.label}</p>; }; return Inner; } return <p/>; };",
        "export const App = () => { class Model { [this.label] = 1; } return <p/>; };",
    ] {
        let allocator = Allocator::new();
        let out = compile_jsx(
            &allocator,
            source,
            JsxLang::Tsx,
            &JsxCompileConfig::default(),
        );
        assert_eq!(
            out.diagnostics
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>(),
            [
                "Block-body JSX component setup cannot preserve lexical this, arguments or new.target; use a function declaration or expression-bodied component"
            ]
        );
        assert_eq!(out.module_code(), "");
    }
}

#![expect(clippy::disallowed_macros, reason = "test fixtures use format!")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use super::*;

fn parse_error_count(source: &str, source_type: oxc_span::SourceType) -> usize {
    oxc_parser::Parser::new(&oxc_allocator::Allocator::default(), source, source_type)
        .parse()
        .diagnostics
        .len()
}

// Source/provenance comments may contain authored JSX. Only executable OXC
// nodes decide whether lowering retained a JSX node or the component check.
fn jsx_and_tab_bar_calls(source: &str) -> (usize, usize) {
    use oxc_ast::ast::{Argument, CallExpression, Expression, JSXElement};
    use oxc_ast_visit::{Visit, walk};
    #[derive(Default)]
    struct Calls {
        jsx: usize,
        tab_bar: usize,
    }
    impl<'a> Visit<'a> for Calls {
        fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
            self.jsx += 1;
            walk::walk_jsx_element(self, element);
        }
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "__vize_jsx_component__")
                && matches!(call.arguments.first(), Some(Argument::Identifier(identifier)) if identifier.name == "TabBar")
            {
                self.tab_bar += 1;
            }
            walk::walk_call_expression(self, call);
        }
    }
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::tsx()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let mut calls = Calls::default();
    calls.visit_program(&parsed.program);
    (calls.jsx, calls.tab_bar)
}

#[test]
fn original_element_plus_spread_array_keeps_plain_ts_component_checks() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/jsx-spread-nested-roots");
    let source = std::fs::read_to_string(corpus.join("tab-nav.tsx.txt")).unwrap();
    let before = std::fs::read_to_string(corpus.join("tab-nav.original-h.ts.txt")).unwrap();
    assert_eq!(parse_error_count(&source, oxc_span::SourceType::tsx()), 0);
    assert!(parse_error_count(&before, oxc_span::SourceType::ts()) > 0);
    assert!(before.contains("<TabBar"));
    let after = generate_jsx_virtual_ts(Path::new("tab-nav.tsx"), &source, JsxLang::Tsx).unwrap();
    assert_eq!(
        parse_error_count(&after.code, oxc_span::SourceType::ts()),
        0
    );
    assert_eq!(jsx_and_tab_bar_calls(&after.code), (0, 1));
    for token in ["tabBarRef", "[...props.panes]", "tabRefsMap.value"] {
        assert!(
            after.mappings.iter().any(|mapping| {
                source.get(mapping.src_range.clone()) == Some(token)
                    && after.code.get(mapping.gen_range.clone()) == Some(token)
            }),
            "original component argument {token} must remain mapped"
        );
    }
}

#[test]
fn spread_nested_callbacks_keep_component_props_and_original_scope() {
    let source = "const Child = (props: { count: number }) => <span>{props.count}</span>; const view = <div>{...[1].map(item => item ? <Child count={item.toFixed(1)} /> : null)}</div>;";
    let generated = generate_jsx_virtual_ts(Path::new("Spread.tsx"), source, JsxLang::Tsx).unwrap();
    assert_eq!(
        parse_error_count(&generated.code, oxc_span::SourceType::ts()),
        0
    );
    assert!(generated.code.contains("[1].map(item => item ? __vize_jsx"));
    assert!(generated.code.contains("__vize_jsx_component__(Child"));
    let token = "item.toFixed(1)";
    assert!(generated.mappings.iter().any(|mapping| {
        source.get(mapping.src_range.clone()) == Some(token)
            && generated.code.get(mapping.gen_range.clone()) == Some(token)
    }));
}

#[test]
fn nested_native_roots_preserve_callback_scope_and_exact_token_maps() {
    let source = "// 日本語 😀\r\nconst view = <Host>{{ content: () => <div>{Array.from({ length: 3 }, (_, i) => <p>{i.toFixed(1)}{(() => <span>{i + 1}</span>)()}</p>)}{(() => <aside>{42}</aside>)()}{String(\"<p>literal</p>\")}</div> }}</Host>;";
    let generated = generate_jsx_virtual_ts(Path::new("Nested.tsx"), source, JsxLang::Tsx).unwrap();
    assert!(generated.diagnostics.is_empty());
    assert!(generated.code.contains("Array.from({ length: 3 }, (_, i) => __vize_jsx_expr__(i.toFixed(1), (() => __vize_jsx_expr__(i + 1))())"), "{}", generated.code);
    assert!(generated.code.contains("(() => __vize_jsx_expr__(42))()"));
    assert!(generated.code.contains("\"<p>literal</p>\""));
    for token in ["i.toFixed(1)", "i + 1", "42"] {
        let original = source.find(token).unwrap();
        let mapped = generated
            .mappings
            .iter()
            .find(|mapping| mapping.src_range == (original..original + token.len()))
            .unwrap();
        assert_eq!(&generated.code[mapped.gen_range.clone()], token);
        let map = crate::batch::source_map::SfcSourceMap::new(
            generated.mappings.clone(),
            vec![crate::batch::source_map::SfcBlockRange {
                start: 0,
                end: source.len() as u32,
                block_type: SfcBlockType::Script,
            }],
        );
        assert_eq!(
            map.get_original_position(mapped.gen_range.start as u32),
            Some((original as u32, 0, SfcBlockType::Script))
        );
        let reverse = map
            .get_virtual_offset(original as u32, SfcBlockType::Script)
            .unwrap() as usize;
        assert_eq!(&generated.code[reverse..reverse + token.len()], token);
    }
}

#[test]
fn adjacent_ordinary_roots_keep_each_authored_expression() {
    let source = (0..512)
        .map(|index| format!("const view{index} = <div>{{value{index}}}</div>;\n"))
        .collect::<std::string::String>();
    let generated =
        generate_jsx_virtual_ts(Path::new("Siblings.tsx"), &source, JsxLang::Tsx).unwrap();
    for index in 0..512 {
        assert!(generated.code.contains(&format!(
            "const view{index} = __vize_jsx_expr__(value{index});"
        )));
    }
    assert_eq!(
        generated.code.matches("__vize_jsx_expr__(value").count(),
        512
    );
}

#[test]
fn conditional_raw_callback_arm_retains_native_jsx_roots() {
    let source = "const view = <Host>{{ content: () => <div>{ok ? <p/> : Array.from({ length: 3 }, (_, i) => <p>{i}</p>)}</div> }}</Host>;";
    let generated =
        generate_jsx_virtual_ts(Path::new("Conditional.tsx"), source, JsxLang::Tsx).unwrap();
    assert!(generated.diagnostics.is_empty());
    assert!(
        generated
            .code
            .contains("Array.from({ length: 3 }, (_, i) => __vize_jsx_expr__(i))"),
        "{}",
        generated.code
    );
    assert!(!generated.code.contains("<p>"));
}

#[test]
fn nested_scoped_style_interpolations_stay_in_their_native_callback_scope() {
    let source = "// 日本語 😀\r\nconst view = <Host>{{ content: () => <div><style scoped>{`div { color: ${theme}; }`}</style>{Array.from({ length: 3 }, (_, i) => <p><style scoped>{`p { color: ${i.toFixed()}; }`}</style>{i}</p>)}</div> }}</Host>;";
    let generated = generate_jsx_virtual_ts(Path::new("Style.tsx"), source, JsxLang::Tsx).unwrap();
    assert!(generated.diagnostics.is_empty());
    assert!(
        generated
            .code
            .contains("Array.from({ length: 3 }, (_, i) => __vize_jsx_expr__(i, i.toFixed()))"),
        "{}",
        generated.code
    );
    assert_eq!(generated.code.matches("i.toFixed()").count(), 1);
    assert_eq!(generated.code.matches("theme").count(), 1);
    let original = source.find("i.toFixed()").unwrap();
    let mapping = generated
        .mappings
        .iter()
        .find(|mapping| mapping.src_range == (original..original + "i.toFixed()".len()))
        .unwrap();
    assert_eq!(&generated.code[mapping.gen_range.clone()], "i.toFixed()");
}

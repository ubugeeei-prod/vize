use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use super::convert_render;

/// Wrap a JSX expression in `const x = (<JSX>);` and convert it to template.
fn convert(jsx: &str) -> Option<String> {
    let mut source = std::string::String::from("const x = (");
    source.push_str(jsx);
    source.push_str(");\n");

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::tsx()).parse();
    assert!(!parsed.panicked, "fixture should parse: {source}");

    let Some(Statement::VariableDeclaration(decl)) = parsed.program.body.first() else {
        panic!("expected variable declaration");
    };
    let init = decl.declarations[0]
        .init
        .as_ref()
        .expect("declarator initializer");
    let expr = match init {
        Expression::ParenthesizedExpression(inner) => &inner.expression,
        other => other,
    };
    convert_render(expr, &source)
}

#[test]
fn converts_element_with_string_attr_and_text() {
    assert_eq!(
        convert(r#"<AfButton color="primary">Primary</AfButton>"#).as_deref(),
        Some(r#"<AfButton color="primary">Primary</AfButton>"#)
    );
}

#[test]
fn converts_self_closing_element() {
    assert_eq!(convert("<AfButton />").as_deref(), Some("<AfButton />"));
}

#[test]
fn converts_expression_attr_and_bare_attr() {
    assert_eq!(
        convert("<AfButton count={1 + 2} disabled />").as_deref(),
        Some(r#"<AfButton :count="1 + 2" disabled />"#)
    );
}

#[test]
fn converts_spread_attr_to_v_bind() {
    assert_eq!(
        convert("<AfButton {...props} />").as_deref(),
        Some(r#"<AfButton v-bind="props" />"#)
    );
}

#[test]
fn converts_expression_child_to_mustache() {
    assert_eq!(
        convert("<AfButton>{label}</AfButton>").as_deref(),
        Some("<AfButton>{{ label }}</AfButton>")
    );
}

#[test]
fn fragment_emits_children_without_wrapper() {
    assert_eq!(
        convert("<><AfButton a=\"x\" /><AfButton b=\"y\" /></>").as_deref(),
        Some(r#"<AfButton a="x" /><AfButton b="y" />"#)
    );
}

#[test]
fn converts_dotted_member_element_name() {
    assert_eq!(
        convert("<Form.Item label=\"name\" />").as_deref(),
        Some(r#"<Form.Item label="name" />"#)
    );
}

#[test]
fn escapes_quotes_in_expression_attr() {
    // An expression attribute whose source contains `"` must be HTML-escaped so
    // it cannot break out of the double-quoted Vue binding.
    assert_eq!(
        convert(r#"<AfButton onClick={() => alert("hi")} />"#).as_deref(),
        Some(r#"<AfButton :onClick="() => alert(&quot;hi&quot;)" />"#)
    );
}

#[test]
fn non_jsx_render_returns_none() {
    assert_eq!(convert("42").as_deref(), None);
}

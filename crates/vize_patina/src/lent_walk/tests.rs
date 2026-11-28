//! The lent walks hand out nodes that stay readable after the walk returns.
//! The Miri lane runs these tests, so every lent reference is also checked
//! under the aliasing model.

mod differential;

use super::{LentNodes, walk_expression, walk_jsx_expression_container, walk_program};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, Expression, JSXChild, JSXElement, JSXElementName, JSXFragment, Program,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

const SOURCE: &str = "const view = <main>{items.map((item) => <li key={item}>{format(item)}</li>)}<><b /></></main>;\nregister(defineComponent({}));\n";

fn tag<'a>(element: &'a JSXElement<'a>) -> &'a str {
    match &element.opening_element.name {
        JSXElementName::Identifier(identifier) => identifier.name.as_str(),
        _ => "?",
    }
}

/// Keeps every lent node past the walk, plus the order of the hooks.
#[derive(Default)]
struct Kept<'a> {
    elements: Vec<&'a JSXElement<'a>>,
    fragments: Vec<&'a JSXFragment<'a>>,
    calls: Vec<&'a CallExpression<'a>>,
    events: Vec<&'a str>,
    descend: bool,
}

impl<'a> LentNodes<'a> for Kept<'a> {
    fn enter_jsx_element(&mut self, element: &'a JSXElement<'a>) -> bool {
        self.elements.push(element);
        self.events.push(tag(element));
        self.descend
    }

    fn leave_jsx_element(&mut self, element: &'a JSXElement<'a>) {
        self.events.push(tag(element));
    }

    fn enter_jsx_fragment(&mut self, fragment: &'a JSXFragment<'a>) -> bool {
        self.fragments.push(fragment);
        self.events.push("<>");
        self.descend
    }

    fn leave_jsx_fragment(&mut self, _fragment: &'a JSXFragment<'a>) {
        self.events.push("</>");
    }

    fn call_expression(&mut self, call: &'a CallExpression<'a>) {
        self.calls.push(call);
        self.events.push(call.callee_name().unwrap_or("?"));
    }
}

fn parse<'a>(allocator: &'a Allocator) -> Program<'a> {
    let parsed = Parser::new(allocator, SOURCE, SourceType::jsx()).parse();
    assert!(parsed.diagnostics.is_empty(), "the fixture parses");
    parsed.program
}

fn view<'a>(program: &'a Program<'a>) -> &'a Expression<'a> {
    let Some(Statement::VariableDeclaration(declaration)) = program.body.first() else {
        unreachable!("the fixture starts with `const view = ...`");
    };
    let Some(init) = declaration
        .declarations
        .first()
        .and_then(|d| d.init.as_ref())
    else {
        unreachable!("`view` has an initializer");
    };
    init
}

#[test]
fn a_whole_walk_lends_every_node_in_source_order() {
    let allocator = Allocator::default();
    let program = parse(&allocator);
    let mut kept = Kept {
        descend: true,
        ..Kept::default()
    };
    walk_program(&mut kept, &program);

    assert_eq!(
        kept.events,
        [
            "main",
            "map",
            "li",
            "format",
            "li",
            "<>",
            "b",
            "b",
            "</>",
            "main",
            "register",
            "defineComponent"
        ]
    );
    // The kept references outlive the walk and still read the program.
    let tags: Vec<&str> = kept.elements.iter().map(|element| tag(element)).collect();
    assert_eq!(tags, ["main", "li", "b"]);
    assert_eq!(kept.fragments.len(), 1);
    assert_eq!(
        kept.fragments
            .first()
            .map(|fragment| fragment.children.len()),
        Some(1)
    );
    let callees: Vec<Option<&str>> = kept.calls.iter().map(|call| call.callee_name()).collect();
    assert_eq!(
        callees,
        [
            Some("map"),
            Some("format"),
            Some("register"),
            Some("defineComponent")
        ]
    );
    let starts: Vec<u32> = kept
        .elements
        .iter()
        .map(|element| element.span.start)
        .collect();
    assert!(starts.is_sorted());
}

#[test]
fn an_element_that_refuses_descent_hides_its_contents() {
    let allocator = Allocator::default();
    let program = parse(&allocator);
    let mut kept = Kept::default();
    walk_program(&mut kept, &program);

    // `main` is lent, its contents are not, and no leave hook runs.
    assert_eq!(kept.events, ["main", "register", "defineComponent"]);
    assert_eq!(
        kept.elements.first().map(|element| tag(element)),
        Some("main")
    );
}

#[test]
fn an_expression_walk_lends_nodes_below_that_expression() {
    let allocator = Allocator::default();
    let program = parse(&allocator);
    let mut kept = Kept {
        descend: true,
        ..Kept::default()
    };
    walk_expression(&mut kept, view(&program));

    assert_eq!(
        kept.events,
        [
            "main", "map", "li", "format", "li", "<>", "b", "b", "</>", "main"
        ]
    );
    assert!(
        kept.calls
            .iter()
            .all(|call| call.callee_name() != Some("register"))
    );
}

#[test]
fn a_container_walk_lends_nodes_inside_the_container() {
    let allocator = Allocator::default();
    let program = parse(&allocator);
    let Expression::JSXElement(main) = view(&program) else {
        unreachable!("`view` is a JSX element");
    };
    let Some(JSXChild::ExpressionContainer(container)) = main.children.first() else {
        unreachable!("`main` starts with an expression container");
    };
    let mut kept = Kept::default();
    walk_jsx_expression_container(&mut kept, container);

    assert_eq!(kept.events, ["map", "li"]);
    assert_eq!(
        kept.elements
            .first()
            .map(|element| element.opening_element.attributes.len()),
        Some(1)
    );
}

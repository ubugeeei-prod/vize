use alloc::{format, string::String, vec::Vec};

use oxc_ast::ast::{Expression, ForStatementLeft, Statement};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use super::{
    Attribute, AttributeValue, Child, CommentKind, Node, Region, RegionSite, TagName, TsrxModule,
    parse,
};

fn text(source: &str, span: Span) -> &str {
    source.get(span.start as usize..span.end as usize).unwrap()
}

fn host(source: &str, span: oxc_span::Span) -> &str {
    source.get(span.start as usize..span.end as usize).unwrap()
}

fn clean<'a>(allocator: &'a Allocator, source: &'a str) -> TsrxModule<'a> {
    let module = parse(allocator, source);
    assert!(module.is_clean(), "{source}\n{:?}", module.diagnostics());
    module
}

/// The first error as `CODE start..end`.
fn first_error(source: &str) -> String {
    let allocator = Allocator::new();
    let module = parse(&allocator, source);
    assert!(!module.is_clean(), "expected an error in {source}");
    let diagnostic = module.diagnostics().first().unwrap();
    let code = match (&diagnostic.code.scope, &diagnostic.code.number) {
        (Some(scope), Some(number)) => format!("{scope}{number}"),
        _ => String::from("-"),
    };
    let label = diagnostic.labels.iter().next().unwrap();
    format!(
        "{code} {}..{}",
        label.offset(),
        label.offset() + label.len()
    )
}

fn only_region<'m, 'a>(module: &'m TsrxModule<'a>) -> &'m Region<'a> {
    assert_eq!(module.regions().len(), 1);
    module.regions().first().unwrap()
}

#[test]
fn element_spans_attributes_and_children() {
    let allocator = Allocator::new();
    let source = "const a = <div class=\"x\" id={y} {z} {...rest} disabled>hi {y}</div>;";
    let module = clean(&allocator, source);
    let region = only_region(&module);
    assert_eq!(region.site, RegionSite::Expression);
    assert_eq!(
        text(source, region.span),
        "<div class=\"x\" id={y} {z} {...rest} disabled>hi {y}</div>"
    );
    let Node::Element(element) = &region.node else {
        panic!()
    };
    let TagName::Static(name) = element.name else {
        panic!()
    };
    assert_eq!(text(source, name), "div");
    let attributes: Vec<_> = element
        .attributes
        .iter()
        .map(|attribute| match attribute {
            Attribute::Named { span, name, value } => {
                let value = match value {
                    Some(AttributeValue::String(span)) => format!("={}", text(source, *span)),
                    Some(AttributeValue::Expression { span, expression }) => {
                        assert_eq!(host(source, expression.span()), "y");
                        format!("={}", text(source, *span))
                    }
                    None => String::new(),
                };
                assert_eq!(
                    text(source, *span),
                    format!("{}{value}", text(source, *name))
                );
                format!("named {}{value}", text(source, *name))
            }
            Attribute::Shorthand { span, expression } => {
                format!(
                    "shorthand {} {}",
                    text(source, *span),
                    host(source, expression.span())
                )
            }
            Attribute::Spread { span, expression } => {
                format!(
                    "spread {} {}",
                    text(source, *span),
                    host(source, expression.span())
                )
            }
        })
        .collect();
    assert_eq!(
        attributes,
        [
            "named class=\"x\"",
            "named id={y}",
            "shorthand {z} z",
            "spread {...rest} rest",
            "named disabled"
        ]
    );
    assert_eq!(element.children.len(), 2);
    let Some(Child::Text(span)) = element.children.first() else {
        panic!()
    };
    assert_eq!(text(source, *span), "hi ");
    let Some(Child::Expression(child)) = element.children.get(1) else {
        panic!()
    };
    assert_eq!(text(source, child.span), "{y}");
    assert_eq!(text(source, element.closing.unwrap()), "</div>");
    // The host keeps an empty fragment with the region's exact span.
    let Some(Statement::VariableDeclaration(declaration)) = module.program().body.first() else {
        panic!()
    };
    let init = declaration
        .declarations
        .first()
        .unwrap()
        .init
        .as_ref()
        .unwrap();
    let Expression::JSXFragment(placeholder) = init else {
        panic!()
    };
    assert!(core::ptr::eq(
        module.expression_region(init).unwrap(),
        region
    ));
    assert!(placeholder.children.is_empty());
    assert_eq!(
        Span::new(placeholder.span.start, placeholder.span.end),
        region.span
    );
}

#[test]
fn statement_container_function_bodies_and_the_line_start_rule() {
    let allocator = Allocator::new();
    let source = "export function A({ a, b }: P) @{\n\tconst ok = a < b\n\tconst m = 'x'\n\t<p>{m}</p>\n}\nconst G = () => @{ <i /> };";
    let module = clean(&allocator, source);
    assert_eq!(module.regions().len(), 2);
    let first = module.regions().first().unwrap();
    assert_eq!(first.site, RegionSite::FunctionBody);
    assert_eq!(
        text(source, first.span),
        "@{\n\tconst ok = a < b\n\tconst m = 'x'\n\t<p>{m}</p>\n}"
    );
    let Node::CodeBlock(block) = &first.node else {
        panic!()
    };
    let statements: Vec<_> = block
        .statements
        .iter()
        .map(|statement| host(source, statement.span()))
        .collect();
    assert_eq!(statements, ["const ok = a < b", "const m = 'x'"]);
    assert_eq!(
        text(source, block.render.as_ref().unwrap().span()),
        "<p>{m}</p>"
    );
    let second = module.regions().get(1).unwrap();
    assert_eq!(second.site, RegionSite::FunctionBody);
    assert_eq!(text(source, second.span), "@{ <i /> }");
    // The host function and arrow bodies are placeholders with the region spans.
    let Some(Statement::ExportNamedDeclaration(export)) = module.program().body.first() else {
        panic!()
    };
    let Some(oxc_ast::ast::Declaration::FunctionDeclaration(function)) = &export.declaration else {
        panic!()
    };
    let body = function.body.as_ref().unwrap();
    assert!(body.statements.is_empty());
    assert!(core::ptr::eq(module.body_region(body).unwrap(), first));
    assert_eq!(Span::new(body.span.start, body.span.end), first.span);
}

#[test]
fn directives_record_branches_heads_and_bodies() {
    let allocator = Allocator::new();
    let source = "const v = <ul>\n@if (a) { <b /> } @else if (c) { <i /> } @else { <u /> }\n@for (const item of items; index i; key item.id) { const t = item.t; <li>{t}</li> } @empty { <li /> }\n@switch (mode) { @case 'a': { <b /> } @default: { <i /> } }\n@try { <C /> } @pending { <p /> } @catch (e, reset) { <p /> }\n</ul>;";
    let module = clean(&allocator, source);
    let Node::Element(list) = &only_region(&module).node else {
        panic!()
    };
    let nodes: Vec<_> = list
        .children
        .iter()
        .filter_map(|child| match child {
            Child::Node(node) => Some(node),
            _ => None,
        })
        .collect();
    assert_eq!(nodes.len(), 4);
    let Node::If(branch) = nodes.first().unwrap() else {
        panic!()
    };
    let branches: Vec<_> = branch
        .branches
        .iter()
        .map(|branch| text(source, branch.span))
        .collect();
    assert_eq!(
        branches,
        [
            "@if (a) { <b /> }",
            "@else if (c) { <i /> }",
            "@else { <u /> }"
        ]
    );
    assert_eq!(
        host(
            source,
            branch
                .branches
                .get(1)
                .unwrap()
                .test
                .as_ref()
                .unwrap()
                .span()
        ),
        "c"
    );
    assert!(branch.branches.get(2).unwrap().test.is_none());

    let Node::For(looped) = nodes.get(1).unwrap() else {
        panic!()
    };
    let ForStatementLeft::VariableDeclaration(left) = &looped.left else {
        panic!()
    };
    assert_eq!(host(source, left.span), "const item");
    assert_eq!(host(source, looped.right.span()), "items");
    assert_eq!(looped.index.as_ref().unwrap().name.as_str(), "i");
    assert_eq!(host(source, looped.key.as_ref().unwrap().span()), "item.id");
    assert_eq!(looped.body.statements.len(), 1);
    assert_eq!(
        text(source, looped.empty.as_ref().unwrap().span),
        "{ <li /> }"
    );

    let Node::Switch(switch) = nodes.get(2).unwrap() else {
        panic!()
    };
    assert_eq!(host(source, switch.discriminant.span()), "mode");
    let cases: Vec<_> = switch
        .cases
        .iter()
        .map(|case| text(source, case.span))
        .collect();
    assert_eq!(cases, ["@case 'a': { <b /> }", "@default: { <i /> }"]);

    let Node::Try(attempt) = nodes.get(3).unwrap() else {
        panic!()
    };
    assert_eq!(
        text(source, attempt.pending.as_ref().unwrap().span),
        "{ <p /> }"
    );
    let catch = attempt.catch.as_ref().unwrap();
    assert_eq!(text(source, catch.span), "@catch (e, reset) { <p /> }");
    assert_eq!(catch.params.as_ref().unwrap().items.len(), 2);
}

#[test]
fn comments_between_children_and_comment_like_text() {
    let allocator = Allocator::new();
    let source = "const v = <p>\n/* c */ a//b https://x.y\n// line\r\n<b />me@example.com</p>;";
    let module = clean(&allocator, source);
    let Node::Element(paragraph) = &only_region(&module).node else {
        panic!()
    };
    let children: Vec<_> = paragraph
        .children
        .iter()
        .map(|child| match child {
            Child::Text(span) => format!("text {:?}", text(source, *span)),
            Child::Comment(comment) => {
                let kind = if comment.kind == CommentKind::Block {
                    "block"
                } else {
                    "line"
                };
                format!("{kind} {:?}", text(source, comment.span))
            }
            Child::Expression(child) => format!("expression {:?}", text(source, child.span)),
            Child::Node(node) => format!("node {:?}", text(source, node.span())),
        })
        .collect();
    assert_eq!(
        children,
        [
            "text \"\\n\"",
            "block \"/* c */\"",
            "text \" a//b https://x.y\\n\"",
            "line \"// line\"",
            "text \"\\r\\n\"",
            "node \"<b />\"",
            "text \"me@example.com\"",
        ]
    );
}

#[test]
fn nested_regions_in_embedded_expressions_are_found_by_span() {
    let allocator = Allocator::new();
    let source = "const v = <ul>{items.map((i) => <li>{i}</li>)}</ul>;";
    let module = clean(&allocator, source);
    assert_eq!(module.regions().len(), 2);
    let inner = module.regions().get(1).unwrap();
    assert_eq!(text(source, inner.span), "<li>{i}</li>");
    assert!(module.region(inner.span).is_some());
    assert!(
        module
            .region(Span::new(inner.span.start, inner.span.end - 1))
            .is_none()
    );
}

#[test]
fn dynamic_tags_raw_text_and_decorators() {
    let allocator = Allocator::new();
    let source = "@dec class K {}\nconst v = <{props.as} class=\"p\"><script>{code} if (a < b) {}</script><style>.a { color: red }</style></{props.as}>;";
    let module = clean(&allocator, source);
    let Node::Element(element) = &only_region(&module).node else {
        panic!()
    };
    let TagName::Dynamic { span, expression } = &element.name else {
        panic!()
    };
    assert_eq!(text(source, *span), "{props.as}");
    assert_eq!(host(source, expression.span()), "props.as");
    let raw: Vec<_> = element
        .children
        .iter()
        .map(|child| {
            let Child::Node(Node::Element(child)) = child else {
                panic!()
            };
            text(source, child.raw_text.unwrap())
        })
        .collect();
    assert_eq!(raw, ["{code} if (a < b) {}", ".a { color: red }"]);
}

#[test]
fn unicode_and_crlf_spans_are_byte_exact() {
    let allocator = Allocator::new();
    let source = "const v = <p title=\"雪\">\r\n\t🌸 {x}\r\n</p>;";
    let module = clean(&allocator, source);
    let region = only_region(&module);
    assert_eq!(
        text(source, region.span),
        "<p title=\"雪\">\r\n\t🌸 {x}\r\n</p>"
    );
    let Node::Element(element) = &region.node else {
        panic!()
    };
    let Some(Child::Text(span)) = element.children.first() else {
        panic!()
    };
    assert_eq!(text(source, *span), "\r\n\t🌸 ");
}

#[test]
fn errors_carry_reference_codes_and_spans() {
    let cases = [
        ("export function B() @{\n\t<div>\n}\n", "TSRX1001 30..31"),
        (
            "export function B() @{\n\t<div></span>\n}\n",
            "TSRX1002 29..36",
        ),
        (
            "export function B() @{\n\t<div />\n\tconst late = 1;\n}\n",
            "TSRX2012 33..48",
        ),
        (
            "export function B() @{\n\t<div />\n\t<p />\n}\n",
            "TSRX2011 33..38",
        ),
        (
            "export function B(r: boolean) @{\n\t<div>@if (r) { return null; }</div>\n}\n",
            "TSRX2002 49..61",
        ),
        (
            "export function B(m: number) @{\n\t@switch (m) { @case 1: { break; } }\n}\n",
            "TSRX2008 58..64",
        ),
        (
            "export function B(a: string, b: string) @{\n\t<{a || b} />\n}\n",
            "TSRX2014 46..52",
        ),
        ("const v = <script>a</SCRIPT>b</script>;", "TSRX1004 19..27"),
        ("const v = @try { <p /> };", "TSRX1010 10..24"),
        ("const v = <p>{...xs}</p>;", "TSRX2013 14..17"),
        ("const v = <p>{a;}</p>;", "TSRX1007 15..16"),
        (
            "const v = <ul>@for (let i = 0; i < 2; i++) { <li /> }</ul>;",
            "TSRX2015 19..20",
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(first_error(source), expected, "{source}");
    }
}

/// Every row of the pinned reference corpus (#6858): valid modules parse
/// clean, and each error row reports the reference's code at its byte span.
#[test]
fn reference_corpus_rows_agree() {
    extern crate std;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = std::fs::read_to_string(root.join("tests/_fixtures/tsrx/reference.json")).unwrap();
    let corpus: serde_json::Value = serde_json::from_str(&corpus).unwrap();
    let rows = corpus["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        let file = row["file"].as_str().unwrap();
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let allocator = Allocator::new();
        let module = parse(&allocator, &source);
        if row["error"].is_null() {
            assert!(module.is_clean(), "{file}: {:?}", module.diagnostics());
            continue;
        }
        let error = &row["error"];
        let start = error["start"].as_u64().unwrap();
        let expected = match error["end"].as_u64() {
            Some(end) => format!("{} {start}..{end}", error["code"].as_str().unwrap()),
            None => format!("{} {start}", error["code"].as_str().unwrap()),
        };
        let found = first_error(&source);
        let found = if error["end"].is_null() {
            found
                .split_once("..")
                .map_or(found.clone(), |(head, _)| String::from(head))
        } else {
            found
        };
        assert_eq!(found, expected, "{file}");
    }
}

#[test]
fn host_parser_options_reach_embedded_code() {
    let source = "const a = <p>{(x)}</p>;";
    let allocator = Allocator::new();
    let options = oxc_parser::ParseOptions {
        preserve_parens: false,
        ..oxc_parser::ParseOptions::default()
    };
    let module = super::parse_with_options(&allocator, source, options);
    assert!(module.is_clean(), "{:?}", module.diagnostics());
    let Node::Element(element) = &only_region(&module).node else {
        panic!("expected an element");
    };
    let Some(Child::Expression(child)) = element.children.first() else {
        panic!("expected an expression child");
    };
    assert!(matches!(child.expression, Some(Expression::Identifier(_))));
    assert_eq!(text(source, child.span), "{(x)}");
}

use vize_armature::{ExpressionNode, PropNode, TemplateChildNode, parse_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::markup::document::{
    DocumentPetiteDirectiveKind, DocumentPetiteValue, DocumentTokenKind, NativeDocument,
};

#[derive(Debug, PartialEq, Eq)]
struct Observed {
    kind: &'static str,
    element: String,
    frame: String,
    value: Option<String>,
    argument: Option<String>,
    modifiers: Vec<String>,
}

fn kind(kind: DocumentPetiteDirectiveKind) -> &'static str {
    match kind {
        DocumentPetiteDirectiveKind::Scope => "scope",
        DocumentPetiteDirectiveKind::Effect => "effect",
    }
}

fn native_value(value: DocumentPetiteValue<'_, '_>) -> String {
    let mut result = String::new();
    for token in value.tokens() {
        match token.kind() {
            DocumentTokenKind::AttributeData => {
                result.push_str(token.source().expect("original attribute data"));
            }
            DocumentTokenKind::AttributeEntity => token
                .decoded_entity()
                .expect("retained entity")
                .for_each(|ch| result.push(ch)),
            other => panic!("unexpected value token {other:?}"),
        }
    }
    result
}

fn native(source: &str) -> Vec<Observed> {
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    owner
        .petite_directives()
        .expect("complete native document")
        .map(|directive| {
            let directive = directive.expect("original directive frame");
            Observed {
                kind: kind(directive.kind()),
                element: directive.element_name().into(),
                frame: directive.authored().into(),
                value: directive.value().map(native_value),
                argument: directive
                    .argument()
                    .map(|token| token.source().unwrap().into()),
                modifiers: directive
                    .modifiers()
                    .map(|token| token.source().unwrap().into())
                    .collect(),
            }
        })
        .collect()
}

fn expression(expression: &Option<ExpressionNode<'_>>) -> Option<String> {
    expression.as_ref().map(|expression| match expression {
        ExpressionNode::Simple(value) => value.content.into(),
        ExpressionNode::Compound(_) => panic!("directive field must stay simple"),
    })
}

fn walk_legacy(source: &str, children: &[TemplateChildNode<'_>], result: &mut Vec<Observed>) {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for prop in &element.props {
            let PropNode::Directive(directive) = prop else {
                continue;
            };
            if !matches!(directive.name, "scope" | "effect") {
                continue;
            }
            let Span { start, end } = directive.loc.span;
            result.push(Observed {
                kind: match directive.name {
                    "scope" => "scope",
                    "effect" => "effect",
                    _ => unreachable!(),
                },
                element: element.tag.into(),
                frame: source[start as usize..end as usize].into(),
                value: expression(&directive.exp),
                argument: expression(&directive.arg),
                modifiers: directive
                    .modifiers
                    .iter()
                    .map(|modifier| modifier.content.into())
                    .collect(),
            });
        }
        walk_legacy(source, &element.children, result);
    }
}

fn legacy(source: &str) -> Vec<Observed> {
    let arena = Allocator::default();
    let (root, errors) = parse_document(&arena, source);
    assert!(errors.is_empty(), "legacy document errors: {errors:?}");
    let mut result = Vec::new();
    walk_legacy(source, &root.children, &mut result);
    result
}

#[test]
fn native_petite_directives_match_the_shipped_document_parser() {
    let packets = [
        concat!(
            "<!DOCTYPE html><html><head></head><body>",
            "<div v-scope=\"{ count: 0 }\" v-effect=\"count++\"></div>",
            "</body></html>",
        ),
        concat!(
            "<!DOCTYPE html><html><head></head><body>",
            "<main v-scope><button v-effect=''></button>",
            "<span V-SCOPE=ignored></span></main></body></html>",
        ),
        concat!(
            "<!DOCTYPE html><html><head></head><body>",
            "<div v-scope:state.deep=\"{ label: '&amp;' }\"></div>",
            "</body></html>",
        ),
        concat!(
            "<!DOCTYPE html><html><head></head><body><table><tbody><tr>",
            "<td v-scope=\"{ row: 1 }\" v-effect=\"row += 1\"></td>",
            "</tr></tbody></table></body></html>",
        ),
        concat!(
            "<!DOCTYPE html><html><head></head><body><svg><foreignObject>",
            "<section v-scope=\"{ x: 1 }\"><i v-effect=\"x++\"></i></section>",
            "</foreignObject></svg></body></html>",
        ),
        include_str!("fixtures/document/petite-vue-svg.html"),
    ];

    for source in packets {
        assert_eq!(native(source), legacy(source), "{source}");
    }
}

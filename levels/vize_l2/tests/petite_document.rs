use vize_armature::{ExpressionNode, PropNode, TemplateChildNode, parse_document};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::markup::document::NativeDocument;
use vize_l2::dialect::petite::{PetiteDirective, PetiteDirectiveKind, PetiteDocument};

fn expression<'a>(expression: &Option<ExpressionNode<'a>>) -> Option<&'a str> {
    expression.as_ref().map(|expression| match expression {
        ExpressionNode::Simple(value) => value.content,
        ExpressionNode::Compound(_) => panic!("directive field must stay simple"),
    })
}

fn assert_directive(
    source: &str,
    native: &PetiteDirective<'_>,
    legacy: &vize_armature::DirectiveNode<'_>,
) {
    assert_eq!(
        native.kind(),
        match legacy.name {
            "scope" => PetiteDirectiveKind::Scope,
            "effect" => PetiteDirectiveKind::Effect,
            _ => panic!("unexpected legacy directive"),
        }
    );
    assert_eq!(
        native.value().map(|value| value.decoded()),
        expression(&legacy.exp)
    );
    assert_eq!(native.argument(), expression(&legacy.arg));
    assert_eq!(native.span().slice(source), legacy.loc.span.slice(source));
    if let (Some(value), Some(expression)) = (native.value(), legacy.exp.as_ref()) {
        assert_eq!(value.authored_span(), expression.loc().span);
    }
    assert!(
        native
            .modifiers()
            .eq(legacy.modifiers.iter().map(|modifier| modifier.content))
    );
}

fn walk_legacy<'a>(
    source: &str,
    children: &[TemplateChildNode<'a>],
    native: &mut impl Iterator<Item = &'a PetiteDirective<'a>>,
) {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for prop in &element.props {
            let PropNode::Directive(directive) = prop else {
                continue;
            };
            if matches!(directive.name, "scope" | "effect") {
                let observed = native.next().expect("matching native directive");
                assert_eq!(observed.element_name(), element.tag);
                assert_directive(source, observed, directive);
            }
        }
        walk_legacy(source, &element.children, native);
    }
}

fn assert_matches(source: &str) {
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, SourceRoot::new(source).unwrap());
    let semantic = PetiteDocument::lower_in(&allocator, &owner).unwrap();
    let (legacy, errors) = parse_document(&allocator, source);
    assert!(errors.is_empty(), "legacy document errors: {errors:?}");
    let mut native = semantic.directives();
    walk_legacy(source, &legacy.children, &mut native);
    assert!(native.next().is_none(), "unmatched native directive");
}

#[test]
fn l2_petite_meaning_matches_the_shipped_document_parser() {
    for source in [
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
        include_str!("../../vize_l1/tests/fixtures/document/petite-vue-svg.html"),
    ] {
        assert_matches(source);
    }
}

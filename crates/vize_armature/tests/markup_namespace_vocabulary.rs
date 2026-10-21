use vize_l1::markup::Namespace as MarkupNamespace;
use vize_relief::Namespace as AstNamespace;

fn through_markup(namespace: MarkupNamespace) -> AstNamespace {
    namespace
}

fn through_ast(namespace: AstNamespace) -> MarkupNamespace {
    namespace
}

#[test]
fn markup_and_ast_namespaces_are_the_same_vocabulary() {
    for namespace in [
        MarkupNamespace::Html,
        MarkupNamespace::Svg,
        MarkupNamespace::MathMl,
    ] {
        assert_eq!(through_ast(through_markup(namespace)), namespace);
    }

    assert_eq!(MarkupNamespace::Html as u8, 0);
    assert_eq!(MarkupNamespace::Svg as u8, 1);
    assert_eq!(MarkupNamespace::MathMl as u8, 2);
}

use super::{operands, selected};
use vize_glyph::native_doc::{
    ExpressionRefusal, NativeTemplateRefusal, TemplateRefusal, UnsupportedSyntax,
    native_template_document, template_document,
};
use vize_l0::Allocator;
use vize_l1::embed::{Embed, EmbedSource, Grammar, Lang, Shape, syntax::parse_once};

#[test]
fn missing_extra_reversed_and_duplicate_operands_are_explicitly_refused() {
    let arena = Allocator::default();
    let source = "<template>{{ a }}<p>{{ b }}</p></template>";
    let selected = selected(&arena, source);
    let original = operands(&selected);
    let a = original.first().unwrap();
    let b = original.get(1).unwrap();
    assert!(matches!(
        native_template_document(&selected, &[], &arena),
        Err(NativeTemplateRefusal::MissingOperand { index: 0, .. })
    ));
    assert!(matches!(
        native_template_document(&selected, &[a], &arena),
        Err(NativeTemplateRefusal::MissingOperand { index: 1, .. })
    ));
    assert!(matches!(
        native_template_document(&selected, &[a, b, a], &arena),
        Err(NativeTemplateRefusal::ExtraOperand { index: 2 })
    ));
    for refs in [[b, a], [a, a]] {
        assert!(matches!(
            native_template_document(&selected, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected { .. })
        ));
    }
    assert_eq!(original.first().unwrap().raw_content(), " a ");
    assert_eq!(original.get(1).unwrap().raw_content(), " b ");
}

#[test]
fn foreign_equal_bytes_and_independent_equal_input_owners_cannot_admit() {
    let arena = Allocator::default();
    let source = "<template>{{ a }}</template>";
    let selected_owner = selected(&arena, source);
    let original = operands(&selected_owner);
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    let independent = selected(&arena, source);
    let copied = vize_l0::String::from(source);
    let foreign = selected(&arena, copied.as_str());
    for owner in [&independent, &foreign] {
        assert!(matches!(
            native_template_document(owner, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected { .. })
        ));
    }
    assert!(native_template_document(&selected_owner, &refs, &arena).is_ok());
}

#[test]
fn empty_syntax_holes_and_unsupported_expression_descendants_keep_observations() {
    for source in [
        "<template>{{ }}</template>",
        "<template>{{ /*kept*/ a+ }}</template>",
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let original = operands(&selected);
        let operand = original.first().unwrap();
        let diagnostics = operand.syntax().diagnostics().count();
        let comments = operand.syntax().comments().count();
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        assert!(matches!(
            native_template_document(&selected, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected { hole: Some(_), .. })
        ));
        assert_eq!(operand.syntax().diagnostics().count(), diagnostics);
        assert_eq!(operand.syntax().comments().count(), comments);
        assert_eq!(operand.content_span().slice(source), operand.raw_content());
    }
    for source in [
        "<template>{{a?.value}}</template>",
        "<template>{{call()}}</template>",
        "<template>{{a + call()}}</template>",
        "<template>{{value as boolean}}</template><script setup lang=ts>let value=true</script>",
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let original = operands(&selected);
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        assert!(matches!(
            native_template_document(&selected, &refs, &arena),
            Err(NativeTemplateRefusal::Expression {
                refusal: ExpressionRefusal::UnsupportedNode { .. },
                ..
            })
        ));
        assert!(original.first().unwrap().syntax().expression().is_some());
    }
}

#[test]
fn generic_authored_entity_string_is_admitted_raw_but_actual_native_context_refuses() {
    let arena = Allocator::default();
    let source = "<template>{{\"a&#10;b\"}}</template>";
    let selected = selected(&arena, source);
    let original = operands(&selected);
    let operand = original.first().unwrap();
    let raw = parse_once(
        &arena,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
            source: EmbedSource::authored(source, operand.content_span()).unwrap(),
        },
    )
    .into_expression()
    .unwrap();
    assert!(raw.admitted_expression().is_some());
    assert_eq!(raw.source().text(), "\"a&#10;b\"");
    assert_eq!(operand.syntax().source().text(), "\"a\nb\"");
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    assert!(matches!(
        native_template_document(&selected, &refs, &arena),
        Err(NativeTemplateRefusal::OperandRejected { hole: Some(_), .. })
    ));
    assert_eq!(raw.source().span(), operand.syntax().source().span());
    assert!(operand.syntax().diagnostics().count() > 0);
}

#[test]
fn recovered_template_refuses_and_bare_api_keeps_interpolation_refusal() {
    let arena = Allocator::default();
    let selected = selected(&arena, "<template><p></template>");
    assert!(matches!(
        native_template_document(&selected, &[], &arena),
        Err(NativeTemplateRefusal::Template(
            TemplateRefusal::Recovered { .. }
        ))
    ));
    let bare = vize_l1::dialect::vue3::surface::parse_component(&arena, "<p>{{ a }}</p>").unwrap();
    assert!(matches!(
        template_document(&bare, &arena),
        Err(TemplateRefusal::Unsupported {
            syntax: UnsupportedSyntax::Interpolation,
            ..
        })
    ));
}

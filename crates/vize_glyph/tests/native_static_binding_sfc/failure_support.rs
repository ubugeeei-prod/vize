use super::{assert_original, options};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{
    LineEnding, NativeSfcObservation, NativeSfcRefusal, ObservedNativeTemplateRefusal,
};
use vize_l0::Span;
use vize_l1::{SurfaceChild, markup::NativeConditionKind};

pub(super) fn refused(owner: &NativeSfcObservation<'_>, source: &str, expected: NativeSfcRefusal) {
    assert_eq!(owner.source(), source);
    assert!(core::ptr::eq(owner.source(), source));
    assert_eq!(owner.options(), options(200, 2, LineEnding::Lf));
    assert_eq!(owner.descriptor().options(), owner.options().descriptor);
    for _ in 0..2 {
        assert_eq!(owner.refusal(), Some(expected));
        assert_eq!(owner.document().unwrap_err(), expected);
        assert_eq!(owner.format().unwrap_err(), expected);
    }
}

pub(super) fn document(refusal: vize_glyph::native_doc::NativeTemplateRefusal) -> NativeSfcRefusal {
    NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document { index: 1, refusal })
}

pub(super) fn prefix(
    owner: &NativeSfcObservation<'_>,
    source: &str,
    expected: NativeSfcRefusal,
    binding_count: usize,
    trailing: Span,
) {
    assert_original(owner, source, options(200, 2, LineEnding::Lf));
    refused(owner, source, expected);
    assert_eq!(owner.binding_operands().len(), binding_count);
    assert_eq!(owner.attribute_operands().len(), 1);
    assert_eq!(owner.operands().len(), 1);
    assert!(owner.attribute_failure().is_none());
    assert!(owner.interpolation_failure().is_none());
    let selected = owner.selected().unwrap();
    assert_eq!(
        selected.component().block().source(),
        &source[10..source.len() - 11]
    );
    let first = selected.children().next().unwrap().into_element().unwrap();
    let second = selected.children().nth(1).unwrap().into_element().unwrap();
    let binding = &owner.binding_operands()[0];
    assert_eq!(binding.name_span(), Span::new(13, 19));
    assert_eq!(binding.argument_span(), Span::new(14, 19));
    assert_eq!(binding.value_span(), Span::new(21, 26));
    assert_eq!(binding.raw_value(), "first");
    assert_eq!(binding.syntax().source().text(), "first");
    assert_eq!(binding.syntax().hole(), None);
    assert_eq!(binding.syntax().comments().count(), 0);
    assert_eq!(binding.syntax().diagnostics().count(), 0);
    let view = binding
        .admitted_for(selected, first.attributes().next().unwrap())
        .unwrap();
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        binding.syntax().expression().unwrap()
    ));
    let Expression::Identifier(identifier) = binding.syntax().expression().unwrap() else {
        panic!("original prefix binding")
    };
    assert_eq!(identifier.name.as_str(), "first");
    assert_eq!(identifier.span, oxc_span::Span::new(2, 7));
    let conditional = &owner.attribute_operands()[0];
    assert_eq!(conditional.kind(), NativeConditionKind::If);
    assert_eq!(conditional.name_span(), Span::new(41, 45));
    assert_eq!(conditional.value_span(), Span::new(47, 49));
    assert_eq!(conditional.raw_value(), "ok");
    assert_eq!(conditional.syntax().source().text(), "ok");
    assert!(
        conditional
            .admitted_for(selected, second.attributes().next().unwrap())
            .is_some()
    );
    let Expression::Identifier(identifier) = conditional.syntax().expression().unwrap() else {
        panic!("original prefix conditional")
    };
    assert_eq!(identifier.name.as_str(), "ok");
    assert_eq!(identifier.span, oxc_span::Span::new(2, 4));
    let interpolation = &owner.operands()[0];
    assert_eq!(interpolation.full_span(), Span::new(28, 34));
    assert_eq!(interpolation.raw_content(), "1n");
    assert_eq!(interpolation.syntax().source().text(), "1n");
    assert!(
        interpolation
            .admitted_for(selected, first.children().next().unwrap())
            .is_some()
    );
    let Expression::BigIntLiteral(bigint) = interpolation.syntax().expression().unwrap() else {
        panic!("original prefix bigint")
    };
    assert_eq!(bigint.raw.as_ref().unwrap().as_str(), "1n");
    assert_eq!(second.children().len(), 1);
    assert_eq!(trailing.slice(source), "{{later}}");
    let child = second.children().next().unwrap();
    let SurfaceChild::Interpolation(surface) = child.surface() else {
        panic!("unvisited original child")
    };
    assert_eq!(surface.open.text, "{{");
    assert_eq!(surface.content.text, "later");
    assert_eq!(surface.close.text, "}}");
    for syntax in [
        binding.syntax(),
        conditional.syntax(),
        interpolation.syntax(),
    ] {
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert_eq!(syntax.diagnostics().count(), 0);
    }
}

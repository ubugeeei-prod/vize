//! Genuine full-owner custody across ordinary wrapper moves.

use oxc_ast::ast::{BigintBase, Expression};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcObservation, NativeSfcOptions, NativeSfcRefusal, NativeTemplateRefusal,
    native_template_document, observe_native_sfc_in, print,
};
use vize_l0::Allocator;
use vize_l0::config::VueVersion;
use vize_l1::SurfaceChild;
use vize_l1::container::ContainerErrorCode;
use vize_l1::container::vue::DescriptorIssueCode;
use vize_l1::markup::{NativeChild, NativeChildren, NativeTemplateComponent};

fn options() -> NativeSfcOptions {
    let mut options = NativeSfcOptions::default();
    options.print.width = 200;
    options.print.line_ending = LineEnding::CrLf;
    options
}

// Test-only inspection of the actual immutable body order, not a production pass count.
fn interpolations<'s, 'a>(selected: &'s NativeTemplateComponent<'a>) -> Vec<NativeChild<'s, 'a>> {
    fn visit<'s, 'a>(children: NativeChildren<'s, 'a>, output: &mut Vec<NativeChild<'s, 'a>>) {
        for child in children {
            match child.surface() {
                SurfaceChild::Interpolation(_) => output.push(child),
                SurfaceChild::Element(_) => visit(child.into_element().unwrap().children(), output),
                _ => {}
            }
        }
    }
    let mut output = Vec::new();
    visit(selected.children(), &mut output);
    output
}

#[test]
fn full_owner_moves_keep_original_descriptor_component_ordered_ast_maps_comments_and_pure_storage()
{
    let arena = Allocator::default();
    let source = "<!--前-->\r\n<template lang='html'>{{/*x*/a&#43;1n}}<p>{{/*#__PURE__*/f&#40;0x2_A&#110;&#41;}}</p>{{a,b}}</template>\r\n";
    let owner = observe_native_sfc_in(&arena, source, options());
    let snapshot = |owner: &NativeSfcObservation<'_>| {
        let selected = owner.selected().unwrap();
        (
            (owner.source().as_ptr(), owner.source().len()),
            owner.options(),
            owner.descriptor().container().blocks.as_ptr().cast::<()>(),
            owner.descriptor().root().unwrap().source().as_ptr(),
            (
                selected.template_index(),
                selected.grammar(),
                selected.component().block().span(),
                selected.component().block().source().as_ptr(),
                selected
                    .component()
                    .carrier()
                    .tree
                    .children
                    .as_ptr()
                    .cast::<()>(),
            ),
            owner.operands().as_ptr().cast::<()>(),
            owner
                .operands()
                .iter()
                .map(|operand| {
                    let syntax = operand.syntax();
                    let view = syntax.source();
                    (
                        core::ptr::from_ref(syntax.expression().unwrap()).cast::<()>(),
                        (operand.full_span(), operand.content_span(), view.span()),
                        (
                            operand.raw_content().as_ptr(),
                            operand.raw_content().to_owned(),
                        ),
                        (
                            view.text().as_ptr(),
                            view.text().to_owned(),
                            view.authored_root().as_ptr(),
                        ),
                        view.decode_map()
                            .map(|map| (map.segments().as_ptr(), map.segments().to_vec())),
                        syntax
                            .comments()
                            .map(|comment| {
                                (
                                    comment.kind(),
                                    comment.decoded_span().unwrap(),
                                    comment.authored_span().unwrap(),
                                    comment.text().unwrap().as_ptr(),
                                    comment.text().unwrap().to_owned(),
                                )
                            })
                            .collect::<Vec<_>>(),
                        syntax.grammar(),
                        syntax.source_type(),
                        syntax.hole(),
                        syntax.diagnostics().count(),
                    )
                })
                .collect::<Vec<_>>(),
            owner.refusal(),
        )
    };
    let before = snapshot(&owner);
    assert_eq!(owner.operands().len(), 3);
    assert!(owner.descriptor().issues().is_empty());
    assert!(owner.descriptor().container().errors.is_empty());
    assert!(owner.interpolation_failure().is_none());
    let Expression::CallExpression(call) = owner.operands()[1].syntax().expression().unwrap()
    else {
        panic!("original pure Call")
    };
    assert!(call.pure && !call.optional && call.type_arguments.is_none());
    let arguments = call.arguments.as_ptr();
    let Expression::BigIntLiteral(literal) = call.arguments[0].as_expression().unwrap() else {
        panic!("original mapped BigInt argument")
    };
    assert_eq!(
        (
            literal.base,
            literal.value.as_str(),
            literal.raw.unwrap().as_str()
        ),
        (BigintBase::Hex, "42", "0x2_An")
    );
    let scalar = core::ptr::from_ref(&**literal);
    let value = literal.value.as_str().as_ptr();
    let raw = literal.raw.unwrap().as_str().as_ptr();
    let mut parked = vec![owner];
    parked.reserve(32);
    let moved = parked.pop().unwrap();
    assert_eq!(snapshot(&moved), before);
    let selected = moved.selected().unwrap();
    for (operand, child) in moved.operands().iter().zip(interpolations(selected)) {
        assert!(operand.admitted_for(selected, child).is_some());
        assert_eq!(operand.content_span().slice(source), operand.raw_content());
    }
    let Expression::CallExpression(call) = moved.operands()[1].syntax().expression().unwrap()
    else {
        panic!("same Call")
    };
    assert!(call.pure && !call.optional && call.type_arguments.is_none());
    assert_eq!(call.arguments.as_ptr(), arguments);
    let Expression::BigIntLiteral(literal) = call.arguments[0].as_expression().unwrap() else {
        panic!("same BigInt argument")
    };
    assert_eq!(core::ptr::from_ref(&**literal), scalar);
    assert_eq!(literal.value.as_str().as_ptr(), value);
    assert_eq!(literal.raw.unwrap().as_str().as_ptr(), raw);
    let expected = "<!--前-->\r\n<template lang='html'>{{ /*x*/a &#43; 1n }}<p>{{ /*#__PURE__*/f &#40; 0x2_A&#110; &#41; }}</p>{{ a, b }}</template>\r\n";
    for _ in 0..3 {
        let result = moved.format().unwrap();
        assert_eq!(result.code, expected);
        assert!(result.changed);
        assert_eq!(
            print(moved.document().unwrap(), &moved.options().print),
            expected
        );
        assert_eq!(snapshot(&moved), before);
    }
    let stable = observe_native_sfc_in(&arena, expected, options());
    assert_eq!(stable.format().unwrap().code, expected);
    assert!(!stable.format().unwrap().changed);
}

#[test]
fn moved_full_owner_receipts_refuse_independent_equal_byte_owners_and_copied_roots() {
    let arena = Allocator::default();
    let source = std::string::String::from("<!--前--><template>{{a&#43;1n}}</template>\n");
    let owner = observe_native_sfc_in(&arena, &source, options());
    let mut parked = vec![owner];
    parked.reserve(16);
    let moved = parked.pop().unwrap();
    let selected = moved.selected().unwrap();
    let original = &moved.operands()[0];
    let root = core::ptr::from_ref(original.syntax().expression().unwrap());
    let refs = [original];
    let copied = source.clone();
    for foreign in [
        observe_native_sfc_in(&arena, &source, options()),
        observe_native_sfc_in(&arena, &copied, options()),
    ] {
        let foreign_selected = foreign.selected().unwrap();
        assert!(
            original
                .admitted_for(
                    foreign_selected,
                    foreign_selected.children().next().unwrap()
                )
                .is_none()
        );
        assert!(matches!(
            native_template_document(foreign_selected, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected {
                offset: 0,
                index: 0,
                hole: None
            })
        ));
        assert_eq!(
            core::ptr::from_ref(original.syntax().expression().unwrap()),
            root
        );
        assert_eq!(
            foreign.format().unwrap().code,
            "<!--前--><template>{{ a &#43; 1n }}</template>\n"
        );
    }
    assert!(
        original
            .admitted_for(selected, selected.children().next().unwrap())
            .is_some()
    );
    assert!(native_template_document(selected, &refs, &arena).is_ok());
    assert_eq!(moved.source(), source);
    assert_eq!(
        moved.format().unwrap().code,
        "<!--前--><template>{{ a &#43; 1n }}</template>\n"
    );
}

#[test]
fn moved_refused_owner_keeps_explicit_options_original_descriptor_source_errors_and_no_partial_doc()
{
    let arena = Allocator::default();
    let source = "<template>{{1n}}</template><script";
    let mut options = options();
    options.descriptor.version = VueVersion::V2;
    options.print.width = 17;
    options.print.indent_width = 3;
    let owner = observe_native_sfc_in(&arena, source, options);
    let blocks = owner.descriptor().container().blocks.as_ptr();
    let issues = owner.descriptor().issues().to_vec();
    let errors = owner
        .descriptor()
        .container()
        .errors
        .iter()
        .map(|error| (error.code, error.offset))
        .collect::<Vec<_>>();
    assert_eq!(errors, [(ContainerErrorCode::UnterminatedOpenTag, 27)]);
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == DescriptorIssueCode::UnsupportedVersion)
    );
    let mut parked = vec![owner];
    parked.reserve(32);
    let moved = parked.pop().unwrap();
    assert_eq!(moved.options(), options);
    assert!(core::ptr::eq(moved.source(), source));
    assert_eq!(moved.descriptor().container().blocks.as_ptr(), blocks);
    assert_eq!(moved.descriptor().issues(), issues.as_slice());
    assert_eq!(
        moved
            .descriptor()
            .container()
            .errors
            .iter()
            .map(|error| (error.code, error.offset))
            .collect::<Vec<_>>(),
        errors
    );
    assert_eq!(moved.refusal(), Some(NativeSfcRefusal::Descriptor));
    assert!(matches!(
        moved.document(),
        Err(NativeSfcRefusal::Descriptor)
    ));
    assert!(matches!(moved.format(), Err(NativeSfcRefusal::Descriptor)));
    assert!(moved.selected().is_none());
    assert!(moved.operands().is_empty());
    assert!(moved.interpolation_failure().is_none());
}

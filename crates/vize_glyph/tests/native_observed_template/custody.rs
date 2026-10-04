//! Owned observations remain original across carrier and vector moves.

use oxc_ast::ast::{BigintBase, Expression};
use vize_glyph::native_doc::{
    LineEnding, NativeTemplateRefusal, native_template_document, observed_native_template_document,
    print,
};
use vize_l0::Allocator;
use vize_l1::markup::NativeInterpolationOperand;

use super::{SCRIPTS, WIDTHS, options, preobserved, selected};

#[test]
fn owned_ordered_ast_comments_maps_spans_and_pure_metadata_survive_carrier_and_vec_growth() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<!--前--><template>{{{{/*x*/1&#110;+2n}}}}<p>{{{{/*#__PURE__*/f&#40;0x2_A&#110;&#41;}}}}</p>{{{{a,b}}}}</template>{script}"
        );
        let owner = selected(&arena, &source);
        let carrier = observed_native_template_document(&owner, &arena).unwrap();
        assert_eq!(carrier.operands().len(), 3);
        let observe = |operands: &[NativeInterpolationOperand<'_>]| {
            operands
                .iter()
                .map(|operand| {
                    let syntax = operand.syntax();
                    let view = syntax.source();
                    (
                        core::ptr::from_ref(syntax.expression().unwrap()).cast::<()>(),
                        (
                            operand.raw_content().as_ptr(),
                            operand.raw_content().to_owned(),
                        ),
                        (operand.full_span(), operand.content_span(), view.span()),
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
                            .collect::<std::vec::Vec<_>>(),
                        syntax.grammar(),
                        syntax.source_type(),
                        syntax.hole(),
                        syntax.diagnostics().count(),
                    )
                })
                .collect::<std::vec::Vec<_>>()
        };
        let before = observe(carrier.operands());
        let storage = carrier.operands().as_ptr();
        let Expression::BinaryExpression(binary) =
            carrier.operands()[0].syntax().expression().unwrap()
        else {
            panic!("original Binary")
        };
        let Expression::BigIntLiteral(literal) = &binary.left else {
            panic!("original mapped BigInt")
        };
        assert_eq!(
            (
                literal.base,
                literal.value.as_str(),
                literal.raw.unwrap().as_str()
            ),
            (BigintBase::Decimal, "1", "1n")
        );
        let scalar = core::ptr::from_ref(&**literal);
        let value = literal.value.as_str().as_ptr();
        let raw = literal.raw.unwrap().as_str().as_ptr();
        let Expression::CallExpression(call) = carrier.operands()[1].syntax().expression().unwrap()
        else {
            panic!("original pure Call")
        };
        assert!(call.pure && !call.optional && call.type_arguments.is_none());
        let arguments = call.arguments.as_ptr();
        let mut parked = std::vec![carrier];
        parked.reserve(32);
        let moved = parked.pop().unwrap();
        assert_eq!(moved.operands().as_ptr(), storage);
        assert_eq!(observe(moved.operands()), before);
        let (same, mut owned, document) = moved.into_parts();
        assert!(core::ptr::eq(same, &owner));
        owned.reserve(32);
        assert_eq!(observe(&owned), before);
        let Expression::BinaryExpression(binary) = owned[0].syntax().expression().unwrap() else {
            panic!("same Binary")
        };
        let Expression::BigIntLiteral(literal) = &binary.left else {
            panic!("same BigInt")
        };
        assert_eq!(core::ptr::from_ref(&**literal), scalar);
        assert_eq!(literal.value.as_str().as_ptr(), value);
        assert_eq!(literal.raw.unwrap().as_str().as_ptr(), raw);
        let Expression::CallExpression(call) = owned[1].syntax().expression().unwrap() else {
            panic!("same Call")
        };
        assert!(call.pure && !call.optional && call.type_arguments.is_none());
        assert_eq!(call.arguments.as_ptr(), arguments);
        let refs = owned.iter().collect::<std::vec::Vec<_>>();
        let borrowed = native_template_document(&owner, &refs, &arena).unwrap();
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                assert_eq!(
                    print(&document, &options(width, ending)),
                    print(borrowed.document(), &options(width, ending))
                );
                assert_eq!(observe(&owned), before);
            }
        }
        assert_eq!(
            print(&document, &options(200, LineEnding::Lf)),
            "{{ /*x*/1&#110; + 2n }}<p>{{ /*#__PURE__*/f &#40; 0x2_A&#110; &#41; }}</p>{{ a, b }}"
        );
        assert_eq!(
            owned
                .iter()
                .map(|operand| operand.raw_content())
                .collect::<std::vec::Vec<_>>(),
            [
                "/*x*/1&#110;+2n",
                "/*#__PURE__*/f&#40;0x2_A&#110;&#41;",
                "a,b"
            ]
        );
        assert!(
            owned
                .windows(2)
                .all(|pair| pair[0].full_span().end <= pair[1].full_span().start)
        );
        assert!(owned.iter().all(|operand| core::ptr::eq(
            operand.syntax().source().authored_root(),
            source.as_str()
        )));
    }
}

#[test]
fn preobserved_receiver_and_original_observations_stay_immutable_beside_owned_building() {
    let arena = Allocator::default();
    let source = "<template>{{1n}}<p>{{a+b}}</p></template>";
    let owner = selected(&arena, source);
    let original = preobserved(&owner);
    let roots = original
        .iter()
        .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
        .collect::<std::vec::Vec<_>>();
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    let before = native_template_document(&owner, &refs, &arena).unwrap();
    let observed = observed_native_template_document(&owner, &arena).unwrap();
    assert_eq!(observed.operands().len(), 2);
    for width in WIDTHS {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(
                print(before.document(), &options(width, ending)),
                print(observed.document(), &options(width, ending))
            );
        }
    }
    assert_eq!(
        original
            .iter()
            .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
            .collect::<std::vec::Vec<_>>(),
        roots
    );
    assert_eq!(
        print(observed.document(), &options(200, LineEnding::Lf)),
        "{{ 1n }}<p>{{ a + b }}</p>"
    );
    assert!(
        original
            .iter()
            .all(|operand| operand.syntax().hole().is_none())
    );
    // Each receiver owns its actual observations; this is not a parse-count assertion.
}

#[test]
fn transferred_operands_keep_original_owner_authority_and_reject_foreign_equal_byte_roots() {
    let arena = Allocator::default();
    let source = std::string::String::from("<template>{{1n}}</template>");
    let owner = selected(&arena, &source);
    let (same, owned, document) = observed_native_template_document(&owner, &arena)
        .unwrap()
        .into_parts();
    assert!(core::ptr::eq(same, &owner));
    let root = core::ptr::from_ref(owned[0].syntax().expression().unwrap());
    let refs = owned.iter().collect::<std::vec::Vec<_>>();
    let copied = source.clone();
    for foreign in [selected(&arena, &source), selected(&arena, &copied)] {
        assert!(
            owned[0]
                .admitted_for(&foreign, foreign.children().next().unwrap())
                .is_none()
        );
        assert!(matches!(
            native_template_document(&foreign, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected { index: 0, .. })
        ));
        assert_eq!(
            core::ptr::from_ref(owned[0].syntax().expression().unwrap()),
            root
        );
    }
    assert!(
        owned[0]
            .admitted_for(&owner, owner.children().next().unwrap())
            .is_some()
    );
    assert_eq!(print(&document, &options(200, LineEnding::Lf)), "{{ 1n }}");
}

#[test]
fn bare_doc_transfer_can_release_observation_and_selection_wrappers_with_source_and_arena_alive() {
    assert!(core::mem::needs_drop::<NativeInterpolationOperand<'_>>());
    let arena = Allocator::default();
    let source = "<template>{{&#49;&#110;}}</template>";
    let document = {
        let owner = core::hint::black_box(selected(&arena, source));
        let carrier = observed_native_template_document(&owner, &arena).unwrap();
        let (same, owned, document) = carrier.into_parts();
        assert!(core::ptr::eq(same, &owner));
        assert_eq!(owned.len(), 1);
        drop(owned);
        document
    };
    // Bare Doc keeps authored source/arena storage, without retaining selection authority.
    assert_eq!(
        print(&document, &options(200, LineEnding::Lf)),
        "{{ &#49;&#110; }}"
    );
}

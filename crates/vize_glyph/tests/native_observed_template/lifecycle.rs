//! Actual normal ownership under unwind and deliberate forget, without drop counters.

use oxc_span::GetSpan;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, ObservedNativeTemplateRefusal,
    native_template_document, observed_native_template_document, print,
};
use vize_l0::{Allocator, Span};
use vize_l1::markup::NativeInterpolationOperand;

use super::{SCRIPTS, WIDTHS, assert_fixed, newline, options, selected};

#[test]
fn real_success_and_failure_carrier_unwind_keeps_original_selected_source_maps_and_next_use() {
    for script in SCRIPTS {
        for (body, decoded, refusal) in [
            ("{{1n}}<p>{{a&#43;2n}}</p>", "a+2n", None),
            (
                "{{1n}}<p>{{a&#43;f(...b)}}</p>{{later}}",
                "a+f(...b)",
                Some(ObservedNativeTemplateRefusal::Document {
                    index: 1,
                    refusal: NativeTemplateRefusal::Expression {
                        offset: 9,
                        refusal: ExpressionRefusal::UnsupportedNode {
                            span: Span::new(2, 9),
                        },
                    },
                }),
            ),
        ] {
            let arena = Allocator::default();
            let source = vize_l0::cstr!("<template>{body}</template>{script}");
            let owner = selected(&arena, &source);
            let original = core::ptr::from_ref(&owner.component().carrier().tree);
            let carrier = observed_native_template_document(&owner, &arena);
            let operands = match &carrier {
                Ok(success) => success.operands(),
                Err(failure) => {
                    assert_eq!(Some(failure.refusal()), refusal);
                    assert!(failure.interpolation_failure().is_none());
                    failure.operands()
                }
            };
            assert_eq!(operands.len(), 2);
            let spans = operands
                .iter()
                .map(|operand| (operand.full_span(), operand.content_span()))
                .collect::<std::vec::Vec<_>>();
            let current = &operands[1];
            let root = current.syntax().expression().unwrap();
            let root_span = root.span();
            let view = current.syntax().source();
            let map = view.decode_map().unwrap().segments();
            let map_pointer = map.as_ptr();
            let map_values = map.to_vec();
            assert_eq!(view.text(), decoded);
            let result: Result<(), _> = catch_unwind(AssertUnwindSafe(move || {
                let _carrier = core::hint::black_box(carrier);
                panic!("unwind the actual owned template carrier");
            }));
            assert!(result.is_err());
            assert_eq!(
                core::ptr::from_ref(&owner.component().carrier().tree),
                original
            );
            assert_eq!(owner.component().block().source(), body);
            assert!(core::ptr::eq(
                owner.component().block().root_source(),
                source.as_str()
            ));
            assert_eq!(root.span(), root_span);
            assert_eq!(view.text(), decoded);
            assert_eq!(view.decode_map().unwrap().segments().as_ptr(), map_pointer);
            assert_eq!(view.decode_map().unwrap().segments(), map_values.as_slice());
            assert!(core::ptr::eq(view.authored_root(), source.as_str()));
            let next = observed_native_template_document(&owner, &arena);
            let next_operands = match &next {
                Ok(success) => {
                    assert_eq!(refusal, None);
                    for width in WIDTHS {
                        for ending in [LineEnding::Lf, LineEnding::CrLf] {
                            let expected = if width < 80 {
                                "{{\n  1n\n}}<p>{{\n    a &#43;\n      2n\n  }}</p>"
                                    .replace('\n', newline(ending))
                            } else {
                                "{{ 1n }}<p>{{ a &#43; 2n }}</p>".to_owned()
                            };
                            assert_eq!(
                                print(success.document(), &options(width, ending)),
                                expected
                            );
                        }
                    }
                    assert_fixed(&source, script);
                    success.operands()
                }
                Err(failure) => {
                    assert_eq!(Some(failure.refusal()), refusal);
                    assert!(failure.interpolation_failure().is_none());
                    failure.operands()
                }
            };
            assert_eq!(
                next_operands
                    .iter()
                    .map(|operand| (operand.full_span(), operand.content_span()))
                    .collect::<std::vec::Vec<_>>(),
                spans
            );
            assert_eq!(next_operands[1].syntax().source().text(), decoded);
            assert_eq!(
                next_operands[1]
                    .syntax()
                    .source()
                    .decode_map()
                    .unwrap()
                    .segments(),
                map_values.as_slice()
            );
        }
    }
    // These are real unwind paths; no instrumented destructor or parse-count claim.
}

#[test]
fn forgetting_transferred_owned_remainder_preserves_doc_bytes_without_granting_owner_admission() {
    assert!(core::mem::needs_drop::<
        std::vec::Vec<NativeInterpolationOperand<'_>>,
    >());
    let arena = Allocator::default();
    let source = std::string::String::from("<template>{{1n}}{{a&#43;2n}}</template>");
    let owner = selected(&arena, &source);
    let carrier = observed_native_template_document(&owner, &arena).unwrap();
    let mut parked = std::vec![carrier];
    let (same, mut owned, document) = parked.pop().unwrap().into_parts();
    assert!(core::ptr::eq(same, &owner));
    let kept = owned.remove(0);
    assert_eq!(owned.len(), 1);
    let root = owned[0].syntax().expression().unwrap();
    let root_span = root.span();
    let view = owned[0].syntax().source();
    let map = view.decode_map().unwrap().segments();
    let map_values = map.to_vec();
    core::mem::forget(owned);
    // This deliberately leaks ordinary observations; it creates no new body origin.
    assert_eq!(root.span(), root_span);
    assert_eq!(view.text(), "a+2n");
    assert_eq!(view.decode_map().unwrap().segments(), map_values.as_slice());
    assert_eq!(view.decode_map().unwrap().segments().as_ptr(), map.as_ptr());
    assert!(core::ptr::eq(view.authored_root(), source.as_str()));
    assert_eq!(
        print(&document, &options(200, LineEnding::Lf)),
        "{{ 1n }}{{ a &#43; 2n }}"
    );
    assert!(
        kept.admitted_for(&owner, owner.children().next().unwrap())
            .is_some()
    );
    let refs = [&kept];
    assert!(matches!(
        native_template_document(&owner, &refs, &arena),
        Err(NativeTemplateRefusal::MissingOperand {
            offset: 6,
            index: 1
        })
    ));
    let copied = source.clone();
    for foreign in [selected(&arena, &source), selected(&arena, &copied)] {
        assert!(
            kept.admitted_for(&foreign, foreign.children().next().unwrap())
                .is_none()
        );
        assert!(matches!(
            native_template_document(&foreign, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected {
                offset: 0,
                index: 0,
                hole: None
            })
        ));
    }
    assert_eq!(kept.raw_content(), "1n");
    assert_fixed(&source, "");
    // The transferred bare Doc covers this template body, without enclosing-SFC admission.
}

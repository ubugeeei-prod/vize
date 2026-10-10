use super::{L2Item, walk_items};
use crate::markup::L2Template;
use vize_l0::{Allocator, Box, Span};
use vize_l2::op::{Attribute, BindingOp, VueOnceOp};

#[test]
fn merging_preserves_authored_order_and_static_ties_with_either_tail() {
    type Case<'a> = (&'a [u32], &'a [u32], &'a [(bool, u32)]);
    let cases: &[Case<'_>] = &[
        (&[], &[], &[]),
        (&[1, 2, 3], &[], &[(false, 1), (false, 2), (false, 3)]),
        (&[], &[1, 2, 3], &[(true, 1), (true, 2), (true, 3)]),
        (
            &[1, 3, 5],
            &[2, 4],
            &[(false, 1), (true, 2), (false, 3), (true, 4), (false, 5)],
        ),
        (
            &[2, 4],
            &[1, 3, 5],
            &[(true, 1), (false, 2), (true, 3), (false, 4), (true, 5)],
        ),
        (
            &[1, 2],
            &[1, 2, 3],
            &[(false, 1), (true, 1), (false, 2), (true, 2), (true, 3)],
        ),
        (
            &[1, 2, 3],
            &[3],
            &[(false, 1), (false, 2), (false, 3), (true, 3)],
        ),
    ];
    let allocator = Allocator::new();
    let lowered = L2Template::lower(&allocator, "");
    let document = lowered.markup();
    for &(attribute_spans, binding_spans, expected) in cases {
        let attributes: std::vec::Vec<_> = attribute_spans
            .iter()
            .map(|&start| Attribute {
                name: "authored",
                value: None,
                span: Span::new(start, start + 1),
            })
            .collect();
        let bindings: std::vec::Vec<_> = binding_spans
            .iter()
            .map(|&start| {
                BindingOp::VueOnce(Box::new_in(
                    VueOnceOp {
                        span: Span::new(start, start + 1),
                    },
                    &&allocator,
                ))
            })
            .collect();
        let mut observed = std::vec::Vec::new();
        walk_items(&document, &attributes, &bindings, None, &mut |item| {
            observed.push((matches!(item, L2Item::Binding(_)), item.span().start));
        });
        assert_eq!(
            observed, expected,
            "attributes={attribute_spans:?}, bindings={binding_spans:?}"
        );
    }
}

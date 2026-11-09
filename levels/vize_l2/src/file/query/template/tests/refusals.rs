use super::{
    Allocator, PositionQueryError, TemplateQueryError, at, attempt, file, native, site, uses,
};

#[test]
fn equal_bytes_and_equal_local_ids_never_transfer_file_or_handler_symbols() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items' @click='$event;item' @blur='$event'/></template>";
    let copied_bytes = alloc::vec::Vec::from(source.as_bytes());
    let copied = core::str::from_utf8(&copied_bytes).unwrap();
    let left_output = native(&arena, source);
    let right_output = native(&arena, copied);
    let left = file(&left_output);
    let right = file(&right_output);
    for needle in ["item in items", "$event;item"] {
        let local = site(left, source, needle);
        let foreign = site(right, copied, needle);
        assert!(!local.symbol().same_symbol(foreign.symbol()));
        let mut visits = 0;
        assert_eq!(
            left.for_each_template_reference_to(foreign.symbol(), |_| visits += 1),
            Err(TemplateQueryError::ForeignSymbol)
        );
        assert_eq!(visits, 0);
    }
    let first = site(left, source, "$event;item");
    let second = site(left, source, "$event'/>");
    let super::TemplateSymbolRef::HandlerLocal(first_local) = first.symbol() else {
        panic!("local")
    };
    let super::TemplateSymbolRef::HandlerLocal(second_local) = second.symbol() else {
        panic!("local")
    };
    assert_eq!(first_local.binding().id, second_local.binding().id);
    assert!(!first_local.same_symbol(second_local));
    assert_eq!(uses(left, first.symbol()), [first.span()]);
    assert_eq!(uses(left, second.symbol()), [second.span()]);
}

#[test]
fn actual_late_header_refusal_never_grants_partial_query_completion() {
    let arena = Allocator::default();
    let source = "<script setup>const value=1;</script><template><button @click='value'/><button @blur='$event' :id='missing'/></template>";
    let output = attempt(&arena, source, false);
    assert!(output.view().is_err());
    let partial = output.file().unwrap();
    assert!(!partial.is_complete());
    assert!(partial.unattached_handlers().next().is_some());
    for offset in [0, at(source, "$event"), source.len() as u32, u32::MAX] {
        assert!(matches!(
            partial.template_symbol_at_offset(offset),
            Err(TemplateQueryError::Position(
                PositionQueryError::IncompleteFile
            ))
        ));
    }
    let binding = partial.bindings().next().unwrap();
    let mut visits = 0;
    assert_eq!(
        partial
            .for_each_template_reference_to(super::TemplateSymbolRef::File(binding), |_| visits +=
                1),
        Err(TemplateQueryError::Position(
            PositionQueryError::IncompleteFile
        ))
    );
    assert_eq!(visits, 0);
}

#[test]
fn moved_original_owner_repeated_queries_do_not_change_original_arena_or_sites() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items' @click='item'/></template>";
    let output = core::hint::black_box(native(&arena, source));
    let file = file(&output);
    let before = arena.allocated_bytes();
    let alias = site(file, source, "item in items");
    let expected = site(file, source, "item'/>").span();
    for _ in 0..32 {
        let current = site(file, source, "item in items");
        assert!(current.symbol().same_symbol(alias.symbol()));
        let mut visits = 0;
        file.for_each_template_reference_to(current.symbol(), |site| {
            assert_eq!(site.span(), expected);
            assert!(core::ptr::eq(site.file(), file));
            visits += 1;
        })
        .unwrap();
        assert_eq!(visits, 1);
    }
    assert_eq!(arena.allocated_bytes(), before);
    assert!(matches!(
        file.template_symbol_at_offset(u32::MAX),
        Err(TemplateQueryError::Position(
            PositionQueryError::OutOfBounds
        ))
    ));
    assert!(
        file.template_symbol_at_offset(at(source, "items=2"))
            .unwrap()
            .is_none()
    );
    assert!(
        file.binding_at_offset(at(source, "items=2"))
            .unwrap()
            .is_some()
    );
}

#[test]
fn query_order_and_visitor_unwind_do_not_mutate_original_attachment_or_identity() {
    extern crate std;
    let arena = Allocator::default();
    let source = "<template><button @click='let value=$event; value' @blur='let value=$event; value'/></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let left = site(file, source, "value=$event; value' @blur");
    let right = site(file, source, "value=$event; value'/>");
    let expected_left = uses(file, left.symbol());
    let expected_right = uses(file, right.symbol());
    assert!(!left.symbol().same_symbol(right.symbol()));
    for symbol in [right.symbol(), left.symbol(), left.symbol(), right.symbol()] {
        assert_eq!(
            uses(file, symbol),
            if symbol.same_symbol(left.symbol()) {
                expected_left.clone()
            } else {
                expected_right.clone()
            }
        );
    }
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = file.for_each_template_reference_to(left.symbol(), |_| {
                std::panic::resume_unwind(alloc::boxed::Box::new("original query visitor"));
            });
        }))
        .is_err()
    );
    assert_eq!(uses(file, left.symbol()), expected_left);
    assert_eq!(uses(file, right.symbol()), expected_right);
    assert!(file.is_complete());
}

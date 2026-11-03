use super::*;
use vize_l2::{file::Namespace, op::BindingOp, resolution::HandlerBindingRef};
use vize_l3::decision::DecisionBuildError;

#[test]
fn nested_same_name_collection_borrows_parent_alias_before_child_shadowing() {
    let arena = Allocator::default();
    let owner = completed(
        &arena,
        "<script setup>const items=2;</script><template><div v-for='item in items'><span @click='const e=item;' v-for='item in item'/></div></template>",
    );
    let outer = first(&owner);
    let [Op::Element(div)] = outer.region.ops.as_slice() else {
        panic!("div")
    };
    let [Op::OriginalFor(inner)] = div.children.ops.as_slice() else {
        panic!("inner")
    };
    let [Op::Element(span)] = inner.region.ops.as_slice() else {
        panic!("span")
    };
    let [BindingOp::On(on)] = span.bindings.as_slice() else {
        panic!("earlier original event")
    };
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    let outer_row = analysis.for_head(outer.id().node()).unwrap();
    let inner_row = analysis.for_head(inner.id().node()).unwrap();
    let outer_alias = outer_row.head().value().unwrap();
    let inner_alias = inner_row.head().value().unwrap();
    assert_ne!(outer_alias.id(), inner_alias.id());
    assert_eq!(inner_row.collection().id(), outer_alias.id());
    assert!(inner_row.collection().declaration().is_none());
    let collection = inner_row.collection().template_declaration().unwrap();
    assert_eq!(collection.declaration().origin(), outer.id());
    assert_eq!(
        Some(collection.declaration().scope()),
        outer_row.head().scope()
    );
    assert_eq!(inner_row.head().enclosing_scope(), outer_row.head().scope());
    assert_eq!(
        analysis
            .file()
            .lookup(inner_row.head().scope().unwrap(), "item", Namespace::Value)
            .unwrap()
            .id(),
        inner_alias.id()
    );
    let handler = analysis.file().handler_for(on).unwrap();
    let event_row = analysis.handler(handler.id().node()).unwrap();
    assert_eq!(event_row.handler().scope(), inner_row.head().scope());
    assert_eq!(
        event_row.resolution().references()[0].binding,
        HandlerBindingRef::Outer(inner_alias.id())
    );
    let refusals = analysis.dom().unwrap().unsupported();
    assert_eq!(refusals.len(), 4);
    assert_eq!(refusals[0].reason, DomUnsupported::Operation);
    assert_eq!(refusals[1].reason, DomUnsupported::ForCollectionAccess);
    assert_eq!(
        refusals[1].span,
        inner_row.resolution().collection_authored_span()
    );
    assert_eq!(refusals[2].reason, DomUnsupported::Operation);
    assert_eq!(refusals[3].reason, DomUnsupported::HandlerAccess);
    assert_eq!(refusals[3].node, handler.id().node());
    assert_eq!(analysis.tables().nodes.len(), 5);
    assert_eq!(analysis.tables().controls.len(), 2);
    assert_eq!(
        analysis
            .tables()
            .controls
            .get(inner.id().node())
            .unwrap()
            .parent,
        Some(outer.id().node())
    );
}

#[test]
fn moved_owner_and_many_genuine_siblings_keep_exact_head_and_parameter_storage() {
    let arena = Allocator::default();
    let source = format!(
        "<script setup>const items=2;</script><template>{}</template>",
        "<div v-for='item in items'/>".repeat(64)
    );
    let owner = completed(&arena, &source);
    let pointers: Vec<_> = owner
        .file()
        .unwrap()
        .artifact()
        .root()
        .ops
        .iter()
        .map(|op| {
            let Op::OriginalFor(original) = op else {
                panic!("For")
            };
            let resolution = owner
                .file()
                .unwrap()
                .for_head_for(original)
                .unwrap()
                .resolution()
                .unwrap();
            (
                &**original as *const _,
                resolution.input().aliases().as_ptr(),
                resolution.input().collection() as *const _,
            )
        })
        .collect();
    let owner = core::hint::black_box(owner);
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    for (op, pointers) in analysis.artifact().root().ops.iter().zip(pointers) {
        let Op::OriginalFor(original) = op else {
            panic!("For")
        };
        let row = analysis.for_head(original.id().node()).unwrap();
        assert!(core::ptr::eq(row.original(), pointers.0));
        assert!(core::ptr::eq(
            row.resolution().input().aliases().as_ptr(),
            pointers.1
        ));
        assert!(core::ptr::eq(
            row.resolution().input().collection(),
            pointers.2
        ));
    }
    assert_eq!(analysis.tables().nodes.len(), 128);
    assert_eq!(analysis.tables().controls.len(), 64);
    assert_eq!(analysis.dom().unwrap().unsupported().len(), 64);
}

#[test]
fn forgotten_dropped_and_unwound_root_with_attached_for_never_returns_analysis() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'/>tail</template>";
    for mode in 0..3 {
        let mut owner = prepared(&arena, source);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut walk = owner.begin().unwrap();
            walk.child(walk.selected().children().next().unwrap())
                .unwrap();
            match mode {
                0 => std::mem::forget(walk),
                1 => drop(walk),
                _ => std::panic::resume_unwind(Box::new("after genuine For child")),
            }
        }));
        assert_eq!(result.is_err(), mode == 2);
        let owner = owner.finish();
        assert!(owner.view().is_err());
        let file = owner.file().unwrap();
        assert_eq!(file.artifact().node_count(), 2);
        assert_eq!(file.template_declarations().count(), 1);
        let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            panic!("retained original")
        };
        assert!(file.for_head_for(original).is_some());
        assert!(matches!(
            build_dom_file_decisions(file),
            Err(DecisionBuildError::IncompleteFile)
        ));
    }
}

#[test]
fn later_header_hole_retains_the_whole_input_but_never_mints_a_receipt() {
    let arena = Allocator::default();
    let mut owner = prepared(
        &arena,
        "<script setup>const items=2;</script><template>prefix<div v-for='item in items' :id='unsupported'/></template>",
    );
    {
        let mut walk = owner.begin().unwrap();
        let mut children = walk.selected().children();
        walk.child(children.next().unwrap()).unwrap();
        assert!(walk.child(children.next().unwrap()).is_err());
    }
    let owner = owner.finish();
    assert!(owner.view().is_err());
    let file = owner.file().unwrap();
    assert_eq!(file.artifact().node_count(), 1);
    assert_eq!(
        file.unattached_for_heads()
            .next()
            .unwrap()
            .operand()
            .raw_value(),
        "item in items"
    );
    assert!(matches!(
        build_dom_file_decisions(file),
        Err(DecisionBuildError::IncompleteFile)
    ));
}

mod read;

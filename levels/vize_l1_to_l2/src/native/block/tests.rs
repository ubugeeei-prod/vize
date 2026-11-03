use super::NativeComponent;
use vize_l0::{Allocator, SourceRoot, Span, cstr};
use vize_l1::embed::Lang;
use vize_l2::artifact::Builder;
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, Op};

#[test]
fn whole_file_template_block_keeps_absolute_entities_and_the_same_once_ast() {
    let a = Allocator::default();
    let file = "<script>const 作者 = 1</script>\n<template><p :title=\"作者 &amp;&amp; value\">{{ 作者 }}</p></template>";
    let start = file.find("<p ").unwrap();
    let end = file.find("</template>").unwrap();
    let block = SourceRoot::new(file)
        .unwrap()
        .block(file.get(start..end).unwrap(), start as u32)
        .unwrap();
    let owner = NativeComponent::parse_in(&a, block).unwrap();
    let mut builder = Builder::new(&a, file).unwrap();
    let produced = owner.construct_in(&mut builder.region(), Lang::Js).unwrap();
    assert!(produced.is_supported());
    assert_eq!(produced.component.block(), block);
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 3);
    assert_eq!(artifact.source().as_ptr(), file.as_ptr());
    let Some(Op::Element(element)) = artifact.root().ops.first() else {
        panic!("element");
    };
    assert_eq!(element.span, Span::new(start as u32, end as u32));
    let BindingOp::Bind(bind) = element.bindings.first().unwrap() else {
        panic!("binding");
    };
    let Some(ExprRef::Js(binding)) = bind.value else {
        panic!("retained binding");
    };
    let Op::Interpolation(interpolation) = element.children.ops.first().unwrap() else {
        panic!("interpolation");
    };
    let ExprRef::Js(js) = interpolation.expression else {
        panic!("retained interpolation");
    };
    assert!(core::ptr::eq(
        binding.ast,
        produced
            .embeds
            .first()
            .unwrap()
            .syntax
            .expression()
            .unwrap()
    ));
    let [_, second, ..] = produced.embeds.as_slice() else {
        panic!("second retained expression");
    };
    assert!(core::ptr::eq(js.ast, second.syntax.expression().unwrap()));
    assert_eq!(binding.source, "作者 && value");
    assert!(binding.matches_authored_source(file));
    assert!(js.matches_authored_source(file));
    for record in artifact.provenance() {
        assert!(block.contains_block_span(record.span));
        assert_eq!(
            record.before.as_str(),
            file.get(record.span.start as usize..record.span.end as usize)
                .unwrap()
        );
    }
}

#[test]
fn equal_bytes_in_a_foreign_allocation_reject_before_mint_and_keep_the_owner() {
    let a = Allocator::default();
    let original = vize_l0::String::from("<p>{{ value }}</p>");
    let foreign = original.clone();
    let owner =
        NativeComponent::parse_in(&a, SourceRoot::new(&original).unwrap().whole_block()).unwrap();
    let first = owner.carrier().tree.children.first().unwrap() as *const _;
    let mut builder = Builder::new(&a, &foreign).unwrap();
    let rejected = owner
        .construct_in(&mut builder.region(), Lang::Js)
        .unwrap_err();
    assert_eq!(
        rejected.component.block().root_source().as_ptr(),
        original.as_ptr()
    );
    assert_eq!(
        rejected.component.carrier().tree.children.first().unwrap() as *const _,
        first
    );
    assert_eq!(builder.finish().unwrap().node_count(), 0);
    let mut actual = Builder::new(&a, &original).unwrap();
    let produced = rejected
        .component
        .construct_in(&mut actual.region(), Lang::Js)
        .unwrap();
    assert!(produced.is_supported());
    assert_eq!(actual.finish().unwrap().node_count(), 2);
}

#[test]
fn recursive_short_borrows_record_only_actual_expression_ids_and_keep_raw_output() {
    use super::observed::{Observed, Sites};
    let a = Allocator::default();
    let file = vize_l0::String::from(
        "<p title=\"作者\" :id=\"value\">x<Child :title=\"value + 1\">{{ value }}</Child><!--tail--></p>",
    );
    let raw = crate::native::lower_component_native(&a, &file, Lang::Js).unwrap();
    let owner =
        NativeComponent::parse_in(&a, SourceRoot::new(&file).unwrap().whole_block()).unwrap();
    let mut builder = Builder::new(&a, &file).unwrap();
    let mut sites = Sites {
        expressions: alloc::vec::Vec::new(),
        owners: alloc::vec::Vec::new(),
    };
    let produced = owner
        .construct_in(
            &mut Observed {
                region: &mut builder.region(),
                sites: &mut sites,
            },
            Lang::Js,
        )
        .unwrap();
    assert!(produced.is_supported());
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), raw.artifact.node_count());
    assert_eq!(artifact.provenance(), raw.artifact.provenance());
    assert_eq!(
        cstr!("{:?}", artifact.root()),
        cstr!("{:?}", raw.artifact.root())
    );
    assert_eq!(
        sites
            .owners
            .iter()
            .map(|node| node.index())
            .collect::<alloc::vec::Vec<_>>(),
        [0, 3]
    );
    assert_eq!(
        sites
            .expressions
            .iter()
            .map(|(node, _)| node.index())
            .collect::<alloc::vec::Vec<_>>(),
        [1, 4, 5]
    );
    for ((node, js), embed) in sites.expressions.iter().zip(&produced.embeds) {
        assert_eq!(embed.node, Some(*node));
        assert!(core::ptr::eq(js.ast, embed.syntax.expression().unwrap()));
        assert!(js.matches_authored_source(&file));
    }
    drop(produced);
    // These observations and the actual arena tree remain valid after ordinary
    // owning L1 diagnostics/comment observations have been dropped.
    assert_eq!(sites.expressions.first().unwrap().1.source, "value");
    assert_eq!(artifact.node_count(), 7);
}

#[test]
fn block_rebases_all_error_and_admission_facts_without_modifying_original_carrier() {
    use crate::native::NativeHoleKind;
    let a = Allocator::default();
    let opens: vize_l0::String = (0..64)
        .map(|n| if n % 2 == 0 { '(' } else { '[' })
        .collect();
    let closes: vize_l0::String = opens
        .chars()
        .rev()
        .map(|ch| if ch == '(' { ')' } else { ']' })
        .collect();
    let content = cstr!("<!DOCTYPE html><div v-pre:[{opens}key{closes}]>{{{{ kept }}}}</div>");
    let prefix = "<script>const 作者 = 1</script>\n<template>";
    let file = cstr!("{prefix}{content}</template>");
    let block = SourceRoot::new(&file)
        .unwrap()
        .block(
            file.get(prefix.len()..prefix.len() + content.len())
                .unwrap(),
            prefix.len() as u32,
        )
        .unwrap();
    let owner = NativeComponent::parse_in(&a, block).unwrap();
    assert!(!owner.carrier().errors.is_empty());
    assert!(!owner.carrier().unsupported.is_empty());
    let mut builder = Builder::new(&a, &file).unwrap();
    let produced = owner.construct_in(&mut builder.region(), Lang::Js).unwrap();
    assert!(!produced.is_supported());
    for error in &produced.component.carrier().errors {
        let at = block.start() + error.offset;
        assert!(
            produced
                .holes
                .iter()
                .any(|hole| hole.kind == NativeHoleKind::Surface(error.code)
                    && hole.span == Span::new(at, at))
        );
    }
    for admission in &produced.component.carrier().unsupported {
        let span = Span::new(
            block.start() + admission.span.start,
            block.start() + admission.span.end,
        );
        assert!(produced.holes.iter().any(|hole| hole.kind
            == NativeHoleKind::DirectiveAdmission(admission.error)
            && hole.span == span));
    }
    for diagnostic in &produced.diagnostics {
        assert!(block.contains_block_span(diagnostic.span));
    }
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.source().as_ptr(), file.as_ptr());
    assert_eq!(artifact.node_count(), 2);
    assert_eq!(produced.embeds.len(), 1);
}

#[test]
fn same_root_pointer_with_a_different_length_cannot_enter_the_factory() {
    let a = Allocator::default();
    let file = "<p>{{value}}</p>tail";
    let owner =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let prefix = file.strip_suffix("tail").unwrap();
    assert_eq!(prefix.as_ptr(), file.as_ptr());
    let mut builder = Builder::new(&a, prefix).unwrap();
    let rejected = owner
        .construct_in(&mut builder.region(), Lang::Js)
        .unwrap_err();
    assert_eq!(rejected.component.block().source(), file);
    assert_eq!(builder.finish().unwrap().node_count(), 0);
}

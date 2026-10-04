use super::*;
use vize_l0::{Span, Vec};
use vize_l2::{
    artifact::ComponentFactory,
    file::{Declaration, TemplatePolicy, TemplateScope},
    lang::js::FileProducer,
    op::{Attribute, Namespace},
};

#[test]
fn foreign_element_sibling_slot_node_duplicate_and_out_of_order_rows_stickily_refuse() -> Test {
    let arena = Allocator::default();
    let source =
        "<template><div title='&amp;lt;' data-x='same'/><div title='&amp;lt;'/></template>";
    let original = completed(&arena, source)?;
    let foreign = completed(&arena, source)?;
    let file = original.file().ok_or("File")?;
    let other = foreign.file().ok_or("foreign File")?;
    let actual = element(file, 0)?;
    let sibling = element(file, 1)?;
    let first = node(file, 0)?;
    let wrong_node = node(file, 2)?;
    for (actual_node, supplied, slot) in [
        (first, element(other, 0)?, 0),
        (first, sibling, 0),
        (first, actual, 1),
        (wrong_node, actual, 0),
        (first, actual, usize::MAX),
    ] {
        let mut cursor = OriginalAttributeCursor::new(file, file.artifact()).map_err(|_| "new")?;
        cursor.observe(actual_node, supplied, slot);
        check(cursor.failure.is_some() && cursor.next == 0)?;
        cursor.observe(first, actual, 0);
        check(cursor.next == 0 && cursor.finish().is_err())?;
    }
    let mut cursor = OriginalAttributeCursor::new(file, file.artifact()).map_err(|_| "new")?;
    cursor.observe(first, actual, 0);
    check(cursor.next == 1)?;
    cursor.observe(first, actual, 0);
    check(cursor.next == 1 && cursor.finish().is_err())?;
    let mut cursor = OriginalAttributeCursor::new(file, file.artifact()).map_err(|_| "new")?;
    cursor.observe(first, actual, 0);
    check(cursor.finish().is_err())?;
    check(OriginalAttributeCursor::new(file, other.artifact()).is_err())?;
    // All refused experiments retain the original full map and normal rows.
    check(file.native_attribute_values().len() == 3)?;
    check(
        file.native_attribute_values()[0]
            .observation()
            .ok_or("source")?
            .source()
            .text()
            == "&lt;",
    )?;
    Ok(())
}

#[test]
fn complete_neutral_value_without_any_original_row_cannot_pass_native_consumption() -> Test {
    struct Empty;
    impl<'a> vize_l2::artifact::ComponentBody<'a> for Empty {
        fn run<R: vize_l2::artifact::ComponentFactory<'a>>(
            self,
            _: &mut R,
            _: vize_l0::id::NodeId,
        ) {
        }
    }
    #[derive(Clone, Copy)]
    struct Hidden;
    impl TemplatePolicy for Hidden {
        fn visible(self, _: &Declaration) -> bool {
            false
        }
    }
    let arena = Allocator::default();
    let source = "<div title='&amp;lt;'></div>";
    let mut producer = FileProducer::new(&arena, source).map_err(|_| "producer")?;
    {
        let mut region = producer
            .template_region(TemplateScope::Root, Hidden)
            .map_err(|_| "region")?;
        let mut attributes = Vec::new_in(&&arena);
        attributes.push(Attribute {
            name: "title",
            value: Some("&lt;"),
            span: Span::new(5, 21),
        });
        region
            .element(
                "div",
                Namespace::Html,
                attributes,
                Span::new(0, source.len() as u32),
                Empty,
            )
            .map_err(|_| "neutral Element")?;
    }
    let file = producer.finish().map_err(|_| "neutral File")?;
    check(file.is_complete() && file.native_attribute_values().is_empty())?;
    let result = crate::decision::build::build_with_original(
        &file,
        TargetPolicy::Dom,
        &crate::decision::dom::LiteralExpressions,
        &crate::decision::dom::vue::policy::NoReads,
    );
    check(matches!(
        result,
        Err(DecisionBuildError::OriginalAttributeValue {
            node: Some(_),
            slot: 0
        })
    ))?;
    // Ordinary diagnostic analysis retains its existing output semantics.
    let ordinary = crate::decision::build_dom_file_decisions(&file).map_err(|_| "ordinary")?;
    check(ordinary.original_attributes().is_none())?;
    Ok(())
}

#[test]
fn attached_prefix_on_incomplete_original_file_never_seals_whole_target_facts() -> Test {
    let arena = Allocator::default();
    let source = "<template><div title='&amp;lt;'/><div :id='1'/></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let mut children = walk.selected().children();
        walk.child(children.next().ok_or("first")?)
            .map_err(|_| "normal prefix")?;
        check(walk.child(children.next().ok_or("late")?).is_err())?;
    }
    let original = original.finish();
    check(original.view().is_err())?;
    let file = original.file().ok_or("retained original prefix")?;
    check(!file.is_complete())?;
    let actual = element(file, 0)?;
    check(file.native_attribute_value_for(0, actual, 0).is_some())?;
    check(matches!(
        OriginalAttributeCursor::new(file, file.artifact()),
        Err(DecisionBuildError::IncompleteFile)
    ))?;
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        let result = crate::decision::build::build_with_original(
            file,
            policy,
            &crate::decision::dom::LiteralExpressions,
            &crate::decision::dom::vue::policy::NoReads,
        );
        check(matches!(result, Err(DecisionBuildError::IncompleteFile)))?;
    }
    Ok(())
}

#[test]
fn original_class_entity_text_and_unsupported_target_names_remain_precise_boundaries() -> Test {
    for template in [
        r#"<div class="a&amp;amp;b">x</div>"#,
        r#"<div title="a &amp;lt; b">&amp;lt;</div>"#,
    ] {
        let arena = Allocator::default();
        let source = alloc::format!("<template>{template}</template>");
        check(completed(&arena, &source).is_err())?;
    }
    let arena = Allocator::default();
    let original = completed(&arena, "<template><div hidden='&amp;lt;'/></template>")?;
    let analysis = vapor::build_native_vapor_file_decisions(original.view().map_err(|_| "view")?)
        .map_err(|_| "authentic native rows")?;
    check(
        analysis
            .original_attributes()
            .ok_or("full original row")?
            .len()
            == 1,
    )?;
    check(
        analysis
            .vapor()
            .ok_or("Vapor facts")?
            .unsupported()
            .iter()
            .any(|hole| hole.reason == vapor::VaporUnsupported::AttributeSemantics),
    )?;
    Ok(())
}

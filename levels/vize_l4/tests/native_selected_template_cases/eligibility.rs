use crate::{check, completed, equal};
use vize_l0::{Allocator, Span, id::NodeId};
use vize_l2::op::{Namespace, Op};
use vize_l3::decision::{dom::DomUnsupported, native::build_native_dom_file_decisions};
use vize_l4::{
    targets::dom::{DomErrorKind, emit_template},
    write::{NoLinks, Recorded},
};

#[test]
fn target_role_refusal_preserves_complete_original_root_and_nested_search_owners()
-> Result<(), &'static str> {
    for (source, nested, index, nodes, span, attribute_span) in [
        (
            "<template><search id=\"kept\"/></template>",
            false,
            0,
            1,
            Span::new(10, 29),
            Span::new(18, 27),
        ),
        (
            "<template><div><search id='kept'/></div></template>",
            true,
            1,
            2,
            Span::new(15, 34),
            Span::new(23, 32),
        ),
    ] {
        let arena = Allocator::default();
        let original = completed(&arena, source)?;
        let file = original.file().ok_or("complete original File")?;
        check(file.is_complete())?;
        let view = original.view().map_err(|_| "normal original completion")?;
        check(core::ptr::eq(view.owner(), &original))?;
        check(core::ptr::eq(view.file().ok_or("view File")?, file))?;
        let analysis =
            build_native_dom_file_decisions(view).map_err(|_| "original DOM analysis")?;
        check(core::ptr::eq(analysis.owner(), &original))?;
        check(core::ptr::eq(analysis.file(), file))?;
        check(core::ptr::eq(analysis.artifact(), file.artifact()))?;
        check(core::ptr::eq(analysis.artifact().source(), source))?;
        equal(analysis.artifact().node_count(), nodes)?;
        equal(analysis.tables().nodes.len(), nodes as usize)?;
        check(analysis.tables().controls.is_empty())?;
        let [root_op] = file.artifact().root().ops.as_slice() else {
            return Err("actual original root");
        };
        let Op::Element(root_element) = root_op else {
            return Err("actual original root Element");
        };
        let canonical = if nested {
            equal(root_element.tag, "div")?;
            check(root_element.attributes.is_empty())?;
            let [search] = root_element.children.ops.as_slice() else {
                return Err("only actual nested search");
            };
            search
        } else {
            root_op
        };
        let Op::Element(search) = canonical else {
            return Err("retained canonical search Element");
        };
        equal(search.tag, "search")?;
        equal(search.namespace, Namespace::Html)?;
        equal(search.span, span)?;
        check(search.bindings.is_empty() && search.children.ops.is_empty())?;
        let [attribute] = search.attributes.as_slice() else {
            return Err("retained canonical header");
        };
        equal(
            (attribute.name, attribute.value, attribute.span),
            ("id", Some("kept"), attribute_span),
        )?;
        let root = original
            .selected()
            .children()
            .next()
            .ok_or("original root child")?
            .into_element()
            .ok_or("original root Element")?;
        let parent = if nested { Some(root.surface()) } else { None };
        let search = if nested {
            root.children()
                .next()
                .ok_or("original nested search")?
                .into_element()
                .ok_or("original nested Element")?
        } else {
            root
        };
        equal(search.ordinal(), 0)?;
        match (parent, search.parent_element()) {
            (None, None) => {}
            (Some(expected), Some(actual)) => check(core::ptr::eq(actual, expected))?,
            _ => return Err("actual original parent membership"),
        }
        check(core::ptr::eq(
            search.component(),
            original.selected().component(),
        ))?;
        let header = search.attributes().next().ok_or("full original header")?;
        equal(search.attributes().len(), 1)?;
        equal(header.ordinal(), 0)?;
        check(core::ptr::eq(header.component(), search.component()))?;
        check(core::ptr::eq(header.element(), search.surface()))?;
        check(core::ptr::eq(
            header.surface(),
            search
                .surface()
                .open
                .attrs
                .first()
                .ok_or("complete original Attribute")?,
        ))?;
        check(core::ptr::eq(attribute.name, header.surface().name.text))?;
        check(core::ptr::eq(
            attribute.value.ok_or("canonical value")?,
            header
                .surface()
                .value
                .as_ref()
                .ok_or("original value")?
                .content
                .text,
        ))?;
        let node = NodeId::from_index(index).ok_or("actual search node")?;
        let dom = analysis.dom().ok_or("retained DOM facts")?;
        check(core::ptr::eq(
            dom.node(node).ok_or("actual DOM node")?.op(),
            canonical,
        ))?;
        check(dom.binding(node).is_none() && analysis.expression(node).is_none())?;
        let [rejection] = dom.unsupported() else {
            return Err("only actual target role refusal");
        };
        equal(
            (rejection.node, rejection.span, rejection.reason),
            (node, span, DomUnsupported::ElementRole),
        )?;
        let recorded = emit_template::<Recorded>(&analysis)
            .err()
            .ok_or("no recording Writer on refusal")?;
        let plain = emit_template::<NoLinks>(&analysis)
            .err()
            .ok_or("no plain Writer on refusal")?;
        equal(recorded, plain)?;
        equal(recorded.node, Some(node))?;
        equal(recorded.span, span)?;
        equal(
            recorded.kind,
            DomErrorKind::Unsupported(DomUnsupported::ElementRole),
        )?;
        check(file.is_complete() && original.view().is_ok())?;
    }
    Ok(())
}

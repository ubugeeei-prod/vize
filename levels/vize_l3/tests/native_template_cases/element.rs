use crate::{Allocator, NodeId, Op, StaticLevel, TargetPolicy, build_native_dom_file_decisions};
use crate::{check, completed, owner};
use vize_l0::Span;
use vize_l2::{lang::js::NativeTemplateIssueKind, op::Namespace, walk::NodeRef};
use vize_l3::decision::dom::{
    DomChanges, DomChild, DomChildren, DomDependency, DomRootKind, DomText,
};

const SOURCE: &str = "<template><section><span>hé</span><!--note--><br/></section></template>";

fn id(index: u32) -> Result<NodeId, &'static str> {
    NodeId::from_index(index).ok_or("node index")
}

#[test]
fn selected_nested_html_retains_exact_ops_spans_decisions_and_dependency_order()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let output = completed(&arena, SOURCE)?;
    let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
        .map_err(|_| "analysis")?;
    let file = output.file().ok_or("original File")?;
    check(core::ptr::eq(analysis.owner(), &output))?;
    check(core::ptr::eq(analysis.file(), file))?;
    check(core::ptr::eq(analysis.artifact(), file.artifact()))?;
    check(core::ptr::eq(analysis.artifact().source(), SOURCE))?;
    check(file.is_complete())?;
    check(analysis.policy() == TargetPolicy::Dom)?;
    check(analysis.artifact().node_count() == 5)?;
    check(analysis.tables().nodes.len() == 5)?;
    check(analysis.tables().controls.is_empty())?;
    let [original_section] = analysis.artifact().root().ops.as_slice() else {
        return Err("single original Element root");
    };
    let Op::Element(section) = original_section else {
        return Err("original section");
    };
    let [original_span, original_comment, original_br] = section.children.ops.as_slice() else {
        return Err("original ordered section children");
    };
    let (Op::Element(span), Op::Comment(comment), Op::Element(br)) =
        (original_span, original_comment, original_br)
    else {
        return Err("original Element Comment Element order");
    };
    let [original_text] = span.children.ops.as_slice() else {
        return Err("single original text child");
    };
    let Op::Text(text) = original_text else {
        return Err("original UTF-8 text");
    };
    check(section.tag == "section" && span.tag == "span" && br.tag == "br")?;
    check(text.content == "hé" && comment.content == "note")?;
    check(core::ptr::eq(
        text.content,
        SOURCE.get(25..28).ok_or("text source")?,
    ))?;
    check(core::ptr::eq(
        comment.content,
        SOURCE.get(39..43).ok_or("comment body source")?,
    ))?;
    for element in [section, span, br] {
        check(element.namespace == Namespace::Html)?;
        check(element.attributes.is_empty() && element.bindings.is_empty())?;
    }
    check(br.children.ops.is_empty())?;
    let dom = analysis.dom().ok_or("DOM facts")?;
    check(dom.unsupported().is_empty())?;
    let expected = [
        (original_section, Span::new(10, 61), StaticLevel::Dynamic),
        (original_span, Span::new(19, 35), StaticLevel::Static),
        (original_text, Span::new(25, 28), StaticLevel::Static),
        (original_comment, Span::new(35, 46), StaticLevel::Dynamic),
        (original_br, Span::new(46, 51), StaticLevel::Static),
    ];
    for (index, (original_op, authored_span, level)) in (0..5).zip(expected) {
        let node = id(index)?;
        let facts = dom.node(node).ok_or("exact original node")?;
        check(core::ptr::eq(facts.op(), original_op))?;
        check(NodeRef::Op(facts.op()).span() == authored_span)?;
        check(facts.block_eligible == (index == 0))?;
        check(facts.changes == DomChanges::default())?;
        check(facts.dynamic_property_bindings.is_empty())?;
        check(dom.binding(node).is_none() && dom.conditional(node).is_none())?;
        check(analysis.expression(node).is_none())?;
        let decision = analysis
            .tables()
            .nodes
            .get(node)
            .ok_or("exact decision row")?;
        check(decision.static_level == level && decision.output_level == level)?;
        check(decision.dynamic_bindings.is_empty() && decision.control.is_none())?;
    }
    check(dom.node(id(5)?).is_none())?;
    check(analysis.tables().nodes.get(id(5)?).is_none())?;
    check(dom.root().kind == DomRootKind::Direct)?;
    check(dom.root().children == vec![DomChild::Node(id(0)?)])?;
    check(
        dom.node(id(0)?).ok_or("section facts")?.children
            == DomChildren::Array(vec![
                DomChild::Node(id(1)?),
                DomChild::Node(id(3)?),
                DomChild::Node(id(4)?),
            ]),
    )?;
    check(
        dom.node(id(1)?).ok_or("span facts")?.children
            == DomChildren::Text(DomText {
                nodes: vec![id(2)?],
                dynamic: false,
            }),
    )?;
    for index in [2, 3, 4] {
        check(dom.node(id(index)?).ok_or("leaf facts")?.children == DomChildren::Empty)?;
    }
    check(
        dom.dependencies()
            == [
                DomDependency::NativeElementValue,
                DomDependency::CommentValue,
                DomDependency::BlockBoundary,
                DomDependency::NativeElementBlock,
            ],
    )?;
    Ok(())
}

#[test]
fn refused_nested_html_retains_its_prefix_without_native_analysis() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for body in ["bad  text", "&amp;", "<svg/>"] {
        let source = format!("<template><section>ok<span>{body}</span></section>tail</template>");
        let mut original = owner(&arena, &source)?;
        let refusal;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let mut children = walk.selected().children();
            refusal = walk
                .child(children.next().ok_or("section")?)
                .err()
                .ok_or("nested refusal")?;
            check(refusal.kind == NativeTemplateIssueKind::UnsupportedChild)?;
            check(walk.child(children.next().ok_or("tail")?).err() == Some(refusal))?;
            check(walk.complete().err() == Some(refusal))?;
        }
        let output = original.finish();
        let denied = output
            .view()
            .map(build_native_dom_file_decisions)
            .err()
            .ok_or("no native analysis of partial prefix")?;
        check(denied == refusal)?;
        check(output.selected().children().len() == 2)?;
        let file = output.file().ok_or("retained partial File")?;
        check(!file.is_complete())?;
        check(file.artifact().node_count() == 3)?;
        check(file.artifact().source() == source)?;
        let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
            return Err("only original partial section root");
        };
        let [Op::Text(text), Op::Element(span)] = section.children.ops.as_slice() else {
            return Err("actual constructed nested prefix");
        };
        check(section.tag == "section" && text.content == "ok" && span.tag == "span")?;
        check(span.children.ops.is_empty())?;
        check(file.template_interruption().is_some())?;
    }
    Ok(())
}

#[test]
fn consuming_the_whole_original_element_without_root_end_cannot_reach_analysis()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let mut original = owner(&arena, SOURCE)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = walk.selected().children().next().ok_or("section")?;
        check(walk.child(child).map_err(|_| "whole Element")? == NodeId::FIRST)?;
    }
    let output = original.finish();
    let denied = output
        .view()
        .map(build_native_dom_file_decisions)
        .err()
        .ok_or("no native analysis without original root completion")?;
    check(denied.kind == NativeTemplateIssueKind::Interrupted)?;
    check(output.selected().children().len() == 1)?;
    let file = output.file().ok_or("retained partial File")?;
    check(!file.is_complete())?;
    check(file.artifact().node_count() == 5)?;
    check(file.template_interruption().is_some())?;
    let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
        return Err("retained whole Element root");
    };
    check(section.tag == "section")?;
    check(section.span == Span::new(10, 61))?;
    let [Op::Element(span), Op::Comment(comment), Op::Element(br)] =
        section.children.ops.as_slice()
    else {
        return Err("retained actual whole body");
    };
    let [Op::Text(text)] = span.children.ops.as_slice() else {
        return Err("retained original nested text");
    };
    check(text.content == "hé" && comment.content == "note" && br.tag == "br")?;
    Ok(())
}

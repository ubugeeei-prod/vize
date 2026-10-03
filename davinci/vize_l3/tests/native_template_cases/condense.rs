use crate::{Allocator, NodeId, Op, build_native_dom_file_decisions, check, owner};
use serde_json::{Value, json};
use vize_l1::SurfaceChild;
use vize_l3::decision::dom::DomChild;

#[test]
fn all_twenty_four_pinned_static_root_packets_retain_complete_file_and_l3_order()
-> Result<(), &'static str> {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../vize_l1/tests/fixtures/native-root-text-vue-3.5.35.json"
    ))
    .map_err(|_| "pinned upstream packet")?;
    check(packet["vueVersion"] == "3.5.35")?;
    let mut executed = 0;
    for case in packet["cases"].as_array().ok_or("complete cases")? {
        let slots = case["rootSlots"].as_array().ok_or("original slots")?;
        if case["nativeEligible"] != true
            || slots.iter().any(|slot| slot["kind"] == "interpolation")
        {
            continue;
        }
        executed += 1;
        let source = case["source"].as_str().ok_or("whole original source")?;
        let texts = case["rootText"]
            .as_array()
            .ok_or("complete original text facts")?;
        let expected = expected_ops(slots, texts)?;
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "actual lower start")?;
            let selected = walk.selected();
            for child in selected.children() {
                if matches!(child.surface(), SurfaceChild::Text(_)) {
                    let receipt = selected
                        .prepare_condensed_root_text(child)
                        .map_err(|_| "genuine original root receipt")?;
                    walk.root_text(&receipt).map_err(|_| "actual cursor text")?;
                } else {
                    walk.child(child).map_err(|_| "original nontext")?;
                }
            }
            walk.complete().map_err(|_| "complete original cursor")?;
        }
        let output = core::hint::black_box(original.finish());
        let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
            .map_err(|_| "genuine owner analysis")?;
        check(core::ptr::eq(analysis.owner(), &output))?;
        check(core::ptr::eq(
            analysis.file(),
            output.file().ok_or("sole File")?,
        ))?;
        check(core::ptr::eq(
            analysis.artifact(),
            analysis.file().artifact(),
        ))?;
        check(core::ptr::eq(analysis.artifact().source(), source))?;
        check(analysis.file().is_complete())?;
        let actual = analysis
            .artifact()
            .root()
            .ops
            .iter()
            .map(actual_op)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            actual, expected,
            "{}: complete original operation packet",
            case["id"]
        );
        assert_eq!(analysis.artifact().node_count() as usize, expected.len());
        assert_eq!(analysis.tables().nodes.len(), expected.len());
        check(analysis.tables().controls.is_empty())?;
        let dom = analysis.dom().ok_or("whole DOM facts")?;
        check(dom.unsupported().is_empty())?;
        let actual_children = dom
            .root()
            .children
            .iter()
            .map(|child| match child {
                DomChild::Node(node) => json!({"kind":"node","nodes":[node.index()]}),
                DomChild::Text(text) => json!({
                    "kind":"text",
                    "nodes":text.nodes.iter().map(|node| node.index()).collect::<Vec<_>>(),
                    "dynamic":text.dynamic
                }),
            })
            .collect::<Vec<_>>();
        let expected_children = expected
            .iter()
            .enumerate()
            .map(|(index, op)| {
                if op["kind"] == "text" {
                    json!({"kind":"text","nodes":[index],"dynamic":false})
                } else {
                    json!({"kind":"node","nodes":[index]})
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual_children, expected_children,
            "{}: all root groups",
            case["id"]
        );
        for (index, op) in analysis.artifact().root().ops.iter().enumerate() {
            let node = NodeId::from_index(u32::try_from(index).map_err(|_| "index")?)
                .ok_or("original node")?;
            check(core::ptr::eq(dom.node(node).ok_or("DOM node")?.op(), op))?;
            check(analysis.expression(node).is_none())?;
        }
        assert_eq!(output.selected().children().len(), slots.len());
    }
    assert_eq!(executed, 24, "every bounded static original root packet");
    Ok(())
}

fn expected_ops(slots: &[Value], texts: &[Value]) -> Result<Vec<Value>, &'static str> {
    let mut expected = Vec::new();
    for slot in slots {
        let kind = slot["kind"].as_str().ok_or("original slot kind")?;
        let raw = slot["raw"].as_str().ok_or("original complete raw window")?;
        let content = match kind {
            "text" => {
                let text = texts
                    .iter()
                    .find(|text| text["ordinal"] == slot["ordinal"])
                    .ok_or("independent original text fact")?;
                if text["content"].is_null() {
                    continue;
                }
                text["content"].as_str().ok_or("complete expected text")?
            }
            "comment" => raw
                .strip_prefix("<!--")
                .and_then(|raw| raw.strip_suffix("-->"))
                .ok_or("complete expected comment")?,
            "element" => match raw {
                "<i/>" => "i",
                "<b/>" => "b",
                _ => return Err("bounded independent element witness"),
            },
            _ => return Err("bounded independent root kind"),
        };
        expected.push(json!({"kind":kind,"content":content,"span":slot["span"]}));
    }
    Ok(expected)
}

fn actual_op(op: &Op<'_>) -> Result<Value, &'static str> {
    let (kind, content, span) = match op {
        Op::Text(text) => ("text", text.content, text.span),
        Op::Comment(comment) => ("comment", comment.content, comment.span),
        Op::Element(element) => {
            check(element.attributes.is_empty() && element.bindings.is_empty())?;
            check(element.children.ops.is_empty())?;
            ("element", element.tag, element.span)
        }
        _ => return Err("bounded complete actual op"),
    };
    Ok(json!({"kind":kind,"content":content,"span":[span.start,span.end]}))
}

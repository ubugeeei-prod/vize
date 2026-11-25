//! Exact original static class succeeds through the original owning Vapor path.
use serde_json::{Value, json};
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::AttributeValueOperand;
use vize_l2::file::L2Ir;
use vize_l2::{file::NativeFileAttributeValueState, lang::js::NativeTemplateFile, op::Op};
use vize_l3::decision::vapor::build_template_vapor_decisions;
use vize_l4::{
    targets::vapor::emit_template,
    write::{NoLinks, Recorded},
};

pub(crate) fn assert_current(owner: &NativeTemplateFile<'_>, source: &str) {
    assert_eq!(source, "<template><span class=\"x\">x</span></template>");
    let view = owner.view().unwrap();
    let file = view.file().unwrap();
    assert!(file.is_complete());
    let analysis = build_template_vapor_decisions(view).unwrap();
    assert!(core::ptr::eq(analysis.owner(), owner));
    assert!(core::ptr::eq(analysis.l2ir(), file));
    assert!(core::ptr::eq(analysis.artifact(), file.artifact()));
    assert!(core::ptr::eq(analysis.artifact().source(), source));
    assert_eq!(analysis.artifact().node_count(), 2);
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("complete original class root");
    };
    assert_eq!(element.tag, "span");
    assert_eq!(element.span, Span::new(10, 34));
    assert_eq!(element.attributes.len(), 1);
    let facts = analysis.original_attributes().unwrap();
    assert!(core::ptr::eq(facts.l2ir(), file));
    assert_eq!(facts.len(), 1);
    let [record] = file.native_attribute_values() else {
        panic!("one whole original class observation");
    };
    assert_eq!(
        record.state(),
        NativeFileAttributeValueState::Attached {
            node: NodeId::FIRST,
            slot: 0
        }
    );
    let joined = facts.value(0, element, 0).unwrap();
    assert!(core::ptr::eq(joined.l2ir(), file));
    let value = joined.observation().unwrap();
    assert!(core::ptr::eq(value, record.observation().unwrap()));
    let attribute = joined.attribute().unwrap();
    assert!(core::ptr::eq(attribute, &element.attributes[0]));
    assert_eq!(
        (attribute.name, attribute.value, attribute.span),
        ("class", Some("x"), Span::new(16, 25))
    );
    assert!(core::ptr::eq(
        attribute.value.unwrap(),
        value.source().text()
    ));
    assert!(core::ptr::eq(value.source().authored_root(), source));
    assert_eq!(value.name_span(), Span::new(16, 21));
    assert_eq!(value.equals_span(), Span::new(21, 22));
    assert_eq!(value.full_value_span(), Span::new(22, 25));
    assert_eq!(
        value.quote_spans(),
        Some((Span::new(22, 23), Span::new(24, 25)))
    );
    assert_eq!(value.value_span(), Span::new(23, 24));
    assert_eq!(value.source().span(), Span::new(23, 24));
    assert_eq!(value.raw_value(), "x");
    assert_eq!(value.source().text(), "x");
    assert!(value.source().decode_map().is_none());
    let vapor = analysis.vapor().unwrap();
    assert_eq!(vapor.roots().len(), 1);
    assert_eq!(vapor.roots()[0].node(), NodeId::FIRST);
    assert_eq!(vapor.roots()[0].span(), Span::new(10, 34));
    assert_eq!(vapor.parts(&vapor.roots()[0]).unwrap().len(), 3);
    assert_eq!(vapor.unsupported(), []);
    let recorded = emit_template::<Recorded>(&analysis).unwrap();
    let plain = emit_template::<NoLinks>(&analysis).unwrap();
    let expected = "import { template as _template } from \"vue\"\n\
const t0 = _template(\"<span class=\\\"x\\\">x</span>\", 3)\n\n\
export function render(_ctx) {\n  const n0 = t0()\n  return n0\n}";
    assert_eq!(recorded.text.as_str(), expected);
    assert_eq!(plain.text.as_str(), expected);
    assert_eq!(recorded.helpers, plain.helpers);
    assert!(plain.links == NoLinks);
    let document = recorded.into_document();
    let anchors = [
        (expected.find("<span").unwrap(), 10),
        (expected.find("class=").unwrap(), 16),
        (expected.find(">x</span>").unwrap() + 1, 26),
        (expected.find("</span>").unwrap(), 34),
    ];
    assert_eq!(document.links().len(), anchors.len());
    for (link, (generated, authored)) in document.links().iter().zip(anchors) {
        assert_eq!(
            link.generated,
            Span::new(generated as u32, generated as u32)
        );
        assert_eq!(link.authored, Span::new(authored, authored));
    }
    let map_text = document.source_map("NativeVapor.vue", source);
    let map: Value = serde_json::from_str(&map_text).unwrap();
    assert_eq!(map["sourcesContent"], json!([source]));
    if let Ok(path) = std::env::var("VIZE_NATIVE_VAPOR_CAPTURE") {
        std::fs::write(format!("{path}.class-successor.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "source":source,"file":file_observation(file,source),"tables":format!("{:#?}",analysis.tables()),
            "value":value_observation(value,source),"vapor":format!("{vapor:#?}"),
            "code":document.as_str(),"mapText":map_text,"map":map,
            "links":document.links().iter().map(|link|json!({"generated":span(link.generated),"authored":span(link.authored)})).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}

fn span(span: Span) -> serde_json::Value {
    serde_json::json!({"start":span.start,"end":span.end})
}
fn debug_rows<T: std::fmt::Debug>(rows: &[T]) -> Vec<std::string::String> {
    rows.iter().map(|row| format!("{row:#?}")).collect()
}
fn file_observation(l2ir: &L2Ir<'_>, source: &str) -> Value {
    let artifact = l2ir.artifact();
    let provenance: Vec<_> = artifact.provenance().iter().map(|record|json!({
        "rule":record.rule.as_str(),"node":record.node.map(|node|node.index()),
        "before":record.before.as_str(),"after":record.after.as_str(),"span":span(record.span)
    })).collect();
    let bindings: Vec<_> = l2ir.bindings().map(|binding|json!({
        "id":binding.id().index(),"declarationDebug":binding.declaration().map(|row|format!("{row:#?}"))
    })).collect();
    let attributes: Vec<_> = l2ir
        .native_attribute_values()
        .iter()
        .enumerate()
        .map(|(index, row)| {
            json!({
                "index":index,"state":format!("{:?}",row.state()),
                "observation":row.observation().map(|value|value_observation(value,source)),
                "failure":row.failure().map(|failure|format!("{failure:#?}"))
            })
        })
        .collect();
    let interpolations: Vec<_> = l2ir.native_interpolations().iter().map(|row|json!({
        "state":format!("{:?}",row.state()),"inputDebug":format!("NativeInterpolationInput {{ full_span: {:?}, content_span: {:?}, raw_content: {:?}, syntax: {:#?} }}",row.input().operand().full_span(),row.input().operand().content_span(),row.input().operand().raw_content(),row.input().operand().syntax())
    })).collect();
    let interpolation_failures: Vec<_> = l2ir
        .native_interpolation_failures()
        .iter()
        .map(|row| {
            json!({
                "span":span(row.span()),"failureDebug":format!("{:#?}",row.failure())
            })
        })
        .collect();
    json!({
        "source":artifact.source(),"sameSource":core::ptr::eq(artifact.source(),source),
        "complete":l2ir.is_complete(),"artifactDebug":format!("{artifact:#?}"),
        "nodeCount":artifact.node_count(),"rootDebug":format!("{:#?}",artifact.root()),
        "artifactScopesDebug":format!("{:#?}",artifact.scopes()),"provenance":provenance,
        "units":debug_rows(l2ir.units()),"scopes":debug_rows(l2ir.scopes()),"bindings":bindings,
        "references":debug_rows(l2ir.references()),"exports":debug_rows(l2ir.exports()),
        "imports":debug_rows(l2ir.imports()),"issues":debug_rows(l2ir.issues()),
        "templateIssues":debug_rows(l2ir.template_issues()),
        "diagnosticCounts":{"issues":l2ir.issues().len(),"templateIssues":l2ir.template_issues().len(),
            "templateInterruption":usize::from(l2ir.template_interruption().is_some()),
            "interruptedPrograms":l2ir.interrupted_programs().count(),
            "interpolationFailures":l2ir.native_interpolation_failures().len()},
        "templateInterruption":l2ir.template_interruption().map(|issue|format!("{issue:#?}")),
        "interruptedPrograms":l2ir.interrupted_programs().map(|issue|format!("{issue:#?}")).collect::<Vec<_>>(),
        "rejectedHandlers":debug_rows(l2ir.rejected_handlers()),
        "unattachedHandlers":l2ir.unattached_handlers().map(|input|format!("{input:#?}")).collect::<Vec<_>>(),
        "unattachedForHeads":l2ir.unattached_for_heads().map(|input|format!("{input:#?}")).collect::<Vec<_>>(),
        "rejectedForHeads":debug_rows(l2ir.rejected_for_heads()),"interpolations":interpolations,
        "interpolationFailures":interpolation_failures,"attributeValues":attributes
    })
}

fn value_observation(value: &AttributeValueOperand<'_>, original_source: &str) -> Value {
    let source = value.source();
    let decode_map = source.decode_map().map(|map| {
        map.segments()
            .iter()
            .map(|segment| {
                json!({
                    "decoded":span(segment.decoded()),"authored":span(segment.authored()),
                    "kind":format!("{:?}",segment.kind())
                })
            })
            .collect::<Vec<_>>()
    });
    json!({
        "raw":value.raw_value(),"decoded":source.text(),"nameSpan":span(value.name_span()),
        "valueSpan":span(value.value_span()),"fullValueSpan":span(value.full_value_span()),
        "equalsSpan":span(value.equals_span()),
        "quoteSpans":value.quote_spans().map(|(open,close)|[span(open),span(close)]),
        "authoredRoot":source.authored_root(),
        "authoredRootSame":core::ptr::eq(source.authored_root(),original_source),
        "sourceSpan":span(source.span()),"decodeMap":decode_map,"debugSource":format!("{source:#?}")
    })
}

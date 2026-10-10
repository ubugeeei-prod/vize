//! Exact original static class succeeds through L3 but remains refused by Vapor L4.
use serde_json::{Value, json};
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::NativeAttributeValue;
use vize_l2::file::FileArtifact;
use vize_l2::{file::NativeFileAttributeValueState, lang::js::NativeTemplateFile, op::Op};
use vize_l3::decision::vapor::{
    VaporRejection, VaporUnsupported, build_native_vapor_file_decisions,
};
use vize_l4::{
    targets::vapor::{VaporError, VaporErrorKind, emit_template},
    write::{NoLinks, Recorded},
};

pub(crate) fn assert_current(owner: &NativeTemplateFile<'_>, source: &str) {
    assert_eq!(source, "<template><span class=\"x\">x</span></template>");
    let view = owner.view().unwrap();
    let file = view.file().unwrap();
    assert!(file.is_complete());
    let analysis = build_native_vapor_file_decisions(view).unwrap();
    assert!(core::ptr::eq(analysis.owner(), owner));
    assert!(core::ptr::eq(analysis.file(), file));
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
    assert!(core::ptr::eq(facts.file(), file));
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
    assert!(core::ptr::eq(joined.file(), file));
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
    assert_eq!(
        vapor.unsupported(),
        [VaporRejection {
            node: NodeId::FIRST,
            span: Span::new(16, 25),
            reason: VaporUnsupported::AttributeSemantics
        }]
    );
    let error = VaporError {
        node: Some(NodeId::FIRST),
        span: Span::new(16, 25),
        kind: VaporErrorKind::Unsupported(VaporUnsupported::AttributeSemantics),
    };
    assert_eq!(emit_template::<Recorded>(&analysis).unwrap_err(), error);
    assert_eq!(emit_template::<NoLinks>(&analysis).unwrap_err(), error);
    if let Ok(path) = std::env::var("VIZE_NATIVE_VAPOR_CAPTURE") {
        std::fs::write(format!("{path}.class-successor.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "source":source,"file":file_observation(file,source),"tables":format!("{:#?}",analysis.tables()),
            "value":value_observation(value,source),"vapor":format!("{vapor:#?}"),"error":format!("{error:#?}")
        })).unwrap()).unwrap();
    }
}

fn span(span: Span) -> serde_json::Value {
    serde_json::json!({"start":span.start,"end":span.end})
}
fn debug_rows<T: std::fmt::Debug>(rows: &[T]) -> Vec<std::string::String> {
    rows.iter().map(|row| format!("{row:#?}")).collect()
}
fn file_observation(file: &FileArtifact<'_>, source: &str) -> Value {
    let artifact = file.artifact();
    let provenance: Vec<_> = artifact.provenance().iter().map(|record|json!({
        "rule":record.rule.as_str(),"node":record.node.map(|node|node.index()),
        "before":record.before.as_str(),"after":record.after.as_str(),"span":span(record.span)
    })).collect();
    let bindings: Vec<_> = file.bindings().map(|binding|json!({
        "id":binding.id().index(),"declarationDebug":binding.declaration().map(|row|format!("{row:#?}"))
    })).collect();
    let attributes: Vec<_> = file
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
    let interpolations: Vec<_> = file.native_interpolations().iter().map(|row|json!({
        "state":format!("{:?}",row.state()),"inputDebug":format!("NativeInterpolationInput {{ full_span: {:?}, content_span: {:?}, raw_content: {:?}, syntax: {:#?} }}",row.input().operand().full_span(),row.input().operand().content_span(),row.input().operand().raw_content(),row.input().operand().syntax())
    })).collect();
    let interpolation_failures: Vec<_> = file
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
        "complete":file.is_complete(),"artifactDebug":format!("{artifact:#?}"),
        "nodeCount":artifact.node_count(),"rootDebug":format!("{:#?}",artifact.root()),
        "artifactScopesDebug":format!("{:#?}",artifact.scopes()),"provenance":provenance,
        "units":debug_rows(file.units()),"scopes":debug_rows(file.scopes()),"bindings":bindings,
        "references":debug_rows(file.references()),"exports":debug_rows(file.exports()),
        "imports":debug_rows(file.imports()),"issues":debug_rows(file.issues()),
        "templateIssues":debug_rows(file.template_issues()),
        "diagnosticCounts":{"issues":file.issues().len(),"templateIssues":file.template_issues().len(),
            "templateInterruption":usize::from(file.template_interruption().is_some()),
            "interruptedPrograms":file.interrupted_programs().count(),
            "interpolationFailures":file.native_interpolation_failures().len()},
        "templateInterruption":file.template_interruption().map(|issue|format!("{issue:#?}")),
        "interruptedPrograms":file.interrupted_programs().map(|issue|format!("{issue:#?}")).collect::<Vec<_>>(),
        "rejectedHandlers":debug_rows(file.rejected_handlers()),
        "unattachedHandlers":file.unattached_handlers().map(|input|format!("{input:#?}")).collect::<Vec<_>>(),
        "unattachedForHeads":file.unattached_for_heads().map(|input|format!("{input:#?}")).collect::<Vec<_>>(),
        "rejectedForHeads":debug_rows(file.rejected_for_heads()),"interpolations":interpolations,
        "interpolationFailures":interpolation_failures,"attributeValues":attributes
    })
}

fn value_observation(value: &NativeAttributeValue<'_>, original_source: &str) -> Value {
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

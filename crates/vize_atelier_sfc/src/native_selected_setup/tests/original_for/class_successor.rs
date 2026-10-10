//! Named class+For source completes original custody, then refuses its body.
use super::*;
use serde_json::{Value, json};
use vize_l0::Span;
use vize_l1::markup::NativeAttributeValue;
use vize_l2::file::FileArtifact;
use vize_l2::file::NativeFileAttributeValueState;
use vize_l3::decision::dom::{DomRejection, DomUnsupported};
use vize_l4::{
    targets::dom::{DomError, DomErrorKind, emit_selected_setup_template},
    write::{NoLinks, Recorded},
};

pub(super) fn assert_current(class: &crate::NativeSelectedSetupSfcDomCompilation<'_>) {
    let source = class.observation().original().descriptor().source();
    assert_eq!(
        source,
        "<script setup>let count=2</script><template><i class='fixed' v-for='item in count'>fixed</i></template>"
    );
    let error = DomError {
        node: Some(NodeId::from_index(1).unwrap()),
        span: Span::new(44, 92),
        kind: DomErrorKind::Unsupported(DomUnsupported::ForBody),
    };
    assert_eq!(
        class.result().err(),
        Some(NativeSelectedSetupSfcDomError::Dom(error))
    );
    let owner = class.observation().original().template().unwrap();
    let view = owner.view().unwrap();
    let admitted = class.observation().admitted().unwrap();
    let setup = admitted.setup();
    let file = setup.file();
    assert!(file.is_complete());
    assert!(core::ptr::eq(file, view.file().unwrap()));
    assert!(owner.retained_setup().is_some());
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert_eq!(file.artifact().node_count(), 3);
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("whole original For root");
    };
    assert_eq!(original.id().node(), NodeId::FIRST);
    assert_eq!(original.span, Span::new(44, 92));
    let head = file.for_head_for(original).unwrap();
    assert_eq!(
        head.resolution().unwrap().input().operand().raw_value(),
        "item in count"
    );
    let [Op::Element(element)] = original.region.ops.as_slice() else {
        panic!("whole original class body");
    };
    assert_eq!(element.tag, "i");
    assert_eq!(element.span, Span::new(44, 92));
    let [Op::Text(text)] = element.children.ops.as_slice() else {
        panic!("whole original body text");
    };
    assert_eq!(text.content, "fixed");
    let analysis = build_native_selected_setup_dom_decisions(setup).unwrap();
    assert!(core::ptr::eq(analysis.owner(), owner));
    assert!(core::ptr::eq(analysis.setup(), setup));
    assert!(core::ptr::eq(analysis.file(), file));
    let facts = analysis.original_attributes().unwrap();
    assert!(core::ptr::eq(facts.file(), file));
    assert_eq!(facts.len(), 1);
    let [record] = file.native_attribute_values() else {
        panic!("whole original class value");
    };
    assert_eq!(
        record.state(),
        NativeFileAttributeValueState::Attached {
            node: NodeId::from_index(1).unwrap(),
            slot: 0
        }
    );
    let joined = facts.value(0, element, 0).unwrap();
    let value = joined.observation().unwrap();
    assert!(core::ptr::eq(joined.file(), file));
    assert!(core::ptr::eq(value, record.observation().unwrap()));
    assert!(core::ptr::eq(value.source().authored_root(), source));
    assert_eq!(
        (value.raw_value(), value.source().text()),
        ("fixed", "fixed")
    );
    assert_eq!(value.name_span(), Span::new(47, 52));
    assert_eq!(value.equals_span(), Span::new(52, 53));
    assert_eq!(value.full_value_span(), Span::new(53, 60));
    assert_eq!(value.value_span(), Span::new(54, 59));
    assert_eq!(
        value.quote_spans(),
        Some((Span::new(53, 54), Span::new(59, 60)))
    );
    assert!(value.source().decode_map().is_none());
    let attribute = joined.attribute().unwrap();
    assert_eq!(
        (attribute.name, attribute.value, attribute.span),
        ("class", Some("fixed"), Span::new(47, 60))
    );
    assert!(core::ptr::eq(
        attribute.value.unwrap(),
        value.source().text()
    ));
    assert_eq!(
        analysis.dom().unwrap().unsupported(),
        [
            DomRejection {
                node: NodeId::from_index(1).unwrap(),
                span: Span::new(44, 92),
                reason: DomUnsupported::ForBody
            },
            DomRejection {
                node: NodeId::FIRST,
                span: Span::new(44, 92),
                reason: DomUnsupported::ForBody
            },
        ]
    );
    assert_eq!(
        emit_selected_setup_template::<Recorded>(&analysis).unwrap_err(),
        error
    );
    assert_eq!(
        emit_selected_setup_template::<NoLinks>(&analysis).unwrap_err(),
        error
    );
    if let Ok(path) = std::env::var("VIZE_NATIVE_SELECTED_STATIC_CLASS_CAPTURE") {
        let capture = serde_json::json!({"source":source,"file":file_observation(file,source),"tables":format!("{:#?}",analysis.tables()),"headResolution":format!("{:#?}",head.resolution()),"error":format!("{error:#?}")});
        std::fs::write(
            format!("{path}.for-refusal.json"),
            serde_json::to_vec_pretty(&capture).unwrap(),
        )
        .unwrap();
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

use super::capture::Target;
use serde_json::{Value, json};
use std::fmt::Debug;
use vize_l0::{Span, String, cstr};
use vize_l1::markup::AttributeValueOperand;
use vize_l1_to_l2::native_file::{NativeSelectedSfcIssueKind, NativeSelectedSfcObservation};
use vize_l2::{
    ir::{L2Ir, NativeL2IrAttributeValueState},
    op::Op,
    walk::NodeRef,
};
use vize_l3::decision::{
    OriginalAttributeAnalysis, ssr::build_template_ssr_decisions,
    template::build_template_dom_decisions, vapor::build_template_vapor_decisions,
};

pub fn span(span: Span) -> Value {
    json!({"start":span.start,"end":span.end})
}

fn debug_rows<T: Debug>(rows: &[T]) -> Vec<String> {
    rows.iter().map(|row| cstr!("{row:#?}")).collect()
}

pub fn observation(original: &NativeSelectedSfcObservation<'_>, source: &str) -> Value {
    let descriptor = original.l1ir();
    let selected = original.template().map(|template| {
        let selected = template.selected();
        let component = selected.component();
        let block = component.block();
        json!({
            "debug":cstr!("{selected:#?}"),"index":selected.template_index(),
            "grammar":cstr!("{:?}",selected.grammar()),"blockSpan":span(block.span()),
            "blockSource":block.source(),"rootSource":block.root_source(),
            "sameRootSource":core::ptr::eq(block.root_source(),source),
            "viewError":template.view().err().map(|error|cstr!("{error:?}")),
            "rejectedFileDebug":template.rejected_file().map(|file|cstr!("{file:#?}")),
            "surfaceErrors":debug_rows(&component.carrier().errors),
            "surfaceUnsupported":debug_rows(&component.carrier().unsupported)
        })
    });
    let file = original
        .template()
        .and_then(|template| template.file())
        .map(|file| file_observation(file, source));
    let source_issues: Vec<_> = original.issues().iter().map(|issue|json!({
        "containerIndex":issue.container_index,"span":span(issue.span),
        "kind":cstr!("{:?}",issue.kind),"debug":cstr!("{issue:#?}"),
        "templateIssueKind":match issue.kind {
            NativeSelectedSfcIssueKind::Template(template)=>Some(cstr!("{:?}",template.kind)),
            _=>None
        }
    })).collect();
    json!({
        "debug":cstr!("{original:#?}"),"descriptorDebug":cstr!("{descriptor:#?}"),
        "descriptorOptionsDebug":cstr!("{:?}",descriptor.options()),
        "descriptorSameSource":core::ptr::eq(descriptor.source(),source),
        "descriptorIssues":debug_rows(descriptor.issues()),
        "descriptorErrors":debug_rows(&descriptor.container().errors),
        "counts":{"descriptorIssues":descriptor.issues().len(),
            "descriptorErrors":descriptor.container().errors.len(),"sourceIssues":source_issues.len()},
        "sourceIssues":source_issues,"admitted":original.admitted().is_some(),
        "rejectedCreationDebug":original.rejected_creation().map(|owner|cstr!("RejectedNativeTemplateOwner {{ selected: {:#?}, error: {:?} }}",owner.selected(),owner.error())),
        "interpolationFailureDebug":original.interpolation_failure().map(|failure|cstr!("{failure:#?}")),
        "selected":selected,"file":file
    })
}

fn file_observation(l2ir: &L2Ir<'_>, source: &str) -> Value {
    let artifact = l2ir.artifact();
    let provenance: Vec<_> = artifact.provenance().iter().map(|record|json!({
        "rule":record.rule.as_str(),"node":record.node.map(|node|node.index()),
        "before":record.before.as_str(),"after":record.after.as_str(),"span":span(record.span)
    })).collect();
    let bindings: Vec<_> = l2ir.bindings().map(|binding|json!({
        "id":binding.id().index(),"declarationDebug":binding.declaration().map(|row|cstr!("{row:#?}"))
    })).collect();
    let attributes: Vec<_> = l2ir
        .native_attribute_values()
        .iter()
        .enumerate()
        .map(|(index, row)| {
            json!({
                "index":index,"state":cstr!("{:?}",row.state()),
                "observation":row.observation().map(|value|value_observation(value,source)),
                "failure":row.failure().map(|failure|cstr!("{failure:#?}"))
            })
        })
        .collect();
    let interpolations: Vec<_> = l2ir.native_interpolations().iter().map(|row|json!({
        "state":cstr!("{:?}",row.state()),"inputDebug":cstr!("NativeInterpolationInput {{ full_span: {:?}, content_span: {:?}, raw_content: {:?}, syntax: {:#?} }}",row.input().operand().full_span(),row.input().operand().content_span(),row.input().operand().raw_content(),row.input().operand().syntax())
    })).collect();
    let interpolation_failures: Vec<_> = l2ir
        .native_interpolation_failures()
        .iter()
        .map(|row| {
            json!({
                "span":span(row.span()),"failureDebug":cstr!("{:#?}",row.failure())
            })
        })
        .collect();
    json!({
        "source":artifact.source(),"sameSource":core::ptr::eq(artifact.source(),source),
        "complete":l2ir.is_complete(),"artifactDebug":cstr!("{artifact:#?}"),
        "nodeCount":artifact.node_count(),"rootDebug":cstr!("{:#?}",artifact.root()),
        "artifactScopesDebug":cstr!("{:#?}",artifact.scopes()),"provenance":provenance,
        "units":debug_rows(l2ir.units()),"scopes":debug_rows(l2ir.scopes()),"bindings":bindings,
        "references":debug_rows(l2ir.references()),"exports":debug_rows(l2ir.exports()),
        "imports":debug_rows(l2ir.imports()),"issues":debug_rows(l2ir.issues()),
        "templateIssues":debug_rows(l2ir.template_issues()),
        "diagnosticCounts":{"issues":l2ir.issues().len(),"templateIssues":l2ir.template_issues().len(),
            "templateInterruption":usize::from(l2ir.template_interruption().is_some()),
            "interruptedPrograms":l2ir.interrupted_programs().count(),
            "interpolationFailures":l2ir.native_interpolation_failures().len()},
        "templateInterruption":l2ir.template_interruption().map(|issue|cstr!("{issue:#?}")),
        "interruptedPrograms":l2ir.interrupted_programs().map(|issue|cstr!("{issue:#?}")).collect::<Vec<_>>(),
        "rejectedHandlers":debug_rows(l2ir.rejected_handlers()),
        "unattachedHandlers":l2ir.unattached_handlers().map(|input|cstr!("{input:#?}")).collect::<Vec<_>>(),
        "unattachedForHeads":l2ir.unattached_for_heads().map(|input|cstr!("{input:#?}")).collect::<Vec<_>>(),
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
                    "kind":cstr!("{:?}",segment.kind())
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
        "sourceSpan":span(source.span()),"decodeMap":decode_map,"debugSource":cstr!("{source:#?}")
    })
}

pub fn l3(original: &NativeSelectedSfcObservation<'_>, target: Target) -> Value {
    let Some(admitted) = original.admitted() else {
        return l3_failure("lower-refusal", None);
    };
    let owner = original.template();
    let file = owner.and_then(|owner| owner.file());
    match target {
        Target::Dom => match build_template_dom_decisions(admitted.into_template_view()) {
            Ok(analysis) => l3_receipt(
                analysis.original_attributes(),
                owner.is_some_and(|owner| core::ptr::eq(owner, analysis.owner())),
                file.is_some_and(|file| core::ptr::eq(file, analysis.l2ir())),
                cstr!("{:#?}", analysis.tables()),
            ),
            Err(error) => l3_failure("analysis-refusal", Some(cstr!("{error:?}"))),
        },
        Target::Ssr => match build_template_ssr_decisions(admitted.into_template_view()) {
            Ok(analysis) => l3_receipt(
                analysis.original_attributes(),
                owner.is_some_and(|owner| core::ptr::eq(owner, analysis.owner())),
                file.is_some_and(|file| core::ptr::eq(file, analysis.l2ir())),
                cstr!("{:#?}", analysis.tables()),
            ),
            Err(error) => l3_failure("analysis-refusal", Some(cstr!("{error:?}"))),
        },
        Target::Vapor => match build_template_vapor_decisions(admitted.into_template_view()) {
            Ok(analysis) => l3_receipt(
                analysis.original_attributes(),
                owner.is_some_and(|owner| core::ptr::eq(owner, analysis.owner())),
                file.is_some_and(|file| core::ptr::eq(file, analysis.l2ir())),
                cstr!("{:#?}", analysis.tables()),
            ),
            Err(error) => l3_failure("analysis-refusal", Some(cstr!("{error:?}"))),
        },
    }
}

fn l3_failure(state: &str, error: Option<String>) -> Value {
    json!({"state":state,"publicError":error,"ownerSame":null,"fileSame":null,
        "tablesDebug":null,"valueCount":0,"visitError":null,"values":[]})
}

fn l3_receipt(
    facts: Option<&OriginalAttributeAnalysis<'_, '_>>,
    owner_same: bool,
    file_same: bool,
    tables: String,
) -> Value {
    let (values, visit_error) = facts.map_or_else(|| (Vec::new(), None), joined_values);
    json!({"state":"complete","publicError":null,"ownerSame":owner_same,"fileSame":file_same,
        "tablesDebug":tables,"valueCount":facts.map_or(0,OriginalAttributeAnalysis::len),
        "visitError":visit_error,"values":values})
}

fn joined_values<'owner, 'arena>(
    facts: &OriginalAttributeAnalysis<'owner, 'arena>,
) -> (Vec<Value>, Option<String>) {
    let file = facts.l2ir();
    let mut values = Vec::new();
    for (index, record) in file.native_attribute_values().iter().enumerate() {
        let (node, slot) = match record.state() {
            NativeL2IrAttributeValueState::Attached { node, slot } => (Some(node), Some(slot)),
            _ => (None, None),
        };
        values.push(
            json!({"index":index,"node":node.map(|node|node.index()),"slot":slot,
            "attribute":null,"sameFile":false,"sameObservation":false,
            "sameDecodedValue":false,"observation":null}),
        );
    }
    // This is an independent test readback of actual numbered Element allocations,
    // never a source decoder or caller-created attribute/provider tuple.
    let visit_error = file.artifact().visit_nodes(&mut |node, current| {
        let NodeRef::Op(Op::Element(element)) = current else { return; };
        for (index, record) in file.native_attribute_values().iter().enumerate() {
            let NativeL2IrAttributeValueState::Attached { node: actual, slot } = record.state()
            else { continue; };
            if actual != node { continue; }
            let Some(joined) = facts.value(index,element,slot) else { continue; };
            let Some(value) = joined.observation() else { continue; };
            let Some(attribute) = joined.attribute() else { continue; };
            if let Some(row) = values.get_mut(index) {
                *row = json!({"index":index,"node":node.index(),"slot":slot,
                    "attribute":{"name":attribute.name,"value":attribute.value,"span":span(attribute.span)},
                    "sameFile":core::ptr::eq(joined.l2ir(),file),
                    "sameObservation":record.observation().is_some_and(|original|core::ptr::eq(original,value)),
                    "sameDecodedValue":attribute.value.is_some_and(|decoded|core::ptr::eq(decoded,value.source().text())),
                    "observation":value_observation(value,file.artifact().source())});
            }
        }
    }).err().map(|error|cstr!("{error:?}"));
    (values, visit_error)
}

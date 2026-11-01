//! Check original tokens, AST, File and target role before writing any flags.

use serde_json::{Value, json};
use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
use vize_l2::{
    expr::ExprRef,
    file::{DeclarationKind, InitializerKind, Namespace, NativeFileModelState},
    op::{BindingOp, Op},
    resolution::Usage,
};
use vize_l3::decision::{
    native::build_native_selected_setup_dom_decisions,
    ssr::build_native_selected_setup_ssr_decisions,
    vapor::{VaporPart, build_native_selected_setup_vapor_decisions},
};

pub(super) fn capture(
    observation: &NativeSelectedSetupSfcObservation<'_>,
    fixture: &Value,
    target: &str,
    mode: &str,
    expected_modifier: &str,
) -> Result<Value, String> {
    let id = fixture["id"].as_str().ok_or("id")?;
    let source = fixture["source"].as_str().ok_or("source")?;
    let name = fixture["binding"].as_str().ok_or("binding")?;
    let view = observation
        .admitted()
        .ok_or("actual completed setup owner")?;
    let setup = view.setup();
    let file = setup.file();
    let [record] = file.native_models() else {
        return Err("one actual Model".into());
    };
    let NativeFileModelState::Admitted(node) = record.state() else {
        return Err("normally closed Model".into());
    };
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        return Err("original root Element".into());
    };
    let [BindingOp::Model(model)] = element.bindings.as_slice() else {
        return Err("one original attached Model".into());
    };
    let element_node = record.element_node().ok_or("real original opening")?;
    let [attribute] = model.attributes.as_slice() else {
        return Err("one actual modifier token".into());
    };
    let operand = record.input().operand();
    let modifier = match (
        expected_modifier,
        operand.trim_span(),
        operand.lazy_span(),
        operand.number_span(),
    ) {
        ("number", None, None, Some(span)) => span,
        _ => return Err("one exclusive original modifier span".into()),
    };
    if !file.is_complete()
        || file.units().len() != 1
        || file.scopes().len() != 2
        || file.artifact().node_count() != 2
        || node == element_node
        || !record.accepts(model)
        || !record.accepts_element(element_node, element)
        || element.tag != "input"
        || element.namespace != vize_l2::op::Namespace::Html
        || !element.attributes.is_empty()
        || !element.children.ops.is_empty()
        || model.argument.is_some()
        || attribute.span != modifier
        || attribute.value.is_some()
        || attribute.name != expected_modifier
        || !core::ptr::eq(attribute.name, modifier.slice(source))
        || record.input().attribute_span() != Some(model.span)
        || record.scope() != setup.scope()
        || !core::ptr::eq(
            record.input().operand().syntax().source().authored_root(),
            source,
        )
    {
        return Err(format!(
            "{id}/{target}/{mode}: original opening/modifier owner"
        ));
    }
    let (ExprRef::Js(read), ExprRef::Js(write)) = (model.contract.read, model.contract.write)
    else {
        return Err("one original Model read/write AST".into());
    };
    let resolution = file.expression(node).ok_or("Model expression")?;
    let table = resolution.table().ok_or("original semantic table")?;
    let [occurrence] = table.occurrences() else {
        return Err("one ReadWrite occurrence".into());
    };
    let binding = resolution
        .binding(occurrence.binding)
        .ok_or("same-File target")?;
    let declaration = binding.declaration().ok_or("direct declaration")?;
    let identifier = read
        .ast
        .get_identifier_reference()
        .ok_or("bare Identifier")?;
    let authored = read
        .authored_span(occurrence.span)
        .ok_or("authored occurrence")?;
    let retained = table.expression();
    if !core::ptr::eq(read, write)
        || !record.input().matches_expression(read)
        || !core::ptr::eq(read.ast, retained.ast)
        || !core::ptr::eq(read.source, retained.source)
        || read.span != retained.span
        || !matches!((read.coordinates,retained.coordinates),(Some(a),Some(b)) if core::ptr::eq(a,b))
        || resolution.scope() != Some(setup.scope())
        || !resolution.accepts(binding)
        || record.binding() != Some(binding.id())
        || occurrence.usage != Usage::ReadWrite
        || occurrence.shorthand
        || occurrence.constructor
        || declaration.id != occurrence.binding
        || declaration.namespace != Namespace::Value
        || !declaration.is_direct_program()
        || declaration.unit != setup.unit()
        || declaration.scope != setup.scope()
        || !matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        )
        || declaration.initializer != InitializerKind::PrimitiveLiteral
        || declaration.name != name
        || identifier.name.as_str() != name
        || occurrence.name != name
        || read.source != name
        || authored.slice(source) != name
        || read.ast_span_to_source(identifier.span) != Some(occurrence.span)
        || setup.binding(binding).is_err()
    {
        return Err(format!(
            "{id}/{target}/{mode}: original mutable binding/AST/coordinates"
        ));
    }
    let target_resolution = match target {
        "dom" => {
            let analysis = build_native_selected_setup_dom_decisions(setup)
                .map_err(|error| format!("{error:?}"))?;
            let facts = analysis.dom().ok_or("DOM facts")?;
            if !facts.unsupported().is_empty() {
                return Err(format!("{:?}", facts.unsupported()));
            }
            let row = facts.file_model(node).ok_or("actual DOM Model role")?;
            if !core::ptr::eq(row.original(), record) {
                return Err("DOM original owner".into());
            }
            row.resolution()
        }
        "ssr" => {
            let analysis = build_native_selected_setup_ssr_decisions(setup)
                .map_err(|error| format!("{error:?}"))?;
            if !analysis.ssr().ok_or("SSR facts")?.unsupported().is_empty() {
                return Err("SSR unsupported".into());
            }
            let row = analysis
                .model(element_node, element)
                .map_err(|error| format!("{error:?}"))?
                .ok_or("SSR Model")?;
            if !core::ptr::eq(row.original(), record)
                || !analysis
                    .expression(node)
                    .ok_or("SSR expression")?
                    .reads()
                    .is_empty()
            {
                return Err("SSR Model owner/read policy".into());
            }
            row.resolution()
        }
        "vapor" => {
            let analysis = build_native_selected_setup_vapor_decisions(setup)
                .map_err(|error| format!("{error:?}"))?;
            let facts = analysis.vapor().ok_or("Vapor facts")?;
            if !facts.unsupported().is_empty() {
                return Err("Vapor unsupported".into());
            }
            let [root] = facts.roots() else {
                return Err("one Vapor root".into());
            };
            let [
                VaporPart::Open {
                    node: opened,
                    element: actual,
                    void: true,
                },
                VaporPart::Model { model: row },
            ] = facts.parts(root).ok_or("Vapor root range")?
            else {
                return Err("Vapor original open/Model".into());
            };
            if *opened != element_node
                || !core::ptr::eq(*actual, &**element)
                || !core::ptr::eq(row.original().ok_or("Vapor original")?, record)
            {
                return Err("Vapor original owner".into());
            }
            row.resolution()
        }
        _ => return Err("closed target family".into()),
    };
    if target_resolution.node() != node
        || target_resolution.scope() != resolution.scope()
        || !core::ptr::eq(target_resolution.file(), file)
        || !target_resolution.accepts(binding)
    {
        return Err("target same-File identity".into());
    }
    Ok(json!({"id":id,"target":target,"mode":mode,"source":source,
        "model":node.index(),"element":element_node.index(),"scope":setup.scope().index(),
        "unit":setup.unit().index(),"binding":binding.id().index(),
        "name":name,"usage":format!("{:?}",occurrence.usage),
        "declarationKind":format!("{:?}",declaration.kind),"initializerKind":format!("{:?}",declaration.initializer),
        "declarationSpan":[declaration.span.start,declaration.span.end],"modelSpan":[model.span.start,model.span.end],
        "nameSpan":[record.input().operand().name_span().start,record.input().operand().name_span().end],
        "valueSpan":[record.input().operand().value_span().start,record.input().operand().value_span().end],
        "modifier":{"name":attribute.name,"value":attribute.value,"span":[modifier.start,modifier.end]},
        "expressionSpan":[read.span.start,read.span.end],"authoredReadWriteSpan":[authored.start,authored.end],
        "actualElementPointer":true,"actualModelPointer":true,"sameFileBinding":true,
        "sameOriginalAstAndCoordinates":true,"actualCanonicalTargetRole":true,"genericReadCount":0}))
}

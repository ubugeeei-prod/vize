//! Actual File/Model/Element and distinct L3 read/write receipt, never a name list.

use super::*;
use vize_l2::{
    expr::ExprRef,
    file::{DeclarationKind, InitializerKind, Namespace, NativeFileModelState},
    op::{BindingOp, Op},
    resolution::Usage,
};
use vize_l3::decision::dom::{DomChild, DomChildren, DomDependency, PropertyRole, ValueKind};

pub(super) fn capture(
    compilation: &NativeSelectedSetupSfcDomCompilation<'_>,
    fixture: &Value,
    mode: &str,
) -> Result<Value, String> {
    let id = text(fixture, "id")?;
    let source = text(fixture, "source")?;
    let name = text(fixture, "binding")?;
    let view = compilation.observation().admitted().ok_or("setup owner")?;
    let setup = view.setup();
    let file = setup.file();
    require!(
        file.is_complete() && file.units().len() == 1 && file.scopes().len() == 2,
        "{id}: actual whole File/Program/scope"
    );
    require!(
        core::ptr::eq(
            setup.syntax(),
            setup.owner().retained_setup().ok_or("whole setup syntax")?
        ),
        "{id}: exact original setup owner"
    );
    let [record] = file.native_models() else {
        return Err(cstr!("{id}: one original Model"));
    };
    let NativeFileModelState::Admitted(node) = record.state() else {
        return Err(cstr!("{id}: closed Model"));
    };
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        return Err(cstr!("{id}: one real root input"));
    };
    let [BindingOp::Model(model)] = element.bindings.as_slice() else {
        return Err(cstr!("{id}: one actual attached Model"));
    };
    let element_node = record.element_node().ok_or("actual closed Element")?;
    require!(
        node != element_node
            && file.artifact().node_count() == 2
            && record.accepts(model)
            && record.accepts_element(element_node, element),
        "{id}: actual Model/parent node and pointer identity"
    );
    require!(
        element.tag == "input"
            && element.namespace == vize_l2::op::Namespace::Html
            && element.attributes.is_empty()
            && element.children.ops.is_empty()
            && model.argument.is_none()
            && model.attributes.is_empty(),
        "{id}: exact first input Model family"
    );
    require!(
        record.input().attribute_span() == Some(model.span)
            && record.scope() == setup.scope()
            && core::ptr::eq(record, file.native_model(node).ok_or("same model query")?),
        "{id}: current complete File row/attribute/scope"
    );
    let (ExprRef::Js(read), ExprRef::Js(write)) = (model.contract.read, model.contract.write)
    else {
        return Err(cstr!("{id}: original read/write AST"));
    };
    let syntax = record.input().operand().syntax();
    require!(
        core::ptr::eq(read, write)
            && record.input().matches_expression(read)
            && core::ptr::eq(syntax.expression().ok_or("retained AST")?, read.ast)
            && core::ptr::eq(syntax.source().authored_root(), source)
            && core::ptr::eq(syntax.source().text(), read.source),
        "{id}: one original parsed Identifier no generated assignment AST"
    );
    let resolution = file.expression(node).ok_or("actual model resolution")?;
    let table = resolution.table().ok_or("one semantic reference table")?;
    let [occurrence] = table.occurrences() else {
        return Err(cstr!("{id}: one actual Identifier occurrence"));
    };
    let retained = table.expression();
    let binding = resolution
        .binding(occurrence.binding)
        .ok_or("actual same File binding")?;
    let declaration = binding.declaration().ok_or("original declaration")?;
    let target = setup.binding(binding).map_err(|error| cstr!("{error:?}"))?;
    let identifier = read
        .ast
        .get_identifier_reference()
        .ok_or("original bare Identifier")?;
    let authored = read
        .authored_span(occurrence.span)
        .ok_or("authored occurrence")?;
    require!(
        resolution.scope() == Some(setup.scope())
            && resolution.accepts(binding)
            && record.binding() == Some(binding.id())
            && declaration.id == occurrence.binding
            && occurrence.usage == Usage::ReadWrite
            && !occurrence.shorthand
            && !occurrence.constructor,
        "{id}: one same-File semantic ReadWrite, no caller usage flag"
    );
    require!(
        core::ptr::eq(retained.ast, read.ast)
            && core::ptr::eq(retained.source, read.source)
            && retained.span == read.span
            && core::ptr::eq(
                retained.coordinates.ok_or("retained coordinates")?,
                read.coordinates.ok_or("original coordinates")?
            ),
        "{id}: whole actual AST/source/coordinate identity"
    );
    require!(
        declaration.is_direct_program()
            && declaration.unit == setup.unit()
            && declaration.scope == setup.scope()
            && declaration.namespace == Namespace::Value
            && matches!(
                declaration.kind,
                DeclarationKind::Let | DeclarationKind::Var
            ),
        "{id}: mutable original declaration authority independent of initializer"
    );
    require!(
        declaration.initializer == InitializerKind::PrimitiveLiteral
            && declaration.name.as_str() == name
            && identifier.name.as_str() == name
            && occurrence.name == name
            && read.source == name
            && authored.slice(source) == name
            && read.ast_span_to_source(identifier.span) == Some(occurrence.span)
            && target.same_owner(binding),
        "{id}: primitive target value policy and exact original named coordinates"
    );
    let analysis =
        build_native_selected_setup_dom_decisions(setup).map_err(|error| cstr!("{error:?}"))?;
    let dom = analysis.dom().ok_or("DOM decisions")?;
    let model_row = dom
        .file_model(node)
        .ok_or("separate genuine Model write receipt")?;
    let [DomChild::Node(root)] = dom.root().children.as_slice() else {
        return Err(cstr!("{id}: one real root DOM node"));
    };
    let body = dom.node(element_node).ok_or("actual input node")?;
    let fact = dom.binding(node).ok_or("actual Model role")?;
    require!(
        dom.unsupported().is_empty()
            && *root == element_node
            && analysis.expression(node).is_none()
            && file.native_interpolations().is_empty(),
        "{id}: no generic read/write or fake interpolation promotion"
    );
    require!(
        model_row.accepts_model(model)
            && model_row.accepts_element(element_node, element)
            && core::ptr::eq(model_row.original(), record)
            && core::ptr::eq(model_row.occurrence(), occurrence)
            && model_row.binding().same_owner(binding)
            && model_row.resolution().node() == node
            && core::ptr::eq(model_row.resolution().file(), file)
            && model_row.authored_span() == authored,
        "{id}: L3 borrows exact current FileModel/Element/Binding/ReadWrite identity"
    );
    require!(
        matches!(body.children, DomChildren::Empty)
            && body.block_eligible
            && body.changes.properties
            && matches!(body.op(), Op::Element(actual) if core::ptr::eq(&**actual, &**element))
            && body.dynamic_property_bindings.as_slice() == [node]
            && matches!(fact.binding(), BindingOp::Model(actual) if core::ptr::eq(&**actual, &**model))
            && fact.role == PropertyRole::Model
            && fact.value == ValueKind::FileDependent,
        "{id}: actual input grouping/property decision"
    );
    require!(
        dom.dependencies()
            == [
                DomDependency::TextModel,
                DomDependency::DirectiveApplication,
                DomDependency::BlockBoundary,
                DomDependency::NativeElementBlock
            ],
        "{id}: exact ordered actual semantic dependency demand"
    );
    Ok(serde_json::json!({
        "id":id,"mode":mode,"source":source,"model":node.index(),"element":element_node.index(),
        "scope":setup.scope().index(),"unit":setup.unit().index(),"binding":binding.id().index(),
        "declarationKind":cstr!("{:?}",declaration.kind),"initializerKind":cstr!("{:?}",declaration.initializer),
        "declarationSpan":[declaration.span.start,declaration.span.end],"modelSpan":[model.span.start,model.span.end],
        "nameSpan":[record.input().operand().name_span().start,record.input().operand().name_span().end],
        "valueSpan":[record.input().operand().value_span().start,record.input().operand().value_span().end],
        "expressionSpan":[read.span.start,read.span.end],"authoredReadWriteSpan":[authored.start,authored.end],
        "name":name,"usage":cstr!("{:?}",occurrence.usage),"shorthand":occurrence.shorthand,"constructor":occurrence.constructor,
        "actualElementPointer":true,"actualModelPointer":true,"sameFileBinding":true,"sameOriginalAstAndCoordinates":true,
        "modelValueKind":cstr!("{:?}",fact.value),"modelRole":cstr!("{:?}",fact.role),
        "dependencies":dom.dependencies().iter().map(|dependency|cstr!("{dependency:?}")).collect::<Vec<_>>(),
    }))
}

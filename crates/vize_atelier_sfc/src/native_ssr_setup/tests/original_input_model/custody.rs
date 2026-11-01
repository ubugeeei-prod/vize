//! Actual File/Model/Element and distinct L3 read/write receipt, never a name list.

use super::*;
use vize_l2::{
    expr::ExprRef,
    file::{DeclarationKind, InitializerKind, Namespace, NativeFileModelState},
    op::{BindingOp, Op},
    resolution::Usage,
};
use vize_l3::decision::ssr::{SsrPart, build_native_selected_setup_ssr_decisions};

pub(super) fn capture(
    compilation: &NativeSetupSsrSfcCompilation<'_>,
    fixture: &Value,
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
        build_native_selected_setup_ssr_decisions(setup).map_err(|error| cstr!("{error:?}"))?;
    let facts = analysis.ssr().ok_or("SSR decisions")?;
    let [
        SsrPart::Open {
            node: root,
            element: opened,
            void: true,
        },
    ] = facts.parts()
    else {
        return Err(cstr!("{id}: one genuine void opening"));
    };
    let row = analysis
        .model(*root, opened)
        .map_err(|error| cstr!("{error:?}"))?
        .ok_or("dedicated SSR Model value receipt")?;
    require!(
        facts.unsupported().is_empty()
            && !facts.fragment()
            && facts.inherit_attrs() == Some(element_node)
            && *root == element_node
            && core::ptr::eq(*opened, &**element),
        "{id}: actual whole SSR root/opening"
    );
    require!(
        analysis
            .expression(node)
            .ok_or("canonical Model role")?
            .reads()
            .is_empty()
            && file.native_interpolations().is_empty(),
        "{id}: ReadWrite cannot become a generic SSR read or fake interpolation"
    );
    require!(
        core::ptr::eq(row.original(), record)
            && core::ptr::eq(row.model(), &**model)
            && core::ptr::eq(row.occurrence(), occurrence)
            && row.binding().same_owner(binding)
            && row.resolution().node() == node
            && core::ptr::eq(row.resolution().file(), file)
            && row.authored_span() == authored,
        "{id}: exact retained SSR Model/Element/Binding/ReadWrite identity"
    );
    Ok(serde_json::json!({
        "id":id,"source":source,"model":node.index(),"element":element_node.index(),
        "scope":setup.scope().index(),"unit":setup.unit().index(),"binding":binding.id().index(),
        "declarationKind":cstr!("{:?}",declaration.kind),"initializerKind":cstr!("{:?}",declaration.initializer),
        "declarationSpan":[declaration.span.start,declaration.span.end],"modelSpan":[model.span.start,model.span.end],
        "nameSpan":[record.input().operand().name_span().start,record.input().operand().name_span().end],
        "valueSpan":[record.input().operand().value_span().start,record.input().operand().value_span().end],
        "expressionSpan":[read.span.start,read.span.end],"authoredReadWriteSpan":[authored.start,authored.end],
        "name":name,"usage":cstr!("{:?}",occurrence.usage),"shorthand":occurrence.shorthand,"constructor":occurrence.constructor,
        "actualElementPointer":true,"actualModelPointer":true,"sameFileBinding":true,"sameOriginalAstAndCoordinates":true,
        "target":"SSR","modelRole":"ModelValue","genericReadCount":0,
        "rootFallthrough":facts.inherit_attrs().map(|node|node.index()),"fragment":facts.fragment(),
    }))
}

//! Real original File/Model/Element/Binding/ReadWrite joins before receipt flags.

use super::*;
use vize_l2::{
    expr::ExprRef,
    file::{DeclarationKind, InitializerKind, Namespace, NativeFileModelState},
    op::{BindingOp, Op},
    resolution::Usage,
};
use vize_l3::decision::vapor::{VaporPart, build_native_selected_setup_vapor_decisions};

pub(super) fn capture(
    compilation: &NativeVaporSetupSfcCompilation<'_>,
    fixture: &Value,
) -> Result<Value, String> {
    let id = text(fixture, "id")?;
    let source = text(fixture, "source")?;
    let name = text(fixture, "binding")?;
    let view = compilation
        .observation()
        .admitted()
        .ok_or("actual setup owner")?;
    let setup = view.setup();
    let file = setup.file();
    require!(
        file.is_complete() && file.units().len() == 1 && file.scopes().len() == 2,
        "{id}: complete original File/Program/scope"
    );
    require!(
        core::ptr::eq(
            setup.syntax(),
            setup.owner().retained_setup().ok_or("retained syntax")?
        ),
        "{id}: actual original setup syntax owner"
    );
    let [record] = file.native_models() else {
        return Err(cstr!("{id}: one original Model"));
    };
    let NativeFileModelState::Admitted(node) = record.state() else {
        return Err(cstr!("{id}: normally closed Model"));
    };
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        return Err(cstr!("{id}: one original root input"));
    };
    let [BindingOp::Model(model)] = element.bindings.as_slice() else {
        return Err(cstr!("{id}: one attached original Model"));
    };
    let element_node = record.element_node().ok_or("closed original Element")?;
    require!(
        node != element_node
            && file.artifact().node_count() == 2
            && record.accepts(model)
            && record.accepts_element(element_node, element),
        "{id}: real Model/parent pointer and NodeId"
    );
    require!(
        element.tag == "input"
            && element.namespace == vize_l2::op::Namespace::Html
            && element.attributes.is_empty()
            && element.children.ops.is_empty()
            && model.argument.is_none()
            && model.attributes.is_empty(),
        "{id}: bounded original input Model family"
    );
    require!(
        record.input().attribute_span() == Some(model.span) && record.scope() == setup.scope(),
        "{id}: same original attribute and current scope"
    );
    let (ExprRef::Js(read), ExprRef::Js(write)) = (model.contract.read, model.contract.write)
    else {
        return Err(cstr!("{id}: original read/write AST"));
    };
    let syntax = record.input().operand().syntax();
    require!(
        core::ptr::eq(read, write)
            && record.input().matches_expression(read)
            && core::ptr::eq(syntax.expression().ok_or("retained expression")?, read.ast)
            && core::ptr::eq(syntax.source().authored_root(), source)
            && core::ptr::eq(syntax.source().text(), read.source),
        "{id}: original parsed Identifier source and AST"
    );
    let resolution = file.expression(node).ok_or("Model resolution")?;
    let table = resolution.table().ok_or("original semantic table")?;
    let [occurrence] = table.occurrences() else {
        return Err(cstr!("{id}: one occurrence"));
    };
    let binding = resolution
        .binding(occurrence.binding)
        .ok_or("actual binding")?;
    let declaration = binding.declaration().ok_or("actual declaration")?;
    let target = setup.binding(binding).map_err(|error| cstr!("{error:?}"))?;
    let identifier = read
        .ast
        .get_identifier_reference()
        .ok_or("bare actual Identifier")?;
    let authored = read
        .authored_span(occurrence.span)
        .ok_or("authored occurrence")?;
    let retained = table.expression();
    require!(
        resolution.scope() == Some(setup.scope())
            && resolution.accepts(binding)
            && record.binding() == Some(binding.id())
            && declaration.id == occurrence.binding
            && occurrence.usage == Usage::ReadWrite
            && !occurrence.shorthand
            && !occurrence.constructor,
        "{id}: original same-File ReadWrite occurrence"
    );
    require!(
        core::ptr::eq(retained.ast, read.ast)
            && core::ptr::eq(retained.source, read.source)
            && retained.span == read.span
            && core::ptr::eq(
                retained.coordinates.ok_or("retained coordinates")?,
                read.coordinates.ok_or("original coordinates")?
            ),
        "{id}: retained complete AST/source/coordinate identity"
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
        "{id}: mutable declaration authority independent of primitive value type"
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
        "{id}: bounded primitive target and original named coordinates"
    );
    let analysis =
        build_native_selected_setup_vapor_decisions(setup).map_err(|error| cstr!("{error:?}"))?;
    let facts = analysis.vapor().ok_or("Vapor facts")?;
    let [root] = facts.roots() else {
        return Err(cstr!("{id}: one original Vapor root"));
    };
    let [
        VaporPart::Open {
            node: opened_node,
            element: opened,
            void: true,
        },
        VaporPart::Model { model: row },
    ] = facts.parts(root).ok_or("actual root range")?
    else {
        return Err(cstr!("{id}: actual opening plus dedicated Model event"));
    };
    require!(
        facts.unsupported().is_empty()
            && facts.inherit_attrs() == Some(element_node)
            && *opened_node == element_node
            && root.node() == element_node
            && core::ptr::eq(*opened, &**element),
        "{id}: whole canonical original opening"
    );
    require!(
        core::ptr::eq(row.original().ok_or("original row")?, record)
            && core::ptr::eq(row.occurrence().ok_or("original occurrence")?, occurrence)
            && row.binding().ok_or("original binding")?.same_owner(binding)
            && row.resolution().node() == node
            && core::ptr::eq(row.resolution().file(), file)
            && row.authored_span() == Some(authored)
            && file.native_interpolations().is_empty(),
        "{id}: dedicated canonical Model role, no fake interpolation or generic write permission"
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
        "target":"Vapor","modelRole":"InlineModel","genericReadCount":0,
        "rootFallthrough":facts.inherit_attrs().map(|node|node.index()),
    }))
}

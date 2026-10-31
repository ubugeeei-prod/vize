//! First genuine whole root If/text observations; no desired-output fallback.

use super::*;
use vize_l2::{
    expr::ExprRef,
    file::{NativeFileConditionalState, NativeFileInterpolationState},
    op::Op,
    resolution::Usage,
};
use vize_l3::decision::dom::DomChildren;

#[test]
fn eight_whole_original_root_if_text_components_and_raw_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_root_if_text_inputs_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let goldens: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_root_if_text_sfc_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    require!(
        goldens["schema"] == "vize.native-sfc.original-root-if-text-complete-goldens"
            && goldens["filename"] == pack["filename"],
        "whole root If/text golden authority"
    );
    require!(
        pack["schema"] == "vize.native-sfc.original-root-if-text-inputs",
        "original root If/text input schema"
    );
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(fixtures.len() == 4, "four original complete inputs");
    let mut captures = Vec::new();
    let mut custody = Vec::new();
    let mut attempts = Vec::new();
    let mut failures = Vec::new();
    for (mode, mode_name) in [
        (DomMode::Development, "development"),
        (DomMode::Production, "production"),
    ] {
        for fixture in fixtures {
            let id = text(fixture, "id")?;
            let source = text(fixture, "source")?;
            let arena = Allocator::default();
            let options = NativeSelectedSfcDomOptions {
                source_map: true,
                filename: text(&pack, "filename")?,
                ..NativeSelectedSfcDomOptions::default()
            };
            let mapped =
                compile_native_selected_setup_sfc_dom_with_mode(&arena, source, options, mode);
            let plain = compile_native_selected_setup_sfc_dom_with_mode(
                &arena,
                source,
                NativeSelectedSfcDomOptions {
                    source_map: false,
                    ..options
                },
                mode,
            );
            let default = (mode == DomMode::Development)
                .then(|| compile_native_selected_setup_sfc_dom(&arena, source, options));
            let observe = |compilation: &NativeSelectedSetupSfcDomCompilation<'_>| match compilation
                .result()
            {
                Ok(output) => serde_json::json!({
                    "state":"emitted", "code":output.code(), "nativeMapRaw":output.source_map(),
                    "nativeMap":output.source_map().and_then(|raw| serde_json::from_str::<Value>(raw).ok()),
                }),
                Err(error) => serde_json::json!({"state":"refused","error":cstr!("{error:?}")}),
            };
            attempts.push(serde_json::json!({
                "id":id,"mode":mode_name,"source":source,"mapped":observe(&mapped),
                "plain":observe(&plain),"defaultDevelopment":default.as_ref().map(observe),
            }));
            let validated = (|| -> Result<(Value, Value), String> {
                let output = mapped
                    .result()
                    .map_err(|error| cstr!("{id}/{mode_name}: {error:?}"))?;
                let unlinked = plain
                    .result()
                    .map_err(|error| cstr!("{id}/{mode_name}: plain {error:?}"))?;
                require!(
                    output.code() == unlinked.code()
                        && unlinked.source_map().is_none()
                        && unlinked.document().links().is_empty(),
                    "{id}/{mode_name}: complete NoLinks equality"
                );
                if let Some(default) = &default {
                    let default = default
                        .result()
                        .map_err(|error| cstr!("{id}: default {error:?}"))?;
                    require!(
                        default.code() == output.code()
                            && default.source_map() == output.source_map(),
                        "{id}: complete default development code/map equality"
                    );
                }
                let raw_map = output.source_map().ok_or("complete raw map")?;
                let map: Value = serde_json::from_str(raw_map).map_err(|error| cstr!("{error}"))?;
                require!(
                    map["sourcesContent"] == serde_json::json!([source]),
                    "{id}: full original source"
                );
                let view = mapped
                    .observation()
                    .admitted()
                    .ok_or("original setup owner")?;
                let setup = view.setup();
                let file = setup.file();
                require!(
                    file.is_complete() && file.units().len() == 1 && file.scopes().len() == 2,
                    "{id}: same complete File and sole setup scope without alias scope"
                );
                require!(
                    core::ptr::eq(
                        setup.syntax(),
                        setup.owner().retained_setup().ok_or("whole setup syntax")?
                    ),
                    "{id}: same normally retained Program syntax"
                );
                let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
                    return Err(cstr!("{id}: actual root If"));
                };
                let [condition_record] = file.native_conditionals() else {
                    return Err(cstr!("{id}: one original condition"));
                };
                let NativeFileConditionalState::Admitted(if_node) = condition_record.state() else {
                    return Err(cstr!("{id}: normally closed If"));
                };
                require!(
                    if_node == NodeId::FIRST
                        && condition_record.accepts(original)
                        && file
                            .native_conditional(if_node)
                            .is_some_and(|row| core::ptr::eq(row, condition_record)),
                    "{id}: actual introducing If allocation and dense root id"
                );
                let [branch] = original.branches.as_slice() else {
                    return Err(cstr!("{id}: one real branch"));
                };
                let Some(ExprRef::Js(condition)) = branch.condition else {
                    return Err(cstr!("{id}: original condition AST"));
                };
                require!(
                    condition_record.input().matches_expression(condition),
                    "{id}: exact original conditional wrapper"
                );
                let [body_op @ Op::Element(element)] = branch.region.ops.as_slice() else {
                    return Err(cstr!("{id}: original branch Element"));
                };
                let [text_op @ Op::Interpolation(interpolation)] = element.children.ops.as_slice()
                else {
                    return Err(cstr!("{id}: one actual body interpolation"));
                };
                let ExprRef::Js(expression) = interpolation.expression else {
                    return Err(cstr!("{id}: original text AST"));
                };
                let [text_record] = file.native_interpolations() else {
                    return Err(cstr!("{id}: one original text input"));
                };
                let NativeFileInterpolationState::Admitted(text_node) = text_record.state() else {
                    return Err(cstr!("{id}: normally minted text"));
                };
                let syntax = text_record.input().operand().syntax();
                require!(
                    file.native_interpolation(text_node)
                        .is_some_and(|row| core::ptr::eq(row, text_record))
                        && interpolation.span == text_record.input().operand().full_span()
                        && core::ptr::eq(
                            syntax.expression().ok_or("original text root")?,
                            expression.ast
                        )
                        && core::ptr::eq(syntax.source().text(), expression.source)
                        && core::ptr::eq(syntax.source().authored_root(), source)
                        && syntax.source().span() == expression.span,
                    "{id}: same actual original text node/AST/source/coordinates"
                );
                require!(
                    if_node != text_node && !core::ptr::eq(condition.ast, expression.ast),
                    "{id}: genuinely distinct condition and body occurrences, including same binding"
                );
                let analysis = build_native_selected_setup_dom_decisions(setup)
                    .map_err(|error| cstr!("{id}: {error:?}"))?;
                let dom = analysis.dom().ok_or("DOM")?;
                require!(
                    dom.unsupported().is_empty(),
                    "{id}: complete native runtime closure"
                );
                let condition_row = analysis.expression(if_node).ok_or("condition decisions")?;
                let text_row = analysis.expression(text_node).ok_or("text decisions")?;
                let [condition_read] = condition_row.reads() else {
                    return Err(cstr!("{id}: sole original condition READ"));
                };
                let [text_read] = text_row.reads() else {
                    return Err(cstr!("{id}: sole original text READ"));
                };
                require!(
                    !core::ptr::eq(condition_read.occurrence(), text_read.occurrence()),
                    "{id}: distinct resolver occurrences, never copied binding-name evidence"
                );
                for (node, actual, row, read, expected_name) in [
                    (
                        if_node,
                        condition,
                        condition_row,
                        condition_read,
                        text(fixture, "conditionBinding")?,
                    ),
                    (
                        text_node,
                        expression,
                        text_row,
                        text_read,
                        text(fixture, "textBinding")?,
                    ),
                ] {
                    let resolution = row.resolution();
                    let table = resolution.table().ok_or("whole original resolution")?;
                    let [occurrence] = table.occurrences() else {
                        return Err(cstr!("{id}: one retained occurrence"));
                    };
                    let binding = setup
                        .binding(read.binding())
                        .map_err(|error| cstr!("{error:?}"))?;
                    let declaration = binding.declaration().ok_or("real script declaration")?;
                    let retained = table.expression();
                    require!(
                        core::ptr::eq(resolution.file(), file)
                            && resolution.node() == node
                            && resolution.scope() == Some(setup.scope())
                            && condition_record.scope() == setup.scope()
                            && core::ptr::eq(occurrence, read.occurrence())
                            && occurrence.usage == Usage::Read
                            && occurrence.binding == binding.id()
                            && read.binding().same_owner(binding)
                            && declaration.name.as_str() == expected_name
                            && occurrence.name == expected_name
                            && declaration.unit == setup.unit()
                            && declaration.scope == setup.scope()
                            && declaration.is_direct_program()
                            && declaration.initializer.is_primitive()
                            && read.kind() == VueReadKind::SetupLet
                            && core::ptr::eq(retained.ast, actual.ast)
                            && core::ptr::eq(retained.source, actual.source)
                            && retained.span == actual.span
                            && core::ptr::eq(
                                retained.coordinates.ok_or("resolver coordinates")?,
                                actual.coordinates.ok_or("original coordinates")?
                            ),
                        "{id}: current File/scope/BindingId/original AST/occurrence/coordinate identity"
                    );
                }
                let condition_binding = condition_read.binding();
                let text_binding = text_read.binding();
                require!(
                    condition_record.binding() == Some(condition_binding.id())
                        && condition_binding
                            .declaration()
                            .ok_or("condition declaration")?
                            .boolean_initializer()
                            == fixture["ifTextRuntime"]["initial"]["visible"].as_bool(),
                    "{id}: actual BooleanLiteral initializer event"
                );
                require!(
                    (condition_binding.id() == text_binding.id())
                        == (fixture["conditionBinding"] == fixture["textBinding"]),
                    "{id}: one genuine binding at two distinct nodes only for same-binding input"
                );
                let control = dom.conditional(if_node).ok_or("owned conditional")?;
                let [branch_row] = control.branches() else {
                    return Err(cstr!("{id}: one current branch row"));
                };
                require!(
                    core::ptr::eq(control.owner(), original.as_ref())
                        && core::ptr::eq(branch_row.owner(), branch)
                        && branch_row.discriminant().index() == 0
                        && branch_row.condition() == Some(condition_row.value()),
                    "{id}: actual source-order branch and condition facts"
                );
                let body = dom.node(branch_row.root()).ok_or("real branch body")?;
                let DomChildren::Text(group) = &body.children else {
                    return Err(cstr!("{id}: actual singleton text group"));
                };
                require!(
                    core::ptr::eq(body.op(), body_op)
                        && body.block_eligible
                        && element.attributes.is_empty()
                        && element.bindings.is_empty()
                        && Some(element.tag) == fixture["ifTextRuntime"]["tag"].as_str()
                        && matches!(group.nodes.as_slice(), [actual] if *actual == text_node)
                        && group.dynamic
                        && body.changes.text
                        && dom
                            .node(text_node)
                            .is_some_and(|row| core::ptr::eq(row.op(), text_op)),
                    "{id}: real direct Element/text nodes and dynamic text facts"
                );
                for link in output.document().links() {
                    require!(
                        source
                            .get(link.authored.start as usize..link.authored.end as usize)
                            .is_some()
                            && output
                                .code()
                                .get(link.generated.start as usize..link.generated.end as usize)
                                .is_some(),
                        "{id}: actual full-source UTF-8 link geometry"
                    );
                }
                let condition_span = condition
                    .authored_span(condition_read.occurrence().span)
                    .ok_or("condition token")?;
                let text_span = expression
                    .authored_span(text_read.occurrence().span)
                    .ok_or("text token")?;
                Ok((
                    serde_json::json!({"id":id,"mode":mode_name,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw_map}),
                    serde_json::json!({
                        "id":id,"mode":mode_name,"ifNode":if_node.index(),"scope":setup.scope().index(),
                        "conditionBinding":condition_binding.id().index(),"textBinding":text_binding.id().index(),
                        "conditionBooleanInitializer":condition_binding.declaration().and_then(|row| row.boolean_initializer()),
                        "conditionReadKind":cstr!("{:?}",condition_read.kind()),"textReadKind":cstr!("{:?}",text_read.kind()),
                        "conditionValueKind":cstr!("{:?}",condition_row.value()),"textValueKind":cstr!("{:?}",text_row.value()),
                        "conditionAuthoredSpan":[condition_span.start,condition_span.end],"textAuthoredSpan":[text_span.start,text_span.end],
                        "conditionNameSpan":[condition_record.input().operand().name_span().start,condition_record.input().operand().name_span().end],
                        "textFullSpan":[interpolation.span.start,interpolation.span.end],
                        "bodyNode":branch_row.root().index(),"textNode":text_node.index(),
                        "discriminant":branch_row.discriminant().index(),"fallback":cstr!("{:?}",control.fallback()),
                        "bodyDynamic":group.dynamic,"bodyTextChange":body.changes.text,
                    }),
                ))
            })();
            match validated {
                Ok((capture, owner)) => {
                    captures.push(capture);
                    custody.push(owner);
                }
                Err(error) => failures.push(error),
            }
        }
    }
    let emitted = captures.len();
    let captures = Value::Array(captures);
    let custody = Value::Array(custody);
    let attempts = Value::Array(attempts);
    for (field, actual) in [
        ("fixtures", &captures),
        ("custody", &custody),
        ("attempts", &attempts),
    ] {
        if goldens["native"].get(field) != Some(actual) {
            failures.push(cstr!("whole ordered native {field} golden mismatch"));
        }
    }
    let qualified = failures.is_empty() && emitted == fixtures.len() * 2;
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_ROOT_IF_TEXT_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-root-if-text-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if qualified { "current-complete-native-goldens-matched" } else { "failed-current-complete-native-goldens" },
            "nativeGoldenComparisonPerformed":true,
            "sourceRevision":std::env::var("GITHUB_SHA").ok(),
            "fixtures":captures,"custody":custody,"attempts":attempts,"failures":&failures,
        })).map_err(|error| cstr!("{error}"))?).map_err(|error| cstr!("{error}"))?;
    }
    require!(
        failures.is_empty(),
        "complete original If/text capture failures: {failures:?}"
    );
    require!(
        emitted == fixtures.len() * 2,
        "all eight mode-major whole original inputs emitted"
    );
    Ok(())
}

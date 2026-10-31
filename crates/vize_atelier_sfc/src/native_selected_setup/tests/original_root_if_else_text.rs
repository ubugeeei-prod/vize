//! First genuine paired setup-text observations; complete current oracles remain pending.

use super::*;
use vize_l2::{
    expr::ExprRef,
    file::{NativeFileConditionalState, NativeFileInterpolationState},
    op::Op,
    resolution::Usage,
};
use vize_l3::decision::dom::{DomChildren, control::ConditionalFallback};

mod refusal;

#[test]
fn twelve_whole_original_root_if_else_text_components_and_raw_maps_are_captured()
-> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_root_if_else_text_inputs_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    require!(
        pack["schema"] == "vize.native-sfc.original-root-if-else-text-inputs",
        "original paired setup-text input authority"
    );
    let goldens: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_root_if_else_text_sfc_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    require!(
        goldens["schema"] == "vize.native-sfc.original-root-if-else-text-complete-goldens"
            && goldens["filename"] == pack["filename"],
        "whole paired setup-text golden authority"
    );
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(fixtures.len() == 6, "six whole original inputs");
    let mut captures = Vec::new();
    let mut custody = Vec::new();
    let mut attempts = Vec::new();
    let mut failures = Vec::new();
    if let Err(error) = refusal::whole_public_output_rejects_late_second_body_failures() {
        failures.push(error);
    }
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
                    "state":"emitted","code":output.code(),"nativeMapRaw":output.source_map(),
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
                    "{id}: complete original source retained"
                );
                let view = mapped.observation().admitted().ok_or("setup owner")?;
                let setup = view.setup();
                let file = setup.file();
                require!(
                    file.is_complete()
                        && file.units().len() == 1
                        && file.scopes().len() == 2
                        && core::ptr::eq(
                            setup.syntax(),
                            setup.owner().retained_setup().ok_or("syntax")?
                        ),
                    "{id}: same original Program/File/scope without branch alias scope"
                );
                let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
                    return Err(cstr!("{id}: actual original root If"));
                };
                let [record] = file.native_conditionals() else {
                    return Err(cstr!("{id}: exactly one conditional owner"));
                };
                let NativeFileConditionalState::Admitted(node) = record.state() else {
                    return Err(cstr!("{id}: normally completed whole pair"));
                };
                require!(
                    node == NodeId::FIRST
                        && record.accepts(original)
                        && file
                            .native_conditional(node)
                            .is_some_and(|row| core::ptr::eq(row, record))
                        && record.scope() == setup.scope(),
                    "{id}: actual introducing If/NodeId/current scope join"
                );
                let [first, last] = original.branches.as_slice() else {
                    return Err(cstr!("{id}: exactly two actual branches"));
                };
                let Some(ExprRef::Js(condition)) = first.condition else {
                    return Err(cstr!("{id}: genuine first conditional AST"));
                };
                let otherwise = record.else_input().ok_or("original valueless Else")?;
                require!(
                    record.input().matches_expression(condition)
                        && last.condition.is_none()
                        && last.span == otherwise.operand().element_span()
                        && otherwise.operand().name_span().slice(source) == "v-else"
                        && core::ptr::eq(otherwise.operand().source_block().root_source(), source)
                        && first.span.start == original.span.start
                        && first.span.end == last.span.start
                        && last.span.end == original.span.end,
                    "{id}: exact original expression/valueless owner and adjacent full extents"
                );
                let binding = file
                    .binding(record.binding().ok_or("binding id")?)
                    .ok_or("binding")?;
                let declaration = binding.declaration().ok_or("declaration")?;
                let initial_branch = fixture["ifElseTextRuntime"]["initial"]["branch"]
                    .as_u64()
                    .ok_or("branch")?;
                require!(
                    setup.binding(binding).is_ok()
                        && declaration.is_direct_program()
                        && declaration.scope == setup.scope()
                        && declaration.unit == setup.unit()
                        && declaration.name.as_str() == text(fixture, "conditionBinding")?
                        && declaration.boolean_initializer() == Some(initial_branch == 0),
                    "{id}: original direct BooleanLiteral declaration/binding"
                );
                let analysis = build_native_selected_setup_dom_decisions(setup)
                    .map_err(|error| cstr!("{id}: {error:?}"))?;
                let dom = analysis.dom().ok_or("DOM")?;
                require!(
                    dom.unsupported().is_empty(),
                    "{id}: complete native DOM closure"
                );
                let expression = analysis.expression(node).ok_or("condition decisions")?;
                let resolution = expression.resolution();
                let table = resolution.table().ok_or("retained resolution")?;
                let [read] = expression.reads() else {
                    return Err(cstr!("{id}: sole current condition READ"));
                };
                let [occurrence] = table.occurrences() else {
                    return Err(cstr!("{id}: sole original resolver occurrence"));
                };
                let immutable = fixture["immutableBindings"].as_array().ok_or("immutable")?;
                let expected_kind = if immutable.contains(&fixture["conditionBinding"]) {
                    VueReadKind::SetupConst
                } else {
                    VueReadKind::SetupLet
                };
                require!(
                    core::ptr::eq(resolution.file(), file)
                        && resolution.node() == node
                        && resolution.scope() == Some(record.scope())
                        && read.binding().same_owner(binding)
                        && read.binding().id() == binding.id()
                        && read.kind() == expected_kind
                        && occurrence.usage == Usage::Read
                        && core::ptr::eq(read.occurrence(), occurrence)
                        && core::ptr::eq(table.expression().ast, condition.ast)
                        && core::ptr::eq(table.expression().source, condition.source)
                        && table.expression().span == condition.span
                        && core::ptr::eq(
                            table.expression().coordinates.ok_or("coordinates")?,
                            condition.coordinates.ok_or("condition coordinates")?
                        ),
                    "{id}: actual File/Scope/BindingId/READ/AST/source/coordinate identity"
                );
                let control = dom.conditional(node).ok_or("owned control")?;
                let [first_row, last_row] = control.branches() else {
                    return Err(cstr!("{id}: two completed control branches"));
                };
                require!(
                    core::ptr::eq(control.owner(), original.as_ref())
                        && core::ptr::eq(first_row.owner(), first)
                        && core::ptr::eq(last_row.owner(), last)
                        && first_row.discriminant().index() == 0
                        && last_row.discriminant().index() == 1
                        && first_row.condition() == Some(expression.value())
                        && last_row.condition().is_none()
                        && control.fallback() == ConditionalFallback::AuthoredElse,
                    "{id}: two actual source-order branch identities and authored Else"
                );
                let mut bodies = Vec::new();
                for (row, branch) in [(first_row, first), (last_row, last)] {
                    let [op @ Op::Element(element)] = branch.region.ops.as_slice() else {
                        return Err(cstr!("{id}: one real branch Element"));
                    };
                    let body = dom.node(row.root()).ok_or("actual body")?;
                    let branch_index = row.discriminant().index();
                    let expected_name = fixture["textBindings"]
                        .as_array()
                        .ok_or("text binding pair")?
                        .get(usize::try_from(branch_index).map_err(|error| cstr!("{error}"))?)
                        .ok_or("branch text binding")?;
                    let mut text_custody = Value::Null;
                    let literal = match element.children.ops.as_slice() {
                        [] if expected_name.is_null() => None,
                        [Op::Text(text)] if expected_name.is_null() => Some(text.content),
                        [text_op @ Op::Interpolation(interpolation)] => {
                            let name = expected_name.as_str().ok_or("original text binding")?;
                            let ExprRef::Js(actual) = interpolation.expression else {
                                return Err(cstr!("{id}: original text AST"));
                            };
                            let text_record = file.native_interpolations().iter().find(|record| {
                                matches!(record.state(), NativeFileInterpolationState::Admitted(node)
                                    if file.native_interpolation(node).is_some_and(|retained| core::ptr::eq(retained, *record))
                                        && record.input().operand().syntax().expression().is_some_and(|ast| core::ptr::eq(ast, actual.ast)))
                            }).ok_or("same actual text record")?;
                            let NativeFileInterpolationState::Admitted(text_node) =
                                text_record.state()
                            else {
                                return Err(cstr!("{id}: admitted text node"));
                            };
                            let syntax = text_record.input().operand().syntax();
                            let text_row = analysis.expression(text_node).ok_or("text decision")?;
                            let table = text_row.resolution().table().ok_or("text table")?;
                            let [occurrence] = table.occurrences() else {
                                return Err(cstr!("{id}: sole original text occurrence"));
                            };
                            let [text_read] = text_row.reads() else {
                                return Err(cstr!("{id}: sole original text READ"));
                            };
                            let text_binding = setup
                                .binding(text_read.binding())
                                .map_err(|error| cstr!("{error:?}"))?;
                            let declaration =
                                text_binding.declaration().ok_or("text declaration")?;
                            let retained = table.expression();
                            let dynamic = !immutable.contains(expected_name);
                            let expected_kind = if dynamic {
                                VueReadKind::SetupLet
                            } else {
                                VueReadKind::SetupConst
                            };
                            let DomChildren::Text(group) = &body.children else {
                                return Err(cstr!("{id}: actual singleton text group"));
                            };
                            require!(
                                text_node != node
                                    && !core::ptr::eq(condition.ast, actual.ast)
                                    && interpolation.span
                                        == text_record.input().operand().full_span()
                                    && core::ptr::eq(
                                        syntax.expression().ok_or("syntax root")?,
                                        actual.ast
                                    )
                                    && core::ptr::eq(syntax.source().text(), actual.source)
                                    && core::ptr::eq(syntax.source().authored_root(), source)
                                    && syntax.source().span() == actual.span
                                    && core::ptr::eq(text_row.resolution().file(), file)
                                    && text_row.resolution().node() == text_node
                                    && text_row.resolution().scope() == Some(setup.scope())
                                    && core::ptr::eq(retained.ast, actual.ast)
                                    && core::ptr::eq(retained.source, actual.source)
                                    && retained.span == actual.span
                                    && core::ptr::eq(
                                        retained.coordinates.ok_or("retained coordinates")?,
                                        actual.coordinates.ok_or("text coordinates")?
                                    )
                                    && occurrence.usage == Usage::Read
                                    && core::ptr::eq(occurrence, text_read.occurrence())
                                    && !core::ptr::eq(occurrence, read.occurrence())
                                    && occurrence.binding == text_binding.id()
                                    && text_read.binding().same_owner(text_binding)
                                    && declaration.is_direct_program()
                                    && declaration.unit == setup.unit()
                                    && declaration.scope == setup.scope()
                                    && declaration.initializer.is_primitive()
                                    && declaration.name.as_str() == name
                                    && occurrence.name == name
                                    && text_read.kind() == expected_kind
                                    && text_row.value()
                                        == if dynamic {
                                            vize_l3::decision::dom::ValueKind::FileDependent
                                        } else {
                                            vize_l3::decision::dom::ValueKind::LiteralConstant
                                        }
                                    && (text_binding.id() == binding.id())
                                        == (name == text(fixture, "conditionBinding")?)
                                    && matches!(group.nodes.as_slice(), [actual] if *actual == text_node)
                                    && group.dynamic == dynamic
                                    && body.changes.text == dynamic
                                    && dom
                                        .node(text_node)
                                        .is_some_and(|row| core::ptr::eq(row.op(), text_op)),
                                "{id}: same original File/scope/AST/coordinate/BindingId text owner and semantic facts"
                            );
                            let authored = actual
                                .authored_span(occurrence.span)
                                .ok_or("authored text READ")?;
                            text_custody = serde_json::json!({
                                "node":text_node.index(),"binding":text_binding.id().index(),
                                "scope":setup.scope().index(),"name":name,"readKind":cstr!("{:?}",text_read.kind()),
                                "valueKind":cstr!("{:?}",text_row.value()),"dynamic":dynamic,
                                "fullSpan":[interpolation.span.start,interpolation.span.end],
                                "expressionSpan":[actual.span.start,actual.span.end],"authoredReadSpan":[authored.start,authored.end],
                            });
                            None
                        }
                        _ => return Err(cstr!("{id}: one original text or static branch body")),
                    };
                    require!(
                        core::ptr::eq(body.op(), op)
                            && body.block_eligible
                            && element.attributes.is_empty()
                            && element.bindings.is_empty()
                            && (!expected_name.is_null() || !body.changes.text),
                        "{id}: real original branch Element/decision owner"
                    );
                    if row.discriminant().index() == 1 {
                        require!(
                            otherwise.body_node() == Some(row.root())
                                && otherwise.accepts_body(row.root(), element),
                            "{id}: retained Else owner joined to actual closed Element/NodeId"
                        );
                    }
                    bodies.push(serde_json::json!({
                        "node":row.root().index(),"discriminant":row.discriminant().index(),
                        "tag":element.tag,"literalText":literal,"span":[element.span.start,element.span.end],
                        "branchSpan":[branch.span.start,branch.span.end],"text":text_custody,
                    }));
                }
                let expected_text_count = fixture["textBindings"]
                    .as_array()
                    .ok_or("text bindings")?
                    .iter()
                    .filter(|name| !name.is_null())
                    .count();
                require!(
                    file.native_interpolations().len() == expected_text_count,
                    "{id}: exactly the original branch text occurrences"
                );
                if let [first, last] = file.native_interpolations() {
                    require!(
                        first.state() != last.state()
                            && !core::ptr::eq(
                                first
                                    .input()
                                    .operand()
                                    .syntax()
                                    .expression()
                                    .ok_or("first AST")?,
                                last.input()
                                    .operand()
                                    .syntax()
                                    .expression()
                                    .ok_or("last AST")?
                            ),
                        "{id}: two original ASTs/NodeIds even when both resolve to one binding"
                    );
                }
                for link in output.document().links() {
                    require!(
                        source
                            .get(link.authored.start as usize..link.authored.end as usize)
                            .is_some()
                            && output
                                .code()
                                .get(link.generated.start as usize..link.generated.end as usize)
                                .is_some(),
                        "{id}: complete UTF-8 authored/generated geometry"
                    );
                }
                let condition_span = condition
                    .authored_span(occurrence.span)
                    .ok_or("authored READ")?;
                Ok((
                    serde_json::json!({"id":id,"mode":mode_name,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw_map}),
                    serde_json::json!({
                        "id":id,"mode":mode_name,"ifNode":node.index(),"scope":record.scope().index(),
                        "binding":binding.id().index(),"booleanInitializer":declaration.boolean_initializer(),
                        "conditionReadKind":cstr!("{:?}",read.kind()),"conditionValueKind":cstr!("{:?}",expression.value()),
                        "conditionAuthoredSpan":[condition_span.start,condition_span.end],
                        "conditionNameSpan":[record.input().operand().name_span().start,record.input().operand().name_span().end],
                        "elseNameSpan":[otherwise.operand().name_span().start,otherwise.operand().name_span().end],
                        "wholeSpan":[original.span.start,original.span.end],"bodies":bodies,
                        "fallback":cstr!("{:?}",control.fallback()),
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
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_TEXT_SFC_DOM_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema":"vize.native-sfc.original-root-if-else-text-capture",
                "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if qualified { "current-complete-native-goldens-matched" } else { "failed-current-complete-native-goldens" },
            "nativeGoldenComparisonPerformed":true,
                "sourceRevision":std::env::var("GITHUB_SHA").ok(),
                "fixtures":captures,"custody":custody,"attempts":attempts,"failures":&failures,
            }))
            .map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    require!(
        failures.is_empty(),
        "original If/Else capture failures: {failures:?}"
    );
    require!(
        emitted == fixtures.len() * 2,
        "all twelve mode-major original pairs emitted"
    );
    Ok(())
}

//! Whole original root If observations, with explicit target mode and no guessed goldens.
use super::*;
use vize_l2::{expr::ExprRef, file::NativeFileConditionalState, op::Op};

#[test]
fn eighteen_whole_original_root_if_components_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_root_if_inputs_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let goldens: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_root_if_sfc_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    require!(
        goldens["schema"] == "vize.native-sfc.original-root-if-complete-goldens"
            && goldens["filename"] == pack["filename"],
        "complete current root If golden authority"
    );
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(
        fixtures.len() == 9,
        "nine input-only whole original sources"
    );
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
                        "{id}: default development entry preserves every code/map byte"
                    );
                }
                let raw_map = output.source_map().ok_or("complete raw map")?;
                let map: Value = serde_json::from_str(raw_map).map_err(|error| cstr!("{error}"))?;
                require!(
                    map["sourcesContent"] == serde_json::json!([source]),
                    "{id}: original whole source"
                );
                let view = mapped
                    .observation()
                    .admitted()
                    .ok_or("original setup owner")?;
                let setup = view.setup();
                let file = setup.file();
                let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
                    return Err(cstr!("{id}: actual single original If"));
                };
                let [record] = file.native_conditionals() else {
                    return Err(cstr!("{id}: one original conditional record"));
                };
                let NativeFileConditionalState::Admitted(node) = record.state() else {
                    return Err(cstr!("{id}: normal completed If attachment"));
                };
                require!(
                    record.accepts(original)
                        && file
                            .native_conditional(node)
                            .is_some_and(|actual| core::ptr::eq(actual, record)),
                    "{id}: same actual If allocation and node"
                );
                require!(
                    record.scope() == setup.unit_record().scope && file.scopes().len() == 2,
                    "{id}: current actual setup scope with no new alias scope"
                );
                let binding = file
                    .binding(record.binding().ok_or("actual binding")?)
                    .ok_or("owned binding")?;
                let declaration = binding.declaration().ok_or("original script declaration")?;
                require!(
                    setup.binding(binding).is_ok()
                        && declaration.is_direct_program()
                        && declaration.scope == record.scope()
                        && declaration.boolean_initializer()
                            == fixture["ifRuntime"]["initial"].as_bool(),
                    "{id}: actual direct BooleanLiteral declaration event"
                );
                let [branch] = original.branches.as_slice() else {
                    return Err(cstr!("{id}: one original branch"));
                };
                let Some(ExprRef::Js(condition)) = branch.condition else {
                    return Err(cstr!("{id}: original condition AST"));
                };
                require!(
                    record.input().matches_expression(condition)
                        && record
                            .input()
                            .expression()
                            .is_some_and(|actual| core::ptr::eq(actual, condition)),
                    "{id}: genuine transferred header wrapper"
                );
                let analysis = build_native_selected_setup_dom_decisions(setup)
                    .map_err(|error| cstr!("{id}: {error:?}"))?;
                let dom = analysis.dom().ok_or("DOM")?;
                require!(
                    dom.unsupported().is_empty(),
                    "{id}: complete runtime closure"
                );
                let expression = analysis
                    .expression(node)
                    .ok_or("current condition decisions")?;
                let resolution = expression.resolution();
                let [read] = expression.reads() else {
                    return Err(cstr!("{id}: one complete read"));
                };
                let table = resolution.table().ok_or("whole resolution")?;
                let [occurrence] = table.occurrences() else {
                    return Err(cstr!("{id}: one actual resolver occurrence"));
                };
                require!(
                    core::ptr::eq(resolution.file(), file)
                        && resolution.node() == node
                        && resolution.scope() == Some(record.scope())
                        && read.binding().same_owner(binding)
                        && read.binding().id() == binding.id()
                        && core::ptr::eq(read.occurrence(), occurrence),
                    "{id}: same File/scope/binding/original occurrence"
                );
                let control = dom.conditional(node).ok_or("owned control")?;
                let [row] = control.branches() else {
                    return Err(cstr!("{id}: one completed control branch"));
                };
                require!(
                    core::ptr::eq(control.owner(), original.as_ref())
                        && core::ptr::eq(row.owner(), branch)
                        && row.discriminant().index() == 0
                        && row.condition() == Some(expression.value()),
                    "{id}: genuine source-order condition facts"
                );
                let body = dom.node(row.root()).ok_or("actual branch root")?;
                require!(
                    body.block_eligible
                        && matches!(body.op(), Op::Element(element)
                    if Some(element.tag) == fixture["ifRuntime"]["tag"].as_str() && element.attributes.is_empty()),
                    "{id}: direct original native element branch"
                );
                Ok((
                    serde_json::json!({"id":id,"mode":mode_name,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw_map}),
                    serde_json::json!({"id":id,"mode":mode_name,"node":node.index(),"scope":record.scope().index(),
                        "binding":binding.id().index(),"booleanInitializer":declaration.boolean_initializer(),
                        "operandNameSpan":[record.input().operand().name_span().start,record.input().operand().name_span().end],
                        "operandValueSpan":[record.input().operand().value_span().start,record.input().operand().value_span().end],
                        "readKind":cstr!("{:?}",read.kind()),"valueKind":cstr!("{:?}",expression.value()),
                        "bodyNode":row.root().index(),"discriminant":row.discriminant().index(),
                        "fallback":cstr!("{:?}",control.fallback())}),
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
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_ROOT_IF_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-root-if-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if qualified { "current-complete-native-goldens-matched" } else { "failed-current-complete-native-goldens" },
            "nativeGoldenComparisonPerformed":true,
            "sourceRevision":std::env::var("GITHUB_SHA").ok(),"fixtures":captures,"custody":custody,"attempts":attempts,"failures":&failures,
        })).map_err(|error| cstr!("{error}"))?).map_err(|error| cstr!("{error}"))?;
    }
    require!(
        failures.is_empty(),
        "complete original If capture failed: {failures:?}"
    );
    require!(
        emitted == fixtures.len() * 2,
        "both actual modes emitted every whole input"
    );
    Ok(())
}

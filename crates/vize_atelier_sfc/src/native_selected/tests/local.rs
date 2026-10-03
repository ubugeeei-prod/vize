use super::*;

mod refusal;

fn all_on<'o, 'a>(region: &'o Region<'a>, result: &mut Vec<&'o OnOp<'a>>) {
    for op in &region.ops {
        if let Op::Element(element) = op {
            for binding in &element.bindings {
                if let BindingOp::On(on) = binding {
                    result.push(on);
                }
            }
            all_on(&element.children, result);
        }
    }
}

fn utf16_span(text: &str, span: vize_l0::Span) -> Result<Value, String> {
    let begin = text.get(..span.start as usize).ok_or("UTF-8 start")?;
    let end = text.get(..span.end as usize).ok_or("UTF-8 end")?;
    Ok(serde_json::json!([
        begin.encode_utf16().count(),
        end.encode_utf16().count()
    ]))
}

#[test]
fn ten_original_block_local_components_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_handler_local_sfc_click_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let fixtures = pack
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or("fixtures")?;
    require!(fixtures.len() == 10, "ten original local components");
    let mut rows = Vec::new();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let plain =
            compile_native_selected_sfc_dom(&arena, source, NativeSelectedSfcDomOptions::default());
        let mapped = compile_native_selected_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                filename: "HandlerLocal雪🌸.vue",
                source_map: true,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        let output = mapped.result().map_err(|error| cstr!("{id}: {error:?}"))?;
        let unlinked = plain
            .result()
            .map_err(|error| cstr!("{id}: plain {error:?}"))?;
        require!(
            output.code() == text(fixture, "expectedCode")?,
            "{id}: full independent pinned component"
        );
        require!(
            unlinked.code() == output.code(),
            "{id}: NoLinks/Recorded equality"
        );
        require!(
            unlinked.document().links().is_empty() && unlinked.source_map().is_none(),
            "{id}: NoLinks"
        );
        let map: Value = serde_json::from_str(output.source_map().ok_or("map")?)
            .map_err(|error| cstr!("{error}"))?;
        require!(
            map["sourcesContent"] == serde_json::json!([source]),
            "{id}: whole owner map source"
        );
        // Capture preparation only: mandatory Node acceptance rejects unready
        // maps. Remove this branch after freezing the first actual hosted pack.
        if !fixture["nativeMap"].is_null() {
            require!(
                map == fixture["nativeMap"],
                "{id}: complete frozen map equality"
            );
        }
        let original = mapped
            .observation()
            .admitted()
            .ok_or("whole owner")?
            .into_template_view();
        let file = original.file().ok_or("actual File")?;
        require!(
            file.is_complete() && file.units().is_empty(),
            "{id}: complete scriptless File"
        );
        require!(
            core::ptr::eq(file.artifact().source(), source),
            "{id}: original root"
        );
        let mut handlers = Vec::new();
        all_on(file.artifact().root(), &mut handlers);
        let facts = fixture
            .get("bodyFacts")
            .and_then(Value::as_array)
            .ok_or("bodyFacts")?;
        require!(handlers.len() == facts.len(), "{id}: every actual handler");
        for (on, expected) in handlers.iter().zip(facts) {
            let handler = file.handler_for(on).ok_or("actual On allocation")?;
            require!(handler.accepts_on(on), "{id}: original checked On");
            let resolution = handler.resolution().ok_or("whole HandlerResolution")?;
            let syntax = resolution.input().operand().syntax();
            let source_window = syntax.source();
            require!(
                core::ptr::eq(source_window.authored_root(), source),
                "{id}: actual handler source"
            );
            require!(
                source_window.text() == text(expected, "decoded")?,
                "{id}: whole original decoded body"
            );
            require!(
                syntax.diagnostics().count() == 0,
                "{id}: admitted stock body"
            );
            let references = expected
                .get("references")
                .and_then(Value::as_array)
                .ok_or("references")?;
            require!(
                resolution.references().len() == references.len(),
                "{id}: complete recorded references"
            );
            let span = source_window.span();
            let links: Vec<_> = output
                .document()
                .links()
                .iter()
                .filter(|link| link.authored.start >= span.start && link.authored.end <= span.end)
                .collect();
            require!(
                links
                    .first()
                    .is_some_and(|link| link.authored.start == span.start)
                    && links
                        .last()
                        .is_some_and(|link| link.authored.end == span.end),
                "{id}: entire body boundaries"
            );
            require!(
                links
                    .windows(2)
                    .all(|pair| pair[0].authored.end == pair[1].authored.start),
                "{id}: contiguous original body"
            );
            let recovered: String = links
                .iter()
                .map(|link| link.generated.slice(output.code()))
                .collect();
            require!(
                recovered == source_window.text(),
                "{id}: unchanged whole body"
            );
            for (reference, expected) in resolution.references().iter().zip(references) {
                require!(
                    reference.name == text(expected, "name")?,
                    "{id}: original local/event name"
                );
                require!(
                    utf16_span(source_window.text(), reference.span)? == expected["span"],
                    "{id}: independent AST/code-unit reference geometry"
                );
                let authored = resolution
                    .authored_span(reference.span)
                    .map_err(|error| cstr!("{error:?}"))?;
                require!(
                    links.iter().any(|link| link.authored == authored
                        && link.name.as_deref() == Some(reference.name)
                        && link.generated.slice(output.code()) == reference.name),
                    "{id}: original named local reference"
                );
            }
            if let Some(decode_map) = source_window.decode_map() {
                for segment in decode_map
                    .segments()
                    .iter()
                    .filter(|segment| segment.authored().slice(source).starts_with('&'))
                {
                    require!(
                        links.iter().any(|link| link.authored == segment.authored()
                            && link.name.is_none()
                            && link.generated.slice(output.code())
                                == segment.decoded().slice(source_window.text())),
                        "{id}: complete original entity atom"
                    );
                }
            }
        }
        if let (Some(first), Some(second)) = (handlers.first(), handlers.get(1)) {
            let first = file.handler_for(first).ok_or("first On")?;
            let second = file.handler_for(second).ok_or("second On")?;
            require!(
                first.id() != second.id(),
                "{id}: separate introducing nodes"
            );
            require!(
                !core::ptr::eq(
                    first.resolution().ok_or("first")?.input().body(),
                    second.resolution().ok_or("second")?.input().body()
                ),
                "{id}: separate full original body allocations"
            );
            require!(
                !first.accepts_on(handlers.get(1).ok_or("second original On")?),
                "{id}: equal local IDs do not merge handlers"
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
                "{id}: full UTF-8 link coverage"
            );
        }
        rows.push(
            serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map}),
        );
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_CAPTURE") {
        let capture = serde_json::json!({"schema":"vize.native-sfc.handler-local-click-capture","adapter":"vize_atelier_sfc::compile_native_selected_sfc_dom","fixtures":rows});
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    Ok(())
}

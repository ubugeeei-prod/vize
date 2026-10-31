//! Genuine new ForKey captures; immutable older For packs stay separate.
use super::*;
use vize_l2::{op::Op, resolution::ForAliasRole};

#[test]
fn nine_whole_original_key_sources_capture_real_bindings_and_maps() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_for_key_inputs_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let goldens: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_for_key_sfc_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(fixtures.len() == 9, "nine new original key inputs");
    let mut captures = Vec::new();
    let mut custody = Vec::new();
    let mut attempts = Vec::new();
    let mut failures = Vec::new();
    if goldens["schema"] != "vize.native-sfc.original-for-key-complete-goldens" {
        failures.push(cstr!("complete native golden schema"));
    }
    let native = &goldens["native"];
    for field in ["fixtures", "custody", "attempts"] {
        if native[field].as_array().map(Vec::len) != Some(fixtures.len()) {
            failures.push(cstr!("complete native {field}: nine ordered rows"));
        }
    }
    for (index, fixture) in fixtures.iter().enumerate() {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let mapped = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                filename: text(&pack, "filename")?,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        let plain = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions::default(),
        );
        let observe = |compilation: &NativeSelectedSetupSfcDomCompilation<'_>| match compilation
            .result()
        {
            Ok(output) => serde_json::json!({
                "state":"emitted", "code":output.code(), "nativeMapRaw":output.source_map(),
                "nativeMap":output.source_map().and_then(|raw| serde_json::from_str::<Value>(raw).ok()),
            }),
            Err(error) => serde_json::json!({"state":"refused", "error":cstr!("{error:?}")}),
        };
        attempts.push(serde_json::json!({
            "id":id, "source":source, "mapped":observe(&mapped), "plain":observe(&plain),
        }));
        let validated = (|| -> Result<(Value, Value), String> {
            let output = mapped.result().map_err(|error| cstr!("{id}: {error:?}"))?;
            let unlinked = plain
                .result()
                .map_err(|error| cstr!("{id}: plain {error:?}"))?;
            require!(
                output.code() == unlinked.code()
                    && unlinked.source_map().is_none()
                    && unlinked.document().links().is_empty(),
                "{id}: complete NoLinks output equality"
            );
            let raw_map = output.source_map().ok_or("complete raw map")?;
            let map: Value = serde_json::from_str(raw_map).map_err(|error| cstr!("{error}"))?;
            require!(
                map["sourcesContent"] == serde_json::json!([source]),
                "{id}: whole source"
            );
            let view = mapped.observation().admitted().ok_or("original owner")?;
            let setup = view.setup();
            let file = setup.file();
            let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
                return Err(cstr!("{id}: original For root"));
            };
            let head = file
                .for_head_for(original)
                .ok_or("attached original head")?;
            let resolution = head.resolution().ok_or("whole head resolution")?;
            let value = head.value().ok_or("actual value declaration")?;
            let key = head.key().ok_or("actual key declaration")?;
            let declaration = key.template_declaration().ok_or("template declaration")?;
            let key_original = declaration
                .declaration()
                .original()
                .ok_or("whole key parameter")?;
            require!(
                key_original.fact().role() == ForAliasRole::Key
                    && declaration.declaration().origin() == head.id()
                    && Some(declaration.declaration().scope()) == head.scope()
                    && key.id() != value.id()
                    && key.id() != resolution.collection().binding
                    && core::ptr::eq(key_original.resolution(), resolution)
                    && key.declaration().is_none()
                    && setup.binding(key).is_err(),
                "{id}: actual key identity is neither value nor outer ScriptUnit"
            );
            let analysis = build_native_selected_setup_dom_decisions(setup)
                .map_err(|error| cstr!("{id}: {error:?}"))?;
            let dom = analysis.dom().ok_or("DOM")?;
            require!(
                dom.unsupported().is_empty(),
                "{id}: complete key runtime closure"
            );
            let row = dom
                .file_for_head(original.id().node())
                .ok_or("same For row")?;
            require!(
                row.accepts_original(original)
                    && head
                        .scope_facts()
                        .is_some_and(|facts| core::ptr::eq(row.scope_facts(), facts)),
                "{id}: canonical binding row belongs to the original File"
            );
            let [interpolation] = file.native_interpolations() else {
                return Err(cstr!("{id}: one actual key interpolation"));
            };
            let vize_l2::file::NativeFileInterpolationState::Admitted(node) = interpolation.state()
            else {
                return Err(cstr!("{id}: attached original interpolation"));
            };
            let expression = analysis.expression(node).ok_or("key expression")?;
            let [read] = expression.reads() else {
                return Err(cstr!("{id}: one actual key read"));
            };
            let body_resolution = expression.resolution();
            let table = body_resolution.table().ok_or("resolution")?;
            let [occurrence] = table.occurrences() else {
                return Err(cstr!("{id}: one original occurrence"));
            };
            require!(
                read.kind() == VueReadKind::ForKey
                    && read.binding().id() == key.id()
                    && read.binding().same_owner(key)
                    && core::ptr::eq(read.occurrence(), occurrence)
                    && expression.resolution().scope() == head.scope(),
                "{id}: actual current ForKey read and lexical scope"
            );
            let capture = serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw_map});
            let owner = serde_json::json!({"id":id,"node":head.id().node().index(),"scope":head.scope().ok_or("scope")?.index(),
                    "enclosingScope":head.enclosing_scope().ok_or("parent")?.index(),"valueBinding":value.id().index(),
                    "keyBinding":key.id().index(),"keyName":key_original.fact().name(),
                    "keyAuthoredSpan":[key_original.authored_span().start,key_original.authored_span().end],
                    "readKind":"ForKey","canonicalTag":row.scope_facts().tag.index()});
            if native["fixtures"].get(index) != Some(&capture) {
                failures.push(cstr!(
                    "{id}: whole native code/raw-map capture golden mismatch"
                ));
            }
            if native["custody"].get(index) != Some(&owner) {
                failures.push(cstr!("{id}: whole native owner golden mismatch"));
            }
            // Keep the genuine current observations even on a golden mismatch.
            // Only the final zero-failure verdict grants acceptance.
            Ok((capture, owner))
        })();
        match validated {
            Ok((capture, owner)) => {
                captures.push(capture);
                custody.push(owner);
            }
            Err(error) => failures.push(error),
        }
    }
    let emitted = captures.len();
    let attempts = Value::Array(attempts);
    if native.get("attempts") != Some(&attempts) {
        failures.push(cstr!("whole ordered mapped/plain attempt golden mismatch"));
    }
    let qualified = failures.is_empty() && emitted == 9;
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-for-key-capture", "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
            "nativeGoldenComparisonPerformed":true,
            "qualification":if qualified { "current-complete-native-goldens-matched" } else { "failed-current-complete-native-goldens" },
            "sourceRevision":std::env::var("GITHUB_SHA").ok(),"fixtures":captures,"custody":custody,"attempts":attempts,
            "failures":&failures,
        })).map_err(|error| cstr!("{error}"))?).map_err(|error| cstr!("{error}"))?;
    }
    require!(failures.is_empty(), "complete capture failed: {failures:?}");
    require!(
        emitted == fixtures.len(),
        "every new source was genuinely emitted"
    );
    Ok(())
}

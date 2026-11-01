//! First genuine original input Model capture; current whole goldens are pending.

use super::*;

mod custody;
mod refusal;

#[test]
fn fourteen_whole_original_input_model_components_and_raw_maps_are_captured() -> Result<(), String>
{
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    require!(
        pack["schema"] == "vize.native-sfc.original-input-model-inputs",
        "new original input authority"
    );
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(fixtures.len() == 7, "seven NEW full original source inputs");
    let mut captures = Vec::new();
    let mut owners = Vec::new();
    let mut attempts = Vec::new();
    let (refusals, mut failures) = refusal::capture();
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
                    "nativeMap":output.source_map().and_then(|raw|serde_json::from_str::<Value>(raw).ok()),
                }),
                Err(error) => serde_json::json!({"state":"refused","error":cstr!("{error:?}")}),
            };
            attempts.push(serde_json::json!({
                "id":id,"mode":mode_name,"source":source,
                "mapped":observe(&mapped),"plain":observe(&plain),
                "defaultDevelopment":default.as_ref().map(observe),
            }));
            let checked = (|| -> Result<(Value, Value), String> {
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
                    "{id}: complete NoLinks byte equality"
                );
                if let Some(default) = &default {
                    let default = default
                        .result()
                        .map_err(|error| cstr!("{id}: default {error:?}"))?;
                    require!(
                        default.code() == output.code()
                            && default.source_map() == output.source_map(),
                        "{id}: default-development whole code/map equality"
                    );
                }
                let raw = output.source_map().ok_or("whole raw map")?;
                let map: Value = serde_json::from_str(raw).map_err(|error| cstr!("{error}"))?;
                require!(
                    map["sourcesContent"] == serde_json::json!([source]),
                    "{id}: original complete source map"
                );
                let owner = custody::capture(&mapped, fixture, mode_name)?;
                Ok((
                    serde_json::json!({"id":id,"mode":mode_name,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw}),
                    owner,
                ))
            })();
            match checked {
                Ok((capture, owner)) => {
                    captures.push(capture);
                    owners.push(owner);
                }
                Err(error) => failures.push(error),
            }
        }
    }
    let complete = failures.is_empty() && captures.len() == fixtures.len() * 2;
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-input-model-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if complete {"observed-unqualified-current-oracles-pending"} else {"failed-current-source-observation"},
            "nativeGoldenComparisonPerformed":false,"sourceRevision":std::env::var("GITHUB_SHA").ok(),
            "fixtures":captures,"custody":owners,"attempts":attempts,"refusals":refusals,"failures":failures,
        })).map_err(|error|cstr!("{error}"))?).map_err(|error|cstr!("{error}"))?;
    }
    require!(
        complete,
        "all fourteen source/mode Models complete without golden credit: {failures:?}"
    );
    Ok(())
}

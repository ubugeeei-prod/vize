//! Genuine source-built SSR Models with complete independently reviewed oracles.

use super::*;
mod custody;
mod finish;
mod refusal;

#[test]
fn seven_whole_original_input_model_ssr_components_and_raw_maps_are_captured() -> Result<(), String>
{
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let fixtures = pack["fixtures"]
        .as_array()
        .ok_or("seven original input fixtures")?;
    require!(
        fixtures.len() == 7,
        "exact seven original inputs; SSR is a new target observation"
    );
    let mut captures = Vec::new();
    let mut owners = Vec::new();
    let mut attempts = Vec::new();
    let (refusals, mut failures) = refusal::capture();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let options = NativeSsrSfcCompileOptions {
            source_map: true,
            filename: text(&pack, "filename")?,
            ..NativeSsrSfcCompileOptions::default()
        };
        let mapped = compile_native_setup_ssr_sfc(&arena, source, options);
        let plain = compile_native_setup_ssr_sfc(
            &arena,
            source,
            NativeSsrSfcCompileOptions {
                source_map: false,
                ..options
            },
        );
        let default =
            compile_native_setup_ssr_sfc(&arena, source, NativeSsrSfcCompileOptions::default());
        let observe = |compilation: &NativeSetupSsrSfcCompilation<'_>| match compilation.result() {
            Ok(output) => serde_json::json!({"state":"emitted","code":output.code(),
                "nativeMapRaw":output.source_map(),
                "nativeMap":output.source_map().and_then(|raw|serde_json::from_str::<Value>(raw).ok())}),
            Err(error) => serde_json::json!({"state":"refused","error":cstr!("{error:?}")}),
        };
        attempts.push(
            serde_json::json!({"id":id,"source":source,"mapped":observe(&mapped),
            "plain":observe(&plain),"defaultOptions":observe(&default)}),
        );
        let checked = (|| -> Result<(Value, Value), String> {
            let output = mapped.result().map_err(|error| cstr!("{id}: {error:?}"))?;
            let unlinked = plain
                .result()
                .map_err(|error| cstr!("{id}: plain {error:?}"))?;
            let default = default
                .result()
                .map_err(|error| cstr!("{id}: default {error:?}"))?;
            require!(
                output.code() == unlinked.code()
                    && unlinked.source_map().is_none()
                    && unlinked.document().links().is_empty(),
                "{id}: complete NoLinks equality"
            );
            require!(
                output.code() == default.code()
                    && default.source_map().is_none()
                    && default.document().links().is_empty(),
                "{id}: original default-options equality"
            );
            let raw = output.source_map().ok_or("complete original map")?;
            let map: Value = serde_json::from_str(raw).map_err(|error| cstr!("{error}"))?;
            require!(
                map["sourcesContent"] == serde_json::json!([source]),
                "{id}: original complete source"
            );
            let owner = custody::capture(&mapped, fixture)?;
            Ok((
                serde_json::json!({"id":id,"source":source,"code":output.code(),
                "nativeMapRaw":raw,"nativeMap":map}),
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
    finish::write(&pack, captures, owners, attempts, refusals, failures)
}

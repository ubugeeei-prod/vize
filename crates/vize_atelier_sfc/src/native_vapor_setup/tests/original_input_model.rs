//! First genuine Vapor observations retain all attempts before any verdict.

use super::*;
use sha2::{Digest, Sha256};
mod custody;
mod finish;
pub(crate) mod refusal;

#[test]
fn seven_whole_original_input_model_vapor_components_and_raw_maps_are_captured()
-> Result<(), String> {
    let input_raw =
        include_str!("../../../tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json");
    let pack: Value = serde_json::from_str(input_raw).map_err(|error| cstr!("{error}"))?;
    let fixtures = pack["fixtures"]
        .as_array()
        .ok_or("seven original fixtures")?;
    require!(
        fixtures.len() == 7,
        "seven unchanged inputs, new Vapor target"
    );
    let mut captures = Vec::new();
    let mut owners = Vec::new();
    let mut attempts = Vec::new();
    let (refusals, mut failures) = refusal::capture();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let options = NativeVaporSfcCompileOptions {
            source_map: true,
            filename: text(&pack, "filename")?,
            ..Default::default()
        };
        let mapped = compile_native_vapor_setup_sfc(&arena, source, options);
        let plain = compile_native_vapor_setup_sfc(
            &arena,
            source,
            NativeVaporSfcCompileOptions {
                source_map: false,
                ..options
            },
        );
        let default = compile_native_vapor_setup_sfc(&arena, source, Default::default());
        let observe = |compilation: &NativeVaporSetupSfcCompilation<'_>| match compilation.result()
        {
            Ok(output) => serde_json::json!({"state":"emitted","code":output.code(),
                "nativeMapRaw":output.source_map(),
                "nativeMap":output.source_map().and_then(|raw|serde_json::from_str::<Value>(raw).ok())}),
            Err(error) => serde_json::json!({"state":"refused","error":cstr!("{error:?}")}),
        };
        attempts.push(serde_json::json!({"id":id,"source":source,
            "mapped":observe(&mapped),"plain":observe(&plain),"defaultOptions":observe(&default)}));
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
                    && output.code() == default.code()
                    && unlinked.source_map().is_none()
                    && default.source_map().is_none()
                    && unlinked.document().links().is_empty()
                    && default.document().links().is_empty(),
                "{id}: whole mapped/plain/default code equality"
            );
            let raw = output.source_map().ok_or("complete original raw map")?;
            let map: Value = serde_json::from_str(raw).map_err(|error| cstr!("{error}"))?;
            require!(
                map["sourcesContent"] == serde_json::json!([source]),
                "{id}: whole original map source"
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
    let complete = failures.is_empty()
        && captures.len() == 7
        && owners.len() == 7
        && attempts.len() == 7
        && refusals.len() == 12;
    let digest = Sha256::digest(input_raw.as_bytes());
    let mut input_hash = String::with_capacity(64);
    for nibble in digest.iter().flat_map(|byte| [byte >> 4, byte & 0x0f]) {
        input_hash.push(char::from_digit(u32::from(nibble), 16).ok_or("SHA256 nibble")?);
    }
    let capture = serde_json::json!({
        "schema":"vize.native-vapor.original-input-model-capture",
        "adapter":"compile_native_vapor_setup_sfc",
        "sourceRevision":std::env::var("GITHUB_SHA").ok(),
        "inputHash":input_hash,"inputPack":pack,
        "target":{"compiler":"@vue/compiler-sfc@3.6.0-rc.9","runtime":"vue@3.6.0-rc.9"},
        "qualification":"observed-unqualified-current-oracles-pending",
        "nativeGoldenComparisonPerformed":false,"wholePacketQualification":false,
        "failures":failures,"fixtures":captures,"custody":owners,"attempts":attempts,"refusals":refusals,
    });
    finish::write(capture, complete)
}

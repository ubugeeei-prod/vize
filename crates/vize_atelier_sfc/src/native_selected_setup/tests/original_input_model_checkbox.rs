//! Three original checkbox inputs; the first full capture is not a golden.

use crate::{
    NativeSelectedSfcDomOptions, NativeSsrSfcCompileOptions, NativeVaporSfcCompileOptions,
    compile_native_selected_setup_sfc_dom, compile_native_selected_setup_sfc_dom_with_mode,
    compile_native_setup_ssr_sfc, compile_native_vapor_setup_sfc,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_l0::Allocator;
use vize_l4::targets::dom::DomMode;

mod custody;
mod finish;
mod output;
mod refusal;

#[test]
fn three_original_checkbox_models_retain_all_target_mode_attempts_before_verdict()
-> Result<(), String> {
    let raw =
        include_str!("../../../tests/fixtures/native_original_input_model_checkbox_inputs.json");
    let pack: Value = serde_json::from_str(raw).map_err(|error| format!("{error}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("three original inputs")?;
    if fixtures.len() != 3 {
        return Err("three fixed original inputs".into());
    }
    let mut captures = Vec::new();
    let mut owners = Vec::new();
    let mut attempts = Vec::new();
    let (refusals, mut failures) = refusal::capture();
    let refusal_pack: Value =
        serde_json::from_str(refusal::INPUTS).map_err(|error| format!("{error}"))?;
    for target in ["dom", "ssr", "vapor"] {
        for (mode, mode_name) in [
            (DomMode::Development, "development"),
            (DomMode::Production, "production"),
        ] {
            for fixture in fixtures {
                let groups = pack["groups"].as_array().ok_or("checkbox source group")?;
                let group = groups
                    .iter()
                    .find(|group| {
                        group["ids"]
                            .as_array()
                            .is_some_and(|ids| ids.contains(&fixture["id"]))
                    })
                    .ok_or("original source group")?;
                let filename = group["filename"]
                    .as_str()
                    .ok_or("original group filename")?;
                let kind = group["kind"].as_str().ok_or("original input kind")?;
                let source = fixture["source"].as_str().ok_or("source")?;
                let arena = Allocator::default();
                let captured = match target {
                    "dom" => {
                        let options = NativeSelectedSfcDomOptions {
                            source_map: true,
                            filename,
                            ..Default::default()
                        };
                        let mapped = compile_native_selected_setup_sfc_dom_with_mode(
                            &arena, source, options, mode,
                        );
                        let plain = compile_native_selected_setup_sfc_dom_with_mode(
                            &arena,
                            source,
                            NativeSelectedSfcDomOptions {
                                source_map: false,
                                ..options
                            },
                            mode,
                        );
                        let default = (mode == DomMode::Development).then(|| {
                            compile_native_selected_setup_sfc_dom(
                                &arena,
                                source,
                                Default::default(),
                            )
                        });
                        output::capture(
                            fixture,
                            target,
                            mode_name,
                            mapped.result(),
                            plain.result(),
                            default.as_ref().map(|compilation| compilation.result()),
                            mapped.observation(),
                            kind,
                        )
                    }
                    "ssr" => {
                        let options = NativeSsrSfcCompileOptions {
                            source_map: true,
                            filename,
                            ..Default::default()
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
                            compile_native_setup_ssr_sfc(&arena, source, Default::default());
                        output::capture(
                            fixture,
                            target,
                            mode_name,
                            mapped.result(),
                            plain.result(),
                            Some(default.result()),
                            mapped.observation(),
                            kind,
                        )
                    }
                    "vapor" => {
                        let options = NativeVaporSfcCompileOptions {
                            source_map: true,
                            filename,
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
                        let default =
                            compile_native_vapor_setup_sfc(&arena, source, Default::default());
                        output::capture(
                            fixture,
                            target,
                            mode_name,
                            mapped.result(),
                            plain.result(),
                            Some(default.result()),
                            mapped.observation(),
                            kind,
                        )
                    }
                    _ => return Err("closed target set".into()),
                };
                attempts.push(captured.attempt);
                match captured.result {
                    Ok((capture, owner)) => {
                        captures.push(capture);
                        owners.push(owner);
                    }
                    Err(error) => failures.push(error),
                }
            }
        }
    }
    if core::mem::size_of::<vize_l2::file::Declaration>() != 96 {
        failures.push("original Declaration layout must stay96".into());
    }
    let complete = failures.is_empty()
        && captures.len() == 18
        && owners.len() == 18
        && attempts.len() == 18
        && refusals.len() == 48;
    let input_hash = Sha256::digest(raw.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let capture = json!({
        "schema":"vize.native-sfc.original-input-model-checkbox-capture",
        "adapters":["compile_native_selected_setup_sfc_dom_with_mode","compile_native_setup_ssr_sfc","compile_native_vapor_setup_sfc"],
        "sourceRevision":std::env::var("GITHUB_SHA").ok(),"inputHash":input_hash,"inputPack":pack,
        "qualification":"observed-unqualified-current-oracles-pending",
        "nativeGoldenComparisonPerformed":false,"wholePacketQualification":false,
        "nativeCaptureComplete":complete,"failures":failures,
        "fixtures":captures,"custody":owners,"attempts":attempts,"refusals":refusals,
        "refusalInputPack":refusal_pack,
        "layouts":{"declaration":core::mem::size_of::<vize_l2::file::Declaration>(),
            "nativeModel":core::mem::size_of::<vize_l2::file::NativeFileModel<'_>>(),
            "modelOp":core::mem::size_of::<vize_l2::op::ModelOp<'_>>()},
    });
    finish::write(capture, complete)
}

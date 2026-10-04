use super::{custody, inputs};
use inputs::{FILENAME, Fixture, Inputs, sha256};
use serde_json::{Value, json};
use vize_atelier_sfc::{
    NativeSelectedSfcDomOptions, NativeSsrSfcCompileOptions, NativeVaporSfcCompileOptions,
    compile_native_selected_sfc_dom, compile_native_ssr_sfc, compile_native_vapor_sfc,
};
use vize_l0::{Allocator, String, ToCompactString, cstr};
use vize_l1_to_l2::native_file::{NativeSelectedSfcIssueKind, NativeSelectedSfcObservation};
use vize_l4::write::EmitDocument;

#[derive(Clone, Copy)]
pub enum Target {
    Dom,
    Ssr,
    Vapor,
}

impl Target {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dom => "dom",
            Self::Ssr => "ssr",
            Self::Vapor => "vapor",
        }
    }
    pub fn version(self) -> &'static str {
        match self {
            Self::Dom | Self::Ssr => "3.5.35",
            Self::Vapor => "3.6.0-rc.9",
        }
    }
}

pub fn packet(inputs: &Inputs) -> Value {
    let mut rows = Vec::new();
    for fixture in &inputs.fixtures {
        for target in [Target::Dom, Target::Ssr, Target::Vapor] {
            for source_map in [true, false] {
                // Each outcome owns a genuinely fresh buffer, arena and API call.
                let source = cstr!("<template>{}</template>", fixture.source);
                rows.push(compile_row(fixture, target, source_map, &source, None));
            }
        }
    }
    let positive = rows
        .iter()
        .filter(|row| {
            row.get("result")
                .and_then(|result| result.get("classification"))
                .and_then(Value::as_str)
                == Some("complete-original-sfc-module")
        })
        .count();
    let lower_refusals = rows
        .iter()
        .filter(|row| {
            row.get("result")
                .and_then(|result| result.get("classification"))
                .and_then(Value::as_str)
                == Some("lower-refusal")
        })
        .count();
    json!({
        "schema":"vize.native-attribute-values-7502.capture", "version":1,
        "custody":"once-selected-original", "fixtureSha256":inputs::PACK_SHA256,
        "ledgerSha256":inputs.ledger_sha256,
        "original":{"fix":inputs.original_fix,"parent":inputs.original_parent,
            "testBlob":inputs.test_blob,"reporterBlob":inputs.reporter_blob},
        "options":{"filename":FILENAME,"carrier":"<template>{source}</template>",
            "descriptorPolicy":"unchanged target Default::default()",
            "originalCompileOptions":"compile_vapor Default::default()",
            "targets":[{"target":"dom","runtimeVersion":"3.5.35"},
                {"target":"ssr","runtimeVersion":"3.5.35"},
                {"target":"vapor","runtimeVersion":"3.6.0-rc.9"}],
            "linkModes":[{"linkMode":"Recorded","sourceMap":true},
                {"linkMode":"NoLinks","sourceMap":false}]},
        "summary":{"fixtures":inputs.fixtures.len(),"outcomes":rows.len(),
            "positive":positive,"lowerRefusals":lower_refusals}, "rows":rows
    })
}

pub fn compile_row(
    fixture: &Fixture,
    target: Target,
    source_map: bool,
    source: &str,
    expected_envelope_issue: Option<NativeSelectedSfcIssueKind>,
) -> Value {
    let arena = Allocator::default();
    match target {
        Target::Dom => {
            let compilation = compile_native_selected_sfc_dom(
                &arena,
                source,
                NativeSelectedSfcDomOptions {
                    filename: FILENAME,
                    runtime_version: target.version(),
                    source_map,
                    ..Default::default()
                },
            );
            row(
                fixture,
                target,
                source_map,
                source,
                compilation.observation(),
                compilation
                    .result()
                    .map(|output| (output.code(), output.document(), output.source_map()))
                    .map_err(|error| cstr!("{error:?}")),
                expected_envelope_issue,
            )
        }
        Target::Ssr => {
            let compilation = compile_native_ssr_sfc(
                &arena,
                source,
                NativeSsrSfcCompileOptions {
                    filename: FILENAME,
                    runtime_version: target.version(),
                    source_map,
                    ..Default::default()
                },
            );
            row(
                fixture,
                target,
                source_map,
                source,
                compilation.observation(),
                compilation
                    .result()
                    .map(|output| (output.code(), output.document(), output.source_map()))
                    .map_err(|error| cstr!("{error:?}")),
                expected_envelope_issue,
            )
        }
        Target::Vapor => {
            let compilation = compile_native_vapor_sfc(
                &arena,
                source,
                NativeVaporSfcCompileOptions {
                    filename: FILENAME,
                    runtime_version: target.version(),
                    source_map,
                    ..Default::default()
                },
            );
            row(
                fixture,
                target,
                source_map,
                source,
                compilation.observation(),
                compilation
                    .result()
                    .map(|output| (output.code(), output.document(), output.source_map()))
                    .map_err(|error| cstr!("{error:?}")),
                expected_envelope_issue,
            )
        }
    }
}

fn row(
    fixture: &Fixture,
    target: Target,
    source_map: bool,
    source: &str,
    observation: &NativeSelectedSfcObservation<'_>,
    result: Result<(&str, &EmitDocument, Option<&str>), String>,
    expected_envelope_issue: Option<NativeSelectedSfcIssueKind>,
) -> Value {
    let mut captured = json!({
        "id":fixture.id,"target":target.name(),
        "linkMode":if source_map { "Recorded" } else { "NoLinks" },
        "sourceMap":source_map,"runtimeVersion":target.version(),"filename":FILENAME,
        "source":fixture.source,"sourceSha256":fixture.source_sha256,
        "nativeSource":source,"nativeSourceSha256":sha256(source.as_bytes()),
        "existingExpectedTemplateHtml":fixture.existing_expected_template_html,
        "expectedTemplateHtmlSha256":fixture.expected_template_html_sha256,
        "existingOptions":fixture.existing_options,"disposition":fixture.disposition,
        "observation":custody::observation(observation,source),
        "l3":custody::l3(observation,target),
        "result":module_result(result,observation.admitted().is_some())
    });
    if let Some(expected) = expected_envelope_issue
        && let Some(object) = captured.as_object_mut()
    {
        let [issue] = observation.issues() else {
            object.insert("envelopeIssueMatches".into(), json!(false));
            return captured;
        };
        object.insert("envelopeIssueMatches".into(), json!(issue.kind == expected));
    }
    captured
}

fn module_result(
    result: Result<(&str, &EmitDocument, Option<&str>), String>,
    admitted: bool,
) -> Value {
    match result {
        Ok((code, document, map_text)) => {
            let parsed = map_text.map(serde_json::from_str::<Value>).transpose();
            let (map, map_error) = match parsed {
                Ok(map) => (map, None),
                Err(error) => (None, Some(error.to_compact_string())),
            };
            let links: Vec<_> = document
                .links()
                .iter()
                .map(|link| {
                    json!({
                        "authored":custody::span(link.authored),
                        "generated":custody::span(link.generated),
                        "name":link.name.as_deref(),"segment":link.segment
                    })
                })
                .collect();
            json!({"classification":"complete-original-sfc-module", "publicError":null,
                "code":code,"mapText":map_text,"map":map,"mapError":map_error,"links":links})
        }
        Err(error) => json!({
            "classification":if admitted { "target-refusal" } else { "lower-refusal" },
            "publicError":error,"code":null,"mapText":null,"map":null,"mapError":null,"links":[]
        }),
    }
}

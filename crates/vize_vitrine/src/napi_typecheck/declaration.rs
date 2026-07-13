//! Atlas-backed SFC declaration generation.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use vize_atelier_sfc::SfcCroquisMode;
use vize_croquis::declaration_ts::{
    generate_declaration_ts, generate_declaration_ts_with_split_scripts,
};

#[napi(object)]
#[derive(Default)]
pub struct DeclarationOptionsNapi {
    pub filename: Option<String>,
}

#[napi(object)]
pub struct DeclarationResultNapi {
    pub code: String,
}

#[napi(js_name = "generateDeclaration")]
pub fn generate_declaration_napi(
    source: String,
    options: Option<DeclarationOptionsNapi>,
) -> Result<DeclarationResultNapi> {
    let opts = options.unwrap_or_default();
    let filename = opts.filename.unwrap_or_else(|| "anonymous.vue".to_string());
    let graph = crate::artifact_graph::SfcAnalysisGraph::new(
        [(filename.as_str(), source.as_str())],
        SfcCroquisMode::Declaration,
    )
    .map_err(napi_error)?;
    let artifacts = graph.query(&filename).map_err(napi_error)?;
    let descriptor = artifacts.descriptor();
    let summary = artifacts.document().analysis();

    let output = match declaration_script(descriptor) {
        DeclarationScript::None => generate_declaration_ts(summary, None),
        DeclarationScript::Single(script) => generate_declaration_ts(summary, Some(script)),
        DeclarationScript::Split { plain, setup } => {
            generate_declaration_ts_with_split_scripts(summary, plain, setup)
        }
    };

    Ok(DeclarationResultNapi {
        code: output.content.into(),
    })
}

enum DeclarationScript<'a> {
    None,
    Single(&'a str),
    Split { plain: &'a str, setup: &'a str },
}

fn declaration_script<'a>(
    descriptor: &'a vize_atelier_sfc::SfcDescriptor<'a>,
) -> DeclarationScript<'a> {
    match (descriptor.script.as_ref(), descriptor.script_setup.as_ref()) {
        (Some(script), Some(script_setup)) => DeclarationScript::Split {
            plain: script.content.as_ref(),
            setup: script_setup.content.as_ref(),
        },
        (Some(script), None) => DeclarationScript::Single(script.content.as_ref()),
        (None, Some(script_setup)) => DeclarationScript::Single(script_setup.content.as_ref()),
        (None, None) => DeclarationScript::None,
    }
}

fn napi_error(error: vize_carton::String) -> Error {
    Error::new(Status::GenericFailure, error.to_string())
}

#[cfg(test)]
mod tests {
    use vize_atelier_sfc::{
        SfcParseOptions,
        croquis::{SfcCroquisOptions, analyze_sfc_descriptor},
        parse_sfc,
    };

    use super::*;

    #[test]
    fn atlas_declaration_matches_legacy_split_script_output() {
        let source = "<script lang=\"ts\">export interface Shared { id: string }</script>\n<script setup lang=\"ts\">defineProps<{ value: Shared }>()</script>";
        let actual = generate_declaration_napi(
            source.into(),
            Some(DeclarationOptionsNapi {
                filename: Some("Types.vue".into()),
            }),
        )
        .unwrap();
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: "Types.vue".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let summary =
            analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::for_declaration());
        let expected = generate_declaration_ts_with_split_scripts(
            &summary,
            descriptor.script.as_ref().unwrap().content.as_ref(),
            descriptor.script_setup.as_ref().unwrap().content.as_ref(),
        );

        assert_eq!(actual.code.as_bytes(), expected.content.as_bytes());
    }

    #[test]
    fn malformed_declaration_source_returns_cached_parse_message() {
        let error = generate_declaration_napi("<template /><template />".into(), None)
            .err()
            .expect("malformed SFC must fail");
        assert!(error.reason.contains("template"));
    }
}

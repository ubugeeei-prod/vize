use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;
use serde_json::{Value, json};

use super::types::SfcParseOptionsNapi;

/// Parse the original SFC, including content/tag geometry in UTF-8 bytes.
/// Invalid SFC syntax throws `InvalidSfc` with the unchanged parser reason.
#[napi(js_name = "parseSfc")]
pub fn parse_sfc(
    source: String,
    options: Option<SfcParseOptionsNapi>,
) -> Result<Value, &'static str> {
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc as sfc_parse};

    let opts = options.unwrap_or_default();
    let parse_opts = SfcParseOptions {
        filename: opts
            .filename
            .unwrap_or_else(|| "anonymous.vue".to_string())
            .into(),
        ..Default::default()
    };

    match sfc_parse(&source, parse_opts) {
        Ok(descriptor) => Ok(json!({
            "filename": descriptor.filename.as_ref(),
            "source": descriptor.source.as_ref(),
            "template": descriptor.template.as_ref().map(|template| {
                json!({
                    "content": template.content.as_ref(),
                    "loc": block_location(&template.loc),
                    "lang": template.lang.as_deref(),
                })
            }).unwrap_or(Value::Null),
            "script": descriptor.script.as_ref().map(|script| {
                json!({
                    "content": script.content.as_ref(),
                    "loc": block_location(&script.loc),
                    "lang": script.lang.as_deref(),
                    "setup": script.setup,
                })
            }).unwrap_or(Value::Null),
            "scriptSetup": descriptor.script_setup.as_ref().map(|script_setup| {
                json!({
                    "content": script_setup.content.as_ref(),
                    "loc": block_location(&script_setup.loc),
                    "lang": script_setup.lang.as_deref(),
                    "setup": script_setup.setup,
                })
            }).unwrap_or(Value::Null),
            "styles": descriptor.styles.iter().map(|style| {
                json!({
                    "content": style.content.as_ref(),
                    "loc": block_location(&style.loc),
                    "lang": style.lang.as_deref(),
                    "scoped": style.scoped,
                    "module": style.module.as_deref(),
                })
            }).collect::<Vec<_>>(),
            "customBlocks": descriptor.custom_blocks.iter().map(|block| {
                json!({
                    "type": block.block_type.as_ref(),
                    "content": block.content.as_ref(),
                    "loc": block_location(&block.loc),
                })
            }).collect::<Vec<_>>(),
        })),
        Err(e) => Err(Error::new("InvalidSfc", e.message.to_string())),
    }
}

/// Byte geometry from the parser, with content and whole-tag endpoints kept distinct.
/// Offsets and columns are UTF-8 bytes; JavaScript consumers must project UTF-16.
fn block_location(loc: &vize_atelier_sfc::BlockLocation) -> Value {
    json!({
        "start": loc.start,
        "end": loc.end,
        "tagStart": loc.tag_start,
        "tagEnd": loc.tag_end,
        "startLine": loc.start_line,
        "startColumn": loc.start_column,
        "endLine": loc.end_line,
        "endColumn": loc.end_column,
    })
}

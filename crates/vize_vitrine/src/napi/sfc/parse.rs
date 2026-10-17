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

#[cfg(test)]
mod tests {
    use super::parse_sfc;
    use serde_json::json;

    #[test]
    fn parser_result_preserves_all_fields_and_complete_content_and_tag_byte_geometry() {
        let source = "<script setup></script>\r\n<template><template v-if=\"ok\"/><input/></template><style scoped module=\"classes\">.x{}</style><docs>note</docs><script>const a=1;</script>";
        let expected = json!({
          "filename": "anonymous.vue",
          "source": "<script setup></script>\r\n<template><template v-if=\"ok\"/><input/></template><style scoped module=\"classes\">.x{}</style><docs>note</docs><script>const a=1;</script>",
          "scriptSetup": {
            "content": "",
            "loc": {
              "start": 14,
              "end": 14,
              "tagStart": 0,
              "tagEnd": 23,
              "startLine": 1,
              "startColumn": 15,
              "endLine": 1,
              "endColumn": 15
            },
            "lang": null,
            "setup": true
          },
          "template": {
            "content": "<template v-if=\"ok\"/><input/>",
            "loc": {
              "start": 35,
              "end": 64,
              "tagStart": 25,
              "tagEnd": 75,
              "startLine": 2,
              "startColumn": 11,
              "endLine": 2,
              "endColumn": 40
            },
            "lang": null
          },
          "styles": [
            {
              "content": ".x{}",
              "loc": {
                "start": 106,
                "end": 110,
                "tagStart": 75,
                "tagEnd": 118,
                "startLine": 2,
                "startColumn": 82,
                "endLine": 2,
                "endColumn": 86
              },
              "lang": null,
              "scoped": true,
              "module": "classes"
            }
          ],
          "customBlocks": [
            {
              "content": "note",
              "loc": {
                "start": 124,
                "end": 128,
                "tagStart": 118,
                "tagEnd": 135,
                "startLine": 2,
                "startColumn": 100,
                "endLine": 2,
                "endColumn": 104
              },
              "type": "docs"
            }
          ],
          "script": {
            "content": "const a=1;",
            "loc": {
              "start": 143,
              "end": 153,
              "tagStart": 135,
              "tagEnd": 162,
              "startLine": 2,
              "startColumn": 119,
              "endLine": 2,
              "endColumn": 129
            },
            "lang": null,
            "setup": false
          }
        });
        assert_eq!(
            parse_sfc(source.to_owned(), None).expect("valid original SFC"),
            expected
        );
    }

    #[test]
    fn malformed_source_retains_original_reason_under_dedicated_error_code() {
        let error =
            parse_sfc("<template><input/>".to_owned(), None).expect_err("missing root closing tag");
        assert_eq!(error.status, "InvalidSfc");
        assert_eq!(
            error.reason,
            "Malformed <template> block: the closing tag is missing."
        );
    }
}

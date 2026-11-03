//! #11798: complete SFC packets retain ECMAScript line-comment boundaries.
use serde_json::json;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};
use vize_l0::{String, cstr};

fn check_packet(code: &str, ssr: bool) -> Result<(), String> {
    let source = cstr!("<script>{code}</script>");
    let descriptor = parse_sfc(
        &source,
        SfcParseOptions {
            filename: "UnicodeLineComment.vue".into(),
            ..Default::default()
        },
    )
    .map_err(|error| cstr!("ssr={ssr}, {source:?}: {error:?}"))?;
    let end_line = 1 + code.bytes().filter(|byte| *byte == b'\n').count();
    let end_column = code
        .as_bytes()
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(9 + code.len(), |at| code.len() - at);
    assert_eq!(
        serde_json::to_value(&descriptor).unwrap(),
        json!({
            "filename": "UnicodeLineComment.vue", "source": source.as_str(),
            "template": null, "scriptSetup": null, "styles": [], "customBlocks": [],
            "cssVars": [], "slotted": false, "shouldForceReload": false,
            "script": {
                "content": code, "lang": null, "src": null, "setup": false,
                "attrs": {}, "bindings": null,
                "loc": { "start": 8, "end": 8 + code.len(), "tag_start": 0,
                    "tag_end": source.len(), "start_line": 1, "start_column": 9,
                    "end_line": end_line, "end_column": end_column }
            }
        }),
        "complete descriptor, ssr={ssr}: {source:?}"
    );
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "UnicodeLineComment.vue".into(),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile complete SFC");
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        json!({
            "code": code, "css": null, "map": null, "errors": [], "warnings": [],
            "bindings": null, "macroArtifacts": []
        }),
        "complete output, ssr={ssr}: {source:?}"
    );
    Ok(())
}

#[test]
fn export_trivia_preserves_all_thirty_six_public_sfc_packets() {
    let mut failures = Vec::new();
    for prefix in ["", "const label = '雪😀';\r\n"] {
        for gap in [
            " ",
            "\r\n",
            "/* note */",
            "/**//**/",
            "/* default decoy */ \r\n// export decoy\r\n",
            "\u{feff}",
            "\u{2003}/* 😀 */\u{feff}",
            "// note\u{2028}",
            "// note\u{2029}",
        ] {
            let code = cstr!("{prefix}export{gap}default {{ label: '雪' }};");
            for ssr in [false, true] {
                if let Err(error) = check_packet(&code, ssr) {
                    failures.push(error);
                }
            }
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

#[test]
fn ascii_terminators_and_an_authored_lf_before_the_closing_tag_keep_exact_packets() {
    for prefix in ["", "const label = '雪😀';\r\n"] {
        for newline in ["\n", "\r", "\r\n"] {
            let code = cstr!("{prefix}export// note{newline}default {{ label: '雪' }};");
            for ssr in [false, true] {
                check_packet(&code, ssr).unwrap();
            }
        }
        for newline in ["\u{2028}", "\u{2029}"] {
            let code = cstr!("{prefix}export// note{newline}default {{ label: '雪' }};\n");
            for ssr in [false, true] {
                check_packet(&code, ssr).unwrap();
            }
        }
    }
}

#[test]
fn unicode_comment_terminators_inside_template_expressions_preserve_complete_packets() {
    let mut failures = Vec::new();
    for newline in ["\u{2028}", "\u{2029}"] {
        let code = cstr!(
            "const token = `${{ // note{newline} /[}}/]/.test('}}') ? 'a' : 'b' }}`; export default {{}};"
        );
        for ssr in [false, true] {
            if let Err(error) = check_packet(&code, ssr) {
                failures.push(error);
            }
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

#[test]
fn strings_templates_regex_and_terminated_comments_keep_exact_packets() {
    for code in [
        "const token = '</script>'; export default {};",
        "const token = \"</script>\"; export default {};",
        "const token = `</script>`; export default {};",
        "const token = '\u{2028}</script>\u{2029}'; export default {};",
        r"const token = /[</script>\/]/giu; export default {};",
        "/* </script> */ export default {};",
        "// </script>\nexport default {};",
        "// </script>\r\nexport default {};",
    ] {
        for ssr in [false, true] {
            check_packet(code, ssr).unwrap();
        }
    }
}

#[test]
fn unterminated_comments_and_literals_retain_complete_malformed_packets() {
    for source in [
        "<script>// no terminator </script>",
        "<script>/* no end </script>",
        "<script>const token = `unterminated </script>",
        "<script>const token = `${ // no terminator </script>",
    ] {
        let error = parse_sfc(source, SfcParseOptions::default()).unwrap_err();
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            json!({
                "message": "Malformed <script> block: the closing tag is missing.",
                "code": "MALFORMED_BLOCK",
                "loc": { "start": 0, "end": source.len(), "tag_start": 0,
                    "tag_end": source.len(), "start_line": 1, "start_column": 1,
                    "end_line": 1, "end_column": source.len() + 1 }
            }),
            "{source:?}"
        );
    }
}

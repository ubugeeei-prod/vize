//! #7704: resolve Auto from authored input before any block formatting.
#![expect(
    clippy::disallowed_macros,
    reason = "regression fixtures construct independent complete expected bytes"
)]
use vize_glyph::{
    Allocator, EndOfLine, FormatOptions, GlyphFormatter, VueVersion, format_json, format_jsonc,
    format_script, format_script_with_sort_imports, format_script_with_source_type, format_sfc,
    format_sfc_with_allocator, format_sfc_with_allocator_and_vue_version,
    format_sfc_with_vue_version, format_style, format_template, format_template_with_vue_version,
};

fn auto_options() -> FormatOptions {
    FormatOptions {
        end_of_line: EndOfLine::Auto,
        ..FormatOptions::default()
    }
}

#[test]
fn every_standalone_public_entrypoint_resolves_the_same_first_terminator() {
    let options = auto_options();
    let allocator = Allocator::default();
    for newline in ["\n", "\r\n", "\r"] {
        let script = format!("const value=1{newline}");
        let expected_script = format!("const value = 1;{newline}");
        assert_eq!(format_script(&script, &options).unwrap(), expected_script);
        assert_eq!(
            format_script_with_source_type(
                &script,
                &options,
                &allocator,
                oxc_span::SourceType::ts()
            )
            .unwrap(),
            expected_script
        );
        assert_eq!(
            format_script_with_sort_imports(
                &script,
                &options,
                &allocator,
                oxc_span::SourceType::ts(),
                None,
            )
            .unwrap(),
            expected_script
        );

        let template = format!("<div>{newline}<p>{{{{ value }}}}</p>{newline}</div>");
        let expected_template = format!("<div>{newline}  <p>{{{{ value }}}}</p>{newline}</div>");
        assert_eq!(
            format_template(&template, &options).unwrap(),
            expected_template
        );
        for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
            assert_eq!(
                format_template_with_vue_version(&template, &options, version).unwrap(),
                expected_template
            );
        }
        assert_eq!(
            format_template(&expected_template, &options).unwrap(),
            expected_template
        );

        let json = format!("{{{newline}\"value\":1}}");
        let expected_json = format!("{{{newline}  \"value\": 1{newline}}}{newline}");
        assert_eq!(format_json(&json, &options).unwrap(), expected_json);
        assert_eq!(
            format_json(&expected_json, &options).unwrap(),
            expected_json
        );
        let jsonc = format!("// keep{newline}{json}");
        let expected_jsonc = format!("// keep{newline}{expected_json}");
        assert_eq!(format_jsonc(&jsonc, &options).unwrap(), expected_jsonc);
        assert_eq!(
            format_jsonc(&expected_jsonc, &options).unwrap(),
            expected_jsonc
        );

        let style = format!(".box {{{newline}color:red{newline}}}");
        let expected_style = format!(".box {{{newline}  color: red;{newline}}}");
        // Preserve the CSS printer/reindent path's existing final-newline convention.
        let expected_style = if newline == "\n" {
            format!("{expected_style}\n")
        } else {
            expected_style
        };
        assert_eq!(format_style(&style, &options).unwrap(), expected_style);
        assert_eq!(
            format_style(&expected_style, &options).unwrap(),
            expected_style
        );
    }
    assert_eq!(
        options.end_of_line,
        EndOfLine::Auto,
        "caller options stay immutable"
    );
}

#[test]
fn sfc_resolves_whole_document_before_each_embedded_language() {
    let options = auto_options();
    let allocator = Allocator::default();
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!(
            "<script setup>{newline}const value=1\n</script>\n<template>\n<p>{{{{ value }}}}</p>\n</template>\n<style>\n.box{{color:red}}\n</style>\n"
        );
        let expected = format!(
            "<script setup>{newline}const value = 1;{newline}</script>{newline}{newline}<template>{newline}  <p>{{{{ value }}}}</p>{newline}</template>{newline}{newline}<style>{newline}.box {{{newline}  color: red;{newline}}}{newline}</style>{newline}"
        );
        assert_eq!(format_sfc(&source, &options).unwrap().code, expected);
        assert_eq!(
            format_sfc_with_allocator(&source, &options, &allocator)
                .unwrap()
                .code,
            expected
        );
        assert_eq!(
            GlyphFormatter::new(&options, &allocator)
                .format(&source)
                .unwrap()
                .code,
            expected
        );
        for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
            assert_eq!(
                format_sfc_with_vue_version(&source, &options, version)
                    .unwrap()
                    .code,
                expected
            );
            assert_eq!(
                format_sfc_with_allocator_and_vue_version(&source, &options, &allocator, version)
                    .unwrap()
                    .code,
                expected
            );
        }
        let again = format_sfc(&expected, &options).unwrap();
        assert_eq!(again.code, expected);
        assert!(!again.changed);
    }
}

#[test]
fn no_terminator_falls_back_to_lf_and_explicit_options_win() {
    let source = "<template><p>value</p></template>";
    let expected = "<template>\n  <p>value</p>\n</template>\n";
    assert_eq!(
        format_sfc(source, &auto_options()).unwrap().code.as_str(),
        expected
    );
    let windows = expected.replace('\n', "\r\n");
    assert_eq!(
        format_sfc(&windows, &FormatOptions::default())
            .unwrap()
            .code
            .as_str(),
        expected
    );
    let options = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    };
    assert_eq!(format_sfc(expected, &options).unwrap().code, windows);
    assert_eq!(
        format_json("{\"value\":1}", &auto_options())
            .unwrap()
            .as_str(),
        "{ \"value\": 1 }\n"
    );
}

#[test]
fn indented_embedded_blocks_use_the_resolved_terminator() {
    let options = FormatOptions {
        vue_indent_script_and_style: true,
        ..auto_options()
    };
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!(
            "<script setup>{newline}const value=1\n</script>\n<style>\n.box{{color:red}}\n</style>\n"
        );
        let expected = format!(
            "<script setup>{newline}  const value = 1;{newline}</script>{newline}{newline}<style>{newline}  .box {{{newline}    color: red;{newline}  }}{newline}</style>{newline}"
        );
        assert_eq!(format_sfc(&source, &options).unwrap().code, expected);
        let again = format_sfc(&expected, &options).unwrap();
        assert_eq!(again.code, expected);
        assert!(!again.changed);
    }
}

#[test]
fn authored_raw_template_body_keeps_its_newline_bytes() {
    let options = auto_options();
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!(
            "<template>{newline}  <pre>first{newline}second</pre>{newline}</template>{newline}"
        );
        assert_eq!(format_sfc(&source, &options).unwrap().code, source);
        let again = format_sfc(&source, &options).unwrap();
        assert_eq!(again.code, source);
        assert!(!again.changed);
    }
}

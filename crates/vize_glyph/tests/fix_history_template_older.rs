//! Whole public formatter output for older fix-history regressions (#6882).
#![expect(clippy::unwrap_used, reason = "history fixtures assert by panicking")]
use vize_glyph::{FormatOptions, format_sfc, format_template};

fn snapshot_bytes(name: &str, output: &str) {
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, output.as_bytes().to_vec());
    });
}

fn sfc(name: &str, source: &str, options: &FormatOptions) {
    let first = format_sfc(source, options).unwrap();
    snapshot_bytes(name, &first.code);
    let second = format_sfc(&first.code, options).unwrap();
    let third = format_sfc(&second.code, options).unwrap();
    assert_eq!(
        first.code, second.code,
        "first output must be a fixed point"
    );
    assert_eq!(second.code, third.code, "fixed point must stay stable");
}

fn template(name: &str, source: &str, options: &FormatOptions) {
    let first = format_template(source, options).unwrap();
    snapshot_bytes(name, &first);
    let second = format_template(&first, options).unwrap();
    assert_eq!(first, second, "first output must be a fixed point");
}

#[test]
fn original_raw_region_inputs_deleted_by_interpolation_fix_are_retained() {
    // 8a5632066 added these exact sources; 4abb3d0ec deleted their partial tests.
    let options = FormatOptions::default();
    let cases = [
        (
            "history_sfc_original_raw_pre.txt",
            "<template>\n  <pre>\nline1\n      indented\n   z\n</pre>\n</template>\n",
        ),
        (
            "history_sfc_original_raw_textarea.txt",
            "<template>\n  <textarea>\nA\n   B\n</textarea>\n</template>\n",
        ),
        (
            "history_sfc_original_raw_v_pre.txt",
            "<template>\n  <div v-pre>{{   raw   }}</div>\n</template>\n",
        ),
    ];
    for (name, source) in cases {
        sfc(name, source, &options);
    }
}

#[test]
fn raw_template_comment_and_unclosed_remainder_have_complete_output() {
    // 8a5632066 find_matching_close_tag skips fake closes in comments and copies EOF.
    let cases = [
        (
            "history_template_raw_fake_close_comment.txt",
            "<pre>\n<!-- </pre> -->\n   raw\n</pre><span>next</span>",
        ),
        (
            "history_template_unclosed_raw_pre.txt",
            "<pre>\n  raw {{   x   }}\n",
        ),
        (
            "history_template_unclosed_raw_textarea.txt",
            "<textarea>\n  raw {{   x   }}\n",
        ),
        (
            "history_template_unclosed_raw_v_pre.txt",
            "<div v-pre>\n  raw {{   x   }}\n",
        ),
    ];
    for (name, source) in cases {
        template(name, source, &FormatOptions::default());
    }
}

#[test]
fn directive_entity_decode_branches_have_complete_public_output() {
    // 86ca0ba60 decode_expression_attribute_entities: every recognized spelling.
    // &quot; is the same exact input already captured by the public quote fixture
    // in fix_history_template_gaps; reuse that witness instead of duplicating it.
    let cases = [
        (
            "history_template_entity_double_decimal.txt",
            r#"<div :title="prefix + &#34;a'b&#34;"></div>"#,
        ),
        (
            "history_template_entity_double_hex_lower.txt",
            r#"<div :title="prefix + &#x22;a'b&#x22;"></div>"#,
        ),
        (
            "history_template_entity_double_hex_upper.txt",
            r#"<div :title="prefix + &#X22;a'b&#X22;"></div>"#,
        ),
        (
            "history_template_entity_apos.txt",
            r#"<div :title="prefix + &apos;ab&apos;"></div>"#,
        ),
        (
            "history_template_entity_single_decimal.txt",
            r#"<div :title="prefix + &#39;ab&#39;"></div>"#,
        ),
        (
            "history_template_entity_single_hex_lower.txt",
            r#"<div :title="prefix + &#x27;ab&#x27;"></div>"#,
        ),
        (
            "history_template_entity_single_hex_upper.txt",
            r#"<div :title="prefix + &#X27;ab&#X27;"></div>"#,
        ),
        (
            "history_template_entity_amp.txt",
            r#"<div :title="left &amp;&amp; right"></div>"#,
        ),
        (
            "history_template_entity_unknown.txt",
            r#"<div :title="prefix + '&unknown;'"></div>"#,
        ),
    ];
    for (name, source) in cases {
        template(name, source, &FormatOptions::default());
    }
}

#[test]
fn public_script_block_indent_handles_escaped_backticks() {
    // 752416407 helper escape parity, through the public SFC indentation path.
    let source = "<script>\nconst a = `left\\`right\n  tail`\nconst b = 2\n</script>\n";
    let options = FormatOptions {
        vue_indent_script_and_style: true,
        ..FormatOptions::default()
    };
    sfc(
        "history_sfc_indented_escaped_backtick.txt",
        source,
        &options,
    );
}

#[test]
fn wbr_and_uppercase_void_names_have_complete_template_output() {
    // 06a5c56da adds wbr; 398bc61f4 makes uppercase HTML names components.
    let cases = [
        (
            "history_template_wbr_sibling.txt",
            "<div><wbr><span>next</span></div>",
        ),
        (
            "history_template_input_component_child.txt",
            "<Input><span>child</span></Input>",
        ),
        (
            "history_template_img_component_child.txt",
            "<Img><span>child</span></Img>",
        ),
    ];
    for (name, source) in cases {
        template(name, source, &FormatOptions::default());
    }
}

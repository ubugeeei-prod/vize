use vize_glyph::{FormatOptions, format_sfc, format_style};

#[test]
fn keeps_comments_between_and_after_sfc_blocks() {
    let source = concat!(
        "<script setup lang=\"ts\">\n",
        "const label = \"hello\";\n",
        "</script>\n\n",
        "<!-- NOTE: this comment explains the template below -->\n",
        "<template>\n  <div class=\"box\">{{ label }}</div>\n</template>\n\n",
        "<!-- NOTE: keep the closing note too -->\n",
    );
    let options = FormatOptions::default();
    let formatted = format_sfc(source, &options).unwrap().code;
    insta::assert_snapshot!(formatted);
    assert_eq!(format_sfc(&formatted, &options).unwrap().code, formatted);
}

#[test]
fn comments_follow_their_block_when_sfc_blocks_are_sorted() {
    let source = concat!(
        "<!-- template note -->\n",
        "<template><div>ok</div></template>\n",
        "<!-- script note -->\n",
        "<script setup>const ready = true;</script>\n",
    );
    let options = FormatOptions::default();
    let formatted = format_sfc(source, &options).unwrap().code;
    insta::assert_snapshot!(formatted);
    assert_eq!(format_sfc(&formatted, &options).unwrap().code, formatted);
}

#[test]
fn retains_authored_css_color_spellings() {
    let source = ".box { color: grey; border: 1px solid lightgrey; }";
    let options = FormatOptions::default();
    let formatted = format_style(source, &options).unwrap();
    insta::assert_snapshot!(formatted);
    assert_eq!(format_style(&formatted, &options).unwrap(), formatted);
}

#[test]
fn retains_color_functions_and_hex_without_touching_strings_or_urls() {
    let source = concat!(
        ".box {\n",
        "  color: #D3D3D3;\n",
        "  background: linear-gradient(rgb(255 0 0), lightgrey);\n",
        "  content: \"grey\";\n",
        "  background-image: url(/grey.svg);\n",
        "}\n",
    );
    let options = FormatOptions::default();
    let formatted = format_style(source, &options).unwrap();
    insta::assert_snapshot!(formatted);
    assert_eq!(format_style(&formatted, &options).unwrap(), formatted);
}

#[test]
fn retains_authored_css_color_spellings_in_sfc() {
    let source = concat!(
        "<template><div class=\"box\" /></template>\n",
        "<style scoped>\n",
        ".box { color: grey; border: 1px solid lightgrey; }\n",
        "</style>\n",
    );
    let options = FormatOptions::default();
    let formatted = format_sfc(source, &options).unwrap().code;
    insta::assert_snapshot!(formatted);
    assert_eq!(format_sfc(&formatted, &options).unwrap().code, formatted);
}

#[test]
fn css_formatting_keeps_media_features_values_and_nested_selectors() {
    let source = concat!(
        ".box { margin: 1px 0 1px 0; border: solid 1px; }\n",
        ".box::before { content: \"\"; }\n",
        "@media screen and (max-width: 600px) { .box { color: blue; } }\n",
        ".parent { .child { color: red; } }",
    );
    let options = FormatOptions::default();
    let formatted = format_style(source, &options).unwrap();
    assert!(formatted.contains("margin: 1px 0 1px 0"), "{formatted}");
    assert!(formatted.contains("border: solid 1px"), "{formatted}");
    assert!(formatted.contains(".box::before"), "{formatted}");
    assert!(formatted.contains("(max-width: 600px)"), "{formatted}");
    assert!(formatted.contains(".parent {\n  .child {"), "{formatted}");
    assert_eq!(format_style(&formatted, &options).unwrap(), formatted);
}

#[test]
fn scoped_css_keeps_implicit_nested_selectors() {
    let source = "<template><div class=\"p\" /></template>\n<style scoped>\n.p { :deep(.child) { color: red; } .q { color: blue; } }\n</style>";
    let options = FormatOptions::default();
    let formatted = format_sfc(source, &options).unwrap().code;
    assert!(formatted.contains(":deep(.child)"), "{formatted}");
    assert!(!formatted.contains("& :deep(.child)"), "{formatted}");
    assert!(!formatted.contains("& .q"), "{formatted}");
    assert_eq!(format_sfc(&formatted, &options).unwrap().code, formatted);
}

#[test]
fn css_comment_keeps_preceding_blank_line() {
    let source = ".a { color: red; }\n\n/* second */\n.b { color: blue; }";
    let options = FormatOptions::default();
    let formatted = format_style(source, &options).unwrap();
    assert!(formatted.contains("}\n\n/* second */"), "{formatted}");
    assert_eq!(format_style(&formatted, &options).unwrap(), formatted);
}

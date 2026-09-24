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

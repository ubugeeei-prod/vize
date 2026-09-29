use super::{FormatOptions, format_template_content};

#[test]
fn test_print_width_triggers_multiline() {
    // Very narrow print_width should trigger multiline
    let source = r#"<div class="container" id="main" title="tooltip"></div>"#;
    let options = FormatOptions {
        print_width: 30,
        ..FormatOptions::default()
    };
    let result = format_template_content(source, &options).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    assert!(
        lines.len() > 2,
        "Narrow print_width should trigger multiline attributes"
    );
}

#[test]
fn single_long_attribute_wraps_at_print_width() {
    // #7236: one attribute used to ignore print width and was joined back
    // onto the tag line. The block class is longer than 80 on its own; the
    // inline class is exactly 80 at depth 0 (`<span class="…">`) and over 80
    // once indented under a parent.
    let block_class =
        "a-very-long-block-element-class-name-that-alone-is-longer-than-the-print-width";
    let inline_class = "a-long-inline-element-class-name-that-is-close-to-the-print-width";
    let options = FormatOptions {
        print_width: 80,
        ..FormatOptions::default()
    };

    let inline = format!(r#"<div class="{block_class}">text</div>"#);
    assert_eq!(
        format_template_content(&inline, &options).unwrap().as_str(),
        format!("<div\n  class=\"{block_class}\"\n>text</div>")
    );

    let already_wrapped = format!("<div\n  class=\"{block_class}\"\n>\n  text\n</div>\n");
    let wrapped = format_template_content(&already_wrapped, &options).unwrap();
    assert_eq!(
        wrapped.as_str(),
        format!("<div\n  class=\"{block_class}\"\n>\n  text\n</div>")
    );
    assert_eq!(
        format_template_content(&wrapped, &options).unwrap(),
        wrapped,
        "wrapping a single long attribute must be stable"
    );

    let fits = format!(r#"<span class="{inline_class}">x</span>"#);
    assert_eq!(
        format_template_content(&fits, &options).unwrap().as_str(),
        fits.as_str(),
        "a single attribute that fits print width stays on one line"
    );

    let nested = format!(
        "<div>\n  <span class=\"{inline_class}\">{{{{ lap }}}}</span>\n  <div class=\"ok\">text</div>\n</div>\n"
    );
    let nested_formatted = format_template_content(&nested, &options).unwrap();
    assert_eq!(
        nested_formatted.as_str(),
        format!(
            "<div>\n  <span\n    class=\"{inline_class}\"\n  >{{{{ lap }}}}</span>\n  <div class=\"ok\">text</div>\n</div>"
        )
    );
}

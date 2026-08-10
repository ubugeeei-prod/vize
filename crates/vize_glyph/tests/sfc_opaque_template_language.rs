use vize_glyph::{FormatOptions, format_sfc};

fn assert_fixed_point(source: &str, options: &FormatOptions) -> String {
    let first = format_sfc(source, options).unwrap().code;
    let second = format_sfc(&first, options).unwrap().code;
    assert_eq!(
        first, second,
        "non-HTML template formatting must be a fixed point"
    );
    first.into()
}

#[test]
fn pug_keeps_relative_program_indentation_and_verbatim_content() {
    let source = r#"<template lang="pug">
main
  //- formatter must not parse this as HTML
  button(@click="save" :title="'quoted value'") Save
    span {{ message }}
  :plain
    a  b
  | tail  text
</template>
"#;
    let output = assert_fixed_point(source, &FormatOptions::default());
    assert!(output.contains(
        r#"<template lang="pug">
  main
    //- formatter must not parse this as HTML
    button(@click="save" :title="'quoted value'") Save
      span {{ message }}
    :plain
      a  b
    | tail  text
</template>"#
    ));
}

#[test]
fn every_non_html_language_uses_the_opaque_strategy() {
    let source = "<template lang=\"haml\">\n%main\n  %span= message\n</template>\n";
    let output = assert_fixed_point(source, &FormatOptions::default());
    assert!(output.contains("  %main\n    %span= message\n"));
}

#[test]
fn explicit_html_still_uses_the_native_template_formatter() {
    let source = "<template lang=\"html\"><div   :title=\"message\">{{message}}</div></template>\n";
    let output = assert_fixed_point(source, &FormatOptions::default());
    assert!(output.contains("<div :title=\"message\">"), "{output}");
    assert!(output.contains("{{ message }}"), "{output}");
    assert!(!output.contains("<div   :title"), "{output}");
}

#[test]
fn opaque_templates_honor_crlf_and_tab_outer_indentation() {
    let mut options = FormatOptions::default();
    options.use_tabs = true;
    options.end_of_line = vize_glyph::EndOfLine::Crlf;
    let source = "<template lang=\"pug\">\r\n  main\r\n    span text\r\n\r\n</template>\r\n";
    let output = assert_fixed_point(source, &options);
    assert!(output.contains("<template lang=\"pug\">\r\n\tmain\r\n\t  span text\r\n</template>"));
    assert!(!output.replace("\r\n", "").contains('\n'));
}

#[test]
fn opaque_template_src_and_empty_blocks_keep_their_identity() {
    let options = FormatOptions::default();
    for source in [
        "<template lang=\"pug\" src=\"./App.pug\"></template>\n",
        "<template lang=\"pug\">\n\n</template>\n",
    ] {
        let output = assert_fixed_point(source, &options);
        assert!(output.contains("<template lang=\"pug\""));
    }
    let external = assert_fixed_point(
        "<template lang=\"pug\" src=\"./App.pug\"></template>\n",
        &options,
    );
    assert!(external.contains("src=\"./App.pug\""));
}

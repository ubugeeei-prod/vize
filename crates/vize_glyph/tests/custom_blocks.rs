use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn art_block_indents_variants_consistently() {
    let source = r#"<art>
<variant name="Primary" default>
    <DemoButton color="primary">Primary</DemoButton>
  </variant>
  <variant name="Secondary">
    <DemoButton color="secondary">Secondary</DemoButton>
  </variant>
</art>
"#;

    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert_eq!(
        first.code.as_str(),
        r#"<art>
  <variant default name="Primary">
    <DemoButton color="primary">
      Primary
    </DemoButton>
  </variant>
  <variant name="Secondary">
    <DemoButton color="secondary">
      Secondary
    </DemoButton>
  </variant>
</art>
"#
    );
    assert_eq!(first.code, second.code);
}

#[test]
fn art_block_preserves_blank_lines_between_variants() {
    let source = r#"<art>
  <variant name="Primary" default>
    <DemoButton>Primary</DemoButton>
  </variant>

  <variant name="Secondary">
    <DemoButton color="secondary">Secondary</DemoButton>
  </variant>
</art>
"#;

    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert_eq!(
        first.code.as_str(),
        r#"<art>
  <variant default name="Primary">
    <DemoButton>
      Primary
    </DemoButton>
  </variant>

  <variant name="Secondary">
    <DemoButton color="secondary">
      Secondary
    </DemoButton>
  </variant>
</art>
"#
    );
    assert_eq!(first.code, second.code);
}

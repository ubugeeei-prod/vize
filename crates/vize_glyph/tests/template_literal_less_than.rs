use vize_glyph::{FormatOptions, format_template};

#[test]
fn literal_less_than_text_is_preserved_without_hanging() {
    let source = r#"<router-link> << {{ label }}</router-link>"#;
    let options = FormatOptions::default();

    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(first.as_str(), "<router-link> << {{ label }}</router-link>");
    assert_eq!(first, second);
}

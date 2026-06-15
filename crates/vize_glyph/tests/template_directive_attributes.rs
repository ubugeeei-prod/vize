use vize_glyph::{FormatOptions, format_sfc, format_template};

#[test]
fn multiline_directive_attribute_value_is_indented_from_attribute_depth() {
    let source = r#"<span
  :class='[
rec.years.includes(y) && selectedYear === y
  ? "bg-accent border border-accent text-accent-ink"
  : rec.years.includes(y)
    ? "bg-ink border border-ink text-paper"
    : "border border-ink text-ink",
]'
  :title="y"
></span>"#;

    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(
        first.as_str(),
        r#"<span
  :class='[
    rec.years.includes(y) && selectedYear === y
      ? "bg-accent border border-accent text-accent-ink"
      : rec.years.includes(y)
        ? "bg-ink border border-ink text-paper"
        : "border border-ink text-ink",
  ]'
  :title="y"
></span>"#
    );
    assert_eq!(first, second);
}

#[test]
fn sfc_multiline_directive_attribute_keeps_template_indent() {
    let source = "<template>\n  <button\n    type=\"button\"\n    :class='sort === \"name-asc\" || sort === \"name-desc\"\n    ? \"bg-ink text-paper border-ink\"\n    : \"border-rule text-ink-2 hover:text-ink hover:border-ink\"'\n    @click=\"toggleNameSort\"\n  >\n    Name\n  </button>\n</template>\n";
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert_eq!(
        first.code.as_str(),
        "<template>\n  <button\n    type=\"button\"\n    :class='sort === \"name-asc\" || sort === \"name-desc\"\n      ? \"bg-ink text-paper border-ink\"\n      : \"border-rule text-ink-2 hover:text-ink hover:border-ink\"'\n    @click=\"toggleNameSort\"\n  >\n    Name\n  </button>\n</template>\n"
    );
    assert_eq!(first.code, second.code);
}

#[test]
fn sfc_single_multiline_directive_attribute_is_idempotent() {
    let source = r#"<template>
  <label
    :style="props.reverseOrder
      ? 'grid-template-areas: \'toggle . label-text\''
      : 'grid-template-areas: \'label-text . toggle\''"
  >
  </label>
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    assert!(
        first.code.contains("\n    :style="),
        "single multiline attribute should stay on its own line:\n{}",
        first.code
    );
}

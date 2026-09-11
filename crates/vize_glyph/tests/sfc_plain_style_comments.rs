use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn plain_style_preserves_nested_comments() {
    let source = r#"<template>
  <div class="box">hello</div>
</template>

<style scoped>
.box {
  /* keep color note */
  color: red;
  /* keep nested note */
  .inner {
    color: blue; /* keep inline note */
  }
}
</style>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert_eq!(first.code, source);
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
}

use vize_glyph::{FormatOptions, format_template};

/// Regression test for https://github.com/ubugeeei-prod/vize/issues/2244.
///
/// `<Link>` (PascalCase Vue component) used to collide with HTML `<link>`
/// inside `is_void_element_str`, so the formatter skipped the child-depth
/// increment and emitted the component's children flush with their parent.
/// The fix treats any tag starting with an uppercase ASCII letter as a
/// component, never a void element.
#[test]
fn nested_component_with_attrs_keeps_child_indent_stable() {
    let source = r#"<template>
  <div>
    <Link :to='{ name: "signup" }'>
      {{ t("general.signUp._") }}
    </Link>
  </div>
  <div>
    <span>
      {{ t("general.password.reset.description") }}
    </span>
    <Link :to='{ name: "password-reset" }'>
      {{ t("general.password.reset._") }}
    </Link>
  </div>
</template>"#;

    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(
        first.as_str(),
        r#"<template>
  <div>
    <Link :to='{ name: "signup" }'>
      {{ t("general.signUp._") }}
    </Link>
  </div>
  <div>
    <span>
      {{ t("general.password.reset.description") }}
    </span>
    <Link :to='{ name: "password-reset" }'>
      {{ t("general.password.reset._") }}
    </Link>
  </div>
</template>"#
    );
    assert_eq!(first, second, "should be idempotent");
}

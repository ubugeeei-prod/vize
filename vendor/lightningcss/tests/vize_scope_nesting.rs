//! A `@scope` body nested in a style rule is relative to its scoping root,
//! not to the parent rule, so its rules gain no implicit `&` (#10810).
#![expect(clippy::disallowed_types, reason = "the printer returns an owned std string")]

use lightningcss::{
  printer::PrinterOptions,
  stylesheet::{ParserOptions, StyleSheet},
};

fn print(source: &str, minify: bool) -> String {
  let sheet = StyleSheet::parse(source, ParserOptions::default()).unwrap();
  sheet
    .to_css(PrinterOptions {
      minify,
      ..Default::default()
    })
    .unwrap()
    .code
}

#[test]
fn nested_scope_body_rules_stay_relative_to_the_scoping_root() {
  assert_eq!(
    print(".a{@scope (.s){:scope{color:blue}.c{color:red}}}", true),
    ".a{@scope(.s){:scope{color:#00f}.c{color:red}}}"
  );
  // An authored `&` and the parent's own nested rules keep their nesting.
  assert_eq!(
    print(".a{@scope (.s){& .c{color:red}}.b{color:red}}", true),
    ".a{@scope(.s){& .c{color:red}}& .b{color:red}}"
  );
  // A conditional block in the scope body stays relative to the root.
  assert_eq!(
    print(".a{@scope (.s){@media print{.c{color:red}}}}", true),
    ".a{@scope(.s){@media print{.c{color:red}}}}"
  );
}

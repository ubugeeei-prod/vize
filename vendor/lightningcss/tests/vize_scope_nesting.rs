//! A `@scope` body nested in a style rule is relative to its scoping root,
//! not to the parent rule, so its rules gain no implicit `&` (#10810).
#![expect(clippy::disallowed_types, reason = "the printer returns an owned std string")]

use lightningcss::{
  printer::PrinterOptions,
  rules::CssRule,
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

#[test]
fn scope_body_accepts_leading_combinators_and_mixed_selector_lists() {
  for source in [
    ".a{@scope(.s){> .c{color:red}}}",
    "@scope(.s){> .c{color:red}}",
    ".a{@scope(.s){@supports(display:block){> .c{color:red}}}}",
  ] {
    let printed = print(source, true);
    assert!(printed.contains("&>.c{color:red}"), "{printed}");
    assert!(!printed.contains(":scope>"), "{printed}");
  }
  assert_eq!(
    print(".a{@scope(.s){:scope,> .c,.d{color:red}}}", true),
    ".a{@scope(.s){:scope,&>.c,.d{color:red}}}"
  );
  assert_eq!(
    print(".a{@scope(.s){+ .c,~ .d{color:red}}}", true),
    ".a{@scope(.s){&+.c,&~.d{color:red}}}"
  );
}

#[test]
fn scope_body_does_not_change_nested_style_rule_ownership() {
  assert_eq!(
    print(".a{@scope(.s){.c{.d{color:red}}}.e{color:blue}}", true),
    ".a{@scope(.s){.c{& .d{color:red}}}& .e{color:#00f}}"
  );
  assert_eq!(
    print(".a{@scope(.s){main :scope .c,main & .d{color:red}}}", true),
    ".a{@scope(.s){main :scope .c,main & .d{color:red}}}"
  );
}

#[test]
fn only_authored_scope_contributes_specificity() {
  let sheet = StyleSheet::parse(
    ".a{@scope(.s){> .c,:scope>.c,&>.c,.c{color:red}}}",
    ParserOptions::default(),
  )
  .unwrap();
  let CssRule::Style(outer) = &sheet.rules.0[0] else {
    panic!("outer style")
  };
  let CssRule::Scope(scope) = &outer.rules.0[0] else {
    panic!("scope body")
  };
  let CssRule::Style(rule) = &scope.rules.0[0] else {
    panic!("scope style")
  };
  let specificities: Vec<_> = rule.selectors.0.iter().map(|selector| selector.specificity()).collect();
  assert_eq!(specificities, [1 << 10, 2 << 10, 1 << 10, 1 << 10]);
}

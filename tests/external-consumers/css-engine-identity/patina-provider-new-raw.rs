use vize_patina::css_ast::stylesheet::{ParserOptions, StyleSheet};
use vize_patina::style::{ParsedStyleSheet, StyleDocument};

fn main() {
    let source = "a{color:red}";
    let document = StyleDocument::css(source, 0);
    let stylesheet = StyleSheet::parse(source, ParserOptions::default()).unwrap();
    let parsed = ParsedStyleSheet::new(&document, &stylesheet);
    let _: &StyleSheet<'_> = parsed.raw();
    assert_eq!(parsed.source(), source);
    let mut rules = 0;
    parsed.for_each_style_rule(&mut |_| rules += 1);
    assert_eq!(rules, 1);
}

use lightningcss::rules::{font_face::FontFaceRule, style::StyleRule};
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use vize_patina::style::{ParsedStyleSheet, StyleDocument};

fn main() {
    let source = "a{color:red}@font-face{font-family:Demo;src:url(demo.woff2)}";
    let document = StyleDocument::css(source, 0);
    let stylesheet = StyleSheet::parse(source, ParserOptions::default()).unwrap();
    let parsed = ParsedStyleSheet::new(&document, &stylesheet);
    let mut styles = 0;
    parsed.for_each_style_rule(&mut |_: &StyleRule<'_>| styles += 1);
    let mut fonts = 0;
    parsed.for_each_font_face_rule(&mut |_: &FontFaceRule<'_>| fonts += 1);
    assert_eq!([styles, fonts], [1, 1]);
}

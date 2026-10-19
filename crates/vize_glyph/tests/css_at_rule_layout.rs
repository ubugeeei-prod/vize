//! Every block at-rule the layout pass owns, plus nesting, multi-value
//! continuations, CRLF and non-ASCII selector offsets: exact output and fixed
//! points through the public CSS formatter.

#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use vize_glyph::{FormatOptions, format_style};

#[track_caller]
fn assert_formats(source: &str, expected: &str) {
    let options = FormatOptions::default();
    let first = format_style(source, &options).unwrap();
    assert_eq!(first.as_str(), expected, "pass 1");
    let second = format_style(&first, &options).unwrap();
    assert_eq!(second, first, "fmt; fmt must be a no-op");
}

#[test]
fn grouping_at_rules_expand_their_nested_rules() {
    assert_formats(
        "@layer base,components;\n@layer base{html{color:black}}\n@container sidebar (min-width:400px){.card{display:grid}}\n@scope (.card) to (.content){img{border:1px solid}}\n@starting-style{.fade{opacity:0}}\n",
        "@layer base,components;\n@layer base {\n  html {\n    color: black;\n  }\n}\n\n@container sidebar (min-width:400px) {\n  .card {\n    display: grid;\n  }\n}\n\n@scope (.card) to (.content) {\n  img {\n    border: 1px solid;\n  }\n}\n\n@starting-style {\n  .fade {\n    opacity: 0;\n  }\n}\n",
    );
}

#[test]
fn descriptor_at_rules_expand_their_declarations() {
    assert_formats(
        "@page :first{margin:1in}\n@counter-style thumbs{system:cyclic;symbols:\"👍\";suffix:\" \"}\n@property --angle{syntax:'<angle>';inherits:false;initial-value:0deg}\n@font-feature-values Font One{@styleset{nice-style:12}}\n@position-try --top{top:anchor(bottom)}\n@view-transition{navigation:auto}\n",
        "@page :first {\n  margin: 1in;\n}\n\n@counter-style thumbs {\n  system: cyclic;\n  symbols: \"👍\";\n  suffix: \" \";\n}\n\n@property --angle {\n  syntax: '<angle>';\n  inherits: false;\n  initial-value: 0deg;\n}\n\n@font-feature-values Font One {\n  @styleset {\n    nice-style: 12;\n  }\n}\n\n@position-try --top {\n  top: anchor(bottom);\n}\n\n@view-transition {\n  navigation: auto;\n}\n",
    );
}

#[test]
fn keyframes_and_font_face_keep_their_authored_values() {
    assert_formats(
        "@font-face{font-family:\"X\";src:url(data:font/woff2;base64,AAAA) format(\"woff2\")}\n@keyframes spin{from{transform:rotate(0deg)}to{transform:rotate(360deg)}}\n",
        "@font-face {\n  font-family: \"X\";\n  src: url(data:font/woff2;base64,AAAA) format(\"woff2\");\n}\n\n@keyframes spin {\n  from {\n    transform: rotate(0deg);\n  }\n  to {\n    transform: rotate(360deg);\n  }\n}\n",
    );
}

#[test]
fn multi_value_transitions_and_shadows_break_one_value_per_line() {
    assert_formats(
        ".a{transition:opacity .3s ease,transform .3s ease;box-shadow:0 1px 2px rgba(0,0,0,.2),0 2px 4px rgba(0,0,0,.2)}\n.b{transition:color .2s}\n",
        ".a {\n  transition:\n    opacity .3s ease,\n    transform .3s ease;\n  box-shadow:\n    0 1px 2px rgba(0,0,0,.2),\n    0 2px 4px rgba(0,0,0,.2);\n}\n\n.b {\n  transition: color .2s;\n}\n",
    );
}

#[test]
fn nested_rules_selector_lists_and_media_stay_inside_their_parent() {
    assert_formats(
        ".parent{color:red;&:hover{color:blue}.child,.other{margin:0}@media (width>600px){padding:1px}}\n",
        ".parent {\n  color: red;\n  &:hover {\n    color: blue;\n  }\n  .child,\n  .other {\n    margin: 0;\n  }\n  @media (width>600px) {\n    padding: 1px;\n  }\n}\n",
    );
}

#[test]
fn crlf_input_keeps_authored_adjacency_before_a_selector_list() {
    assert_formats(
        ".a {\r\n  color: red;\r\n}\r\n.b,.c {\r\n  color: blue;\r\n}\r\n",
        ".a {\n  color: red;\n}\n.b,\n.c {\n  color: blue;\n}\n",
    );
}

#[test]
fn non_ascii_selectors_and_strings_do_not_shift_later_preludes() {
    // Parser columns count UTF-16 units; `.後` and `.x,.y` must still be
    // matched to their own preludes after astral-plane characters.
    assert_formats(
        ".日本語 {\n  content: \"🎉🎉\";\n}\n.後 {\n  color: red;\n}\n.emoji::before {\n  content: \"👍\";\n}\n.x,.y {\n  color: blue;\n}\n",
        ".日本語 {\n  content: \"🎉🎉\";\n}\n.後 {\n  color: red;\n}\n.emoji::before {\n  content: \"👍\";\n}\n.x,\n.y {\n  color: blue;\n}\n",
    );
}

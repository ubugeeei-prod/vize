use super::{LexOptions, Lexer, TestCallbacks};
use crate::markup::profile::Component;

fn saw_name_backslash(source: &str) -> bool {
    let mut lexer =
        Lexer::<Component, _>::new(source, TestCallbacks::default(), LexOptions::default());
    lexer.run();
    lexer.name_backslash()
}

#[test]
fn a_backslash_anywhere_in_a_name_is_recorded() {
    for source in [
        r#"<div \a="x"></div>"#,
        r#"<div a\b="x"></div>"#,
        r#"<div a\></div>"#,
        r#"<div :a\b="x"></div>"#,
        r#"<div @\a="x"></div>"#,
        r#"<div #a\b></div>"#,
        r#"<div v-a\b="x"></div>"#,
        r#"<div v-x:a\b="x"></div>"#,
        r#"<div @a.b\c="x"></div>"#,
        r#"<div v-x.\c></div>"#,
        r#"<div a\b"#,
    ] {
        assert!(saw_name_backslash(source), "{source}");
    }
}

#[test]
fn a_backslash_outside_names_is_not_recorded() {
    for source in [
        r#"<div a="\x" b='\y' c=\z></div>"#,
        r#"<div :[a\b]="x"></div>"#,
        r#"<div>\{{ a\b }}</div>"#,
        r#"<a\b></a\b>"#,
        r#"<!-- a\b --><div a="x"></div>"#,
    ] {
        assert!(!saw_name_backslash(source), "{source}");
    }
}

//! Template refs reach the runtime ref setter as the authored key and target.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn compile(source: &str) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result.code
}

/// The generated lines that pass something to the ref setter.
fn ref_calls(code: &str) -> std::vec::Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once('(')
                .is_some_and(|(head, _)| head == "_setRef")
        })
        .collect()
}

#[test]
fn static_ref_key_is_a_valid_string_literal() {
    let code = compile(r#"<div ref='a"b\c'></div>"#);
    assert_eq!(ref_calls(&code), [r#"_setRef(n0, "a\"b\\c")"#], "{code}");
}

#[test]
fn slot_outlet_ref_targets_the_outlet_instead_of_becoming_a_slot_prop() {
    let code = compile(r#"<slot ref="s" ref_key="k" name="x" :foo="1"></slot>"#);
    assert_eq!(ref_calls(&code), [r#"_setRef(n0, "s")"#], "{code}");
    let slot_lines: std::vec::Vec<&str> = code
        .lines()
        .filter(|line| line.split_once("_createSlot(").is_some())
        .collect();
    assert_eq!(slot_lines.len(), 1, "{code}");
    for line in slot_lines {
        assert_eq!(line.matches("ref").count(), 0, "{code}");
    }
}

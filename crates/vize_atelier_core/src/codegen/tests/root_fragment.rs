use super::result_output;
use crate::{WhitespaceStrategy, compile, parser::with_whitespace_strategy};

#[test]
fn interpolation_is_a_dynamic_text_vnode() {
    let output = result_output(&compile!("{{ label }}\n<span>!</span>"));
    let expected = "_createTextVNode(_toDisplayString(label) + \" \", 1 /* TEXT */)";

    assert_eq!(output.matches(expected).count(), 1, "{output}");
    assert_eq!(output.matches("1 /* TEXT */").count(), 1, "{output}");
}

#[test]
fn static_text_run_does_not_insert_an_array_hole() {
    let result = with_whitespace_strategy(WhitespaceStrategy::Preserve, || {
        compile!("<!-- note --> <div class=\"child\"></div>")
    });
    let output = result_output(&result);

    assert_eq!(output.matches("\n    ,").count(), 0, "{output}");
    assert_eq!(output.matches("_createTextVNode()").count(), 1, "{output}");
}

#[test]
fn interpolation_keeps_text_run_boundaries() {
    for (source, expected) in [
        (
            "<span>!</span>\n{{ label }}",
            "_createTextVNode(\" \" + _toDisplayString(label), 1 /* TEXT */)",
        ),
        (
            "{{ label }}<!-- retained --><span>!</span>",
            "_createTextVNode(_toDisplayString(label), 1 /* TEXT */)",
        ),
        (
            "{{ label }}<!-- @vize:forget marker --><span>!</span>",
            "_createTextVNode(_toDisplayString(label), 1 /* TEXT */)",
        ),
    ] {
        let output = result_output(&compile!(source));
        assert_eq!(output.matches(expected).count(), 1, "{source}:\n{output}");
        assert_eq!(output.matches("1 /* TEXT */").count(), 1, "{source}");
    }
}

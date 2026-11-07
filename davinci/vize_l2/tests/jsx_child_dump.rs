//! Distinct child semantics survive the existing canonical dump/replay contract.
use vize_l0::dump::{Dump, Mode};
use vize_l2::dump::{Op, Page};
#[test]
fn jsx_value_and_template_text_keep_distinct_tags_with_the_same_expression_payload() {
    let text = "[l2-dump-v2]\nops=2\n\n[l2-dump-v2.ops]\nui.interpolation js(\"value\" @0:5) @0:5\nui.jsx-child js(\"value\" @0:5) @0:5\n\n";
    let page = Page::parse(text).expect("both explicit semantic tags");
    assert!(matches!(page.ops[0], Op::Interpolation(_)));
    assert!(matches!(page.ops[1], Op::JsxChildValue(_)));
    assert_eq!(page.print_to_string(Mode::Full).as_str(), text);
    assert_eq!(
        Page::parse(page.print_to_string(Mode::Full).as_str()).expect("replay"),
        page
    );
}

#[test]
fn raw_html_interpolation_round_trips_as_an_op_rendering_contract() {
    let text = "[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\nui.interpolation raw-html js(\"html\" @3:7) @0:10\n\n";
    let page = Page::parse(text).expect("raw HTML interpolation");
    let Op::Interpolation(interpolation) = &page.ops[0] else {
        panic!("expected interpolation");
    };
    assert!(interpolation.raw_html);
    assert_eq!(page.print_to_string(Mode::Full).as_str(), text);
    assert_eq!(
        Page::parse(page.print_to_string(Mode::Full).as_str()).expect("replay"),
        page
    );
}

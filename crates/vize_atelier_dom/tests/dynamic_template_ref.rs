#![allow(clippy::disallowed_macros)] // `insta::assert_snapshot!` expands to `format!`.

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;

#[test]
fn dynamic_ref_in_v_for_emits_ref_for() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(
        &allocator,
        r#"<span v-for="(item, index) in items" :ref="(el) => (itemEls[index] = el)"></span>"#,
    );
    assert!(errors.is_empty());
    insta::assert_snapshot!(result.code.as_str());
}

use vize_atelier_dom::compile_template;
use vize_s0::{Allocator, String};

#[test]
fn compile_survives_deep_bound_expression_without_recursive_static_analysis() {
    let allocator = Allocator::new();
    let mut source = String::with_capacity(2_736);
    source.push_str(r#"<li :key="S"#);
    for _ in 0..2_704 {
        source.push('[');
    }
    source.push_str(r#"">item</li>"#);

    let (_, errors, result) = compile_template(&allocator, &source);

    assert!(errors.is_empty(), "unexpected compiler errors: {errors:?}");
    assert!(
        result.code.contains("S[[[["),
        "template compilation must preserve the bound expression:\n{}",
        result.code
    );
}

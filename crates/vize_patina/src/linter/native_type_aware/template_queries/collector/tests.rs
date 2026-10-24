use super::expression_binding_generated_offset;

#[test]
fn finds_expression_binding_offset_on_generated_line() {
    let generated = "  const __expr_42 = actions[method];\n";
    let expression_offset = generated.find("method").unwrap() as u32;

    let binding_offset =
        expression_binding_generated_offset(generated, expression_offset).expect("binding offset");

    assert_eq!(
        &generated[binding_offset as usize..binding_offset as usize + 1],
        "2"
    );
}

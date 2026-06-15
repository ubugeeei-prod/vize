use super::{PermittedContents, required_children};
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(PermittedContents));
    Linter::with_registry(registry)
}

// ===== Valid cases =====

#[test]
fn test_valid_inline_in_inline() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<p><span>text</span></p>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_block_in_block() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<div><p>text</p></div>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_list_with_li() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<ul><li>item</li></ul>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_list_with_component_child() {
    // A custom component inside <ul> is exempt: it typically renders an <li>.
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<ul><MyItem :key="1" /></ul>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_table_with_component_child() {
    // A custom component inside <table> is exempt: it typically renders a <tr>.
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<table><MyRow /></table>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_list_with_known_intrinsic_member_component_li() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r#"<ul><motion.li>item</motion.li></ul>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_table_structure() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(
        r#"<table><thead><tr><th>Head</th></tr></thead><tbody><tr><td>Cell</td></tr></tbody></table>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_template_wrapper_in_list() {
    let linter = create_linter();
    // <template> is allowed as a transparent wrapper inside lists
    let result = linter.lint_template_rules_only(
        r#"<ul><template v-for="item in items"><li>{{ item }}</li></template></ul>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_component_in_any_context() {
    let linter = create_linter();
    // Components are skipped: can render anything
    let result = linter.lint_template_rules_only(r#"<p><MyComponent /></p>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_nested_non_interactive() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r##"<a href="#"><span>text</span></a>"##, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_flow_content_in_anchor_when_context_allows_flow() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(
        r##"<main><a href="#"><h2>Documentation</h2><div>Read the guide</div></a></main>"##,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_select_with_options() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(
        r#"<select><option>A</option><option>B</option></select>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_select_with_optgroup() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(
        r#"<select><optgroup label="Group"><option>A</option></optgroup></select>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

// ===== Invalid: Block in inline =====

#[test]
fn test_repaired_div_after_p() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<p><div>block</div></p>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_div_in_span() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<span><div>block</div></span>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_repaired_h1_after_p() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<p><h1>heading</h1></p>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_ul_in_span() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r#"<span><ul><li>item</li></ul></span>"#, "test.vue");
    // ul in span: block_in_inline error
    // But li in ul is valid
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_repaired_flow_content_in_anchor_when_outer_context_is_phrasing() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r##"<p><a href="#"><div>block</div></a></p>"##, "test.vue");
    assert_eq!(result.error_count, 0);
}

// ===== Invalid: Interactive nesting =====

#[test]
fn test_repaired_a_in_a() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r##"<a href="#"><a href="#">nested</a></a>"##, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_repaired_button_in_button() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r#"<button><button>nested</button></button>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_button_in_a() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r##"<a href="#"><button>click</button></a>"##, "test.vue");
    assert_eq!(result.error_count, 1);
}

// ===== Invalid: List content model =====

#[test]
fn test_invalid_div_in_ul() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<ul><div>not li</div></ul>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_span_in_ol() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<ol><span>not li</span></ol>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_unknown_component_in_ul() {
    // Components are skipped: they can render anything, including an <li>
    // (consistent with `test_valid_component_in_any_context`).
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<ul><MyItem /></ul>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_unknown_member_component_in_ul() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(r#"<ul><foo.li>item</foo.li></ul>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_known_intrinsic_member_component_div_in_ul() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r#"<ul><motion.div>item</motion.div></ul>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

// ===== Invalid: Table content model =====

#[test]
fn test_repaired_div_in_table() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r#"<table><div>not valid</div></table>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_repaired_span_in_tr() {
    let linter = create_linter();
    let result = linter.lint_template_rules_only(
        r#"<table><tr><span>not td/th</span></tr></table>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

// ===== Invalid: Select content model =====

#[test]
fn test_invalid_div_in_select() {
    let linter = create_linter();
    let result =
        linter.lint_template_rules_only(r#"<select><div>not option</div></select>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

// ===== Helper function tests =====

#[test]
fn test_required_children_lookup() {
    assert_eq!(required_children("ul"), Some(["li"].as_slice()));
    assert_eq!(required_children("ol"), Some(["li"].as_slice()));
    assert_eq!(
        required_children("table"),
        Some(
            [
                "thead", "tbody", "tfoot", "tr", "caption", "colgroup", "col"
            ]
            .as_slice()
        )
    );
    assert_eq!(required_children("tr"), Some(["td", "th"].as_slice()));
    assert!(required_children("div").is_none());
}

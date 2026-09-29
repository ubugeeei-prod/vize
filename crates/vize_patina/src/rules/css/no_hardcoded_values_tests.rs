use super::NoHardcodedValues;
use crate::rules::css::CssLinter;

fn create_linter() -> CssLinter {
    let mut linter = CssLinter::new();
    linter.add_rule(Box::new(NoHardcodedValues::default()));
    linter
}

#[test]
fn test_valid_css_variable() {
    let linter = create_linter();
    let result = linter.lint(".button { color: var(--primary); }", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_hardcoded_hex() {
    let linter = create_linter();
    let result = linter.lint(".button { color: #ff0000; }", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warns_hardcoded_rgb() {
    let linter = create_linter();
    let result = linter.lint(".button { color: rgb(255, 0, 0); }", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_inherit() {
    let linter = create_linter();
    let result = linter.lint(".button { color: inherit; }", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_current_color() {
    let linter = create_linter();
    let result = linter.lint(".button { color: currentColor; }", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_transparent_color() {
    let linter = create_linter();
    let result = linter.lint(".button { background-color: transparent; }", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_system_colors_follow_forced_colors() {
    let result = create_linter().lint(
            "@media (forced-colors: active) { .button { color: CanvasText; background-color: Canvas; } }",
            0,
        );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_z_index_points_to_its_declaration() {
    let source = ".note { color: var(--note); }\n.raised {\n  z-index: 1;\n}";
    let result = create_linter().lint(source, 100);
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].start as usize,
        100 + source.find("z-index").unwrap()
    );
}

#[test]
fn test_warns_absolute_font_size() {
    let linter = create_linter();
    let result = linter.lint(".button { font-size: 16px; }", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_relative_font_size() {
    let linter = create_linter();
    let result = linter.lint(
        ".button { font-size: 1.17em; } .title { font-size: 1rem; }",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_hardcoded_z_index() {
    let linter = create_linter();
    let result = linter.lint(".modal { z-index: 9999; }", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_z_index_auto() {
    let linter = create_linter();
    let result = linter.lint(".modal { z-index: auto; }", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_nested_hardcoded_color() {
    let linter = create_linter();
    let result = linter.lint(".card { .title { color: #123456; } }", 0);
    assert_eq!(result.warning_count, 1);
}

use super::LandmarkRoles;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(LandmarkRoles));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_single_main() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<main>content</main>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_labeled_navs() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<nav aria-label="Primary">nav1</nav><nav aria-label="Footer">nav2</nav>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_dynamically_labeled_navs() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<nav :aria-label="primaryLabel">nav1</nav><nav v-bind:aria-label="footerLabel">nav2</nav>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_bound_literal_labeled_navs() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<nav :aria-label="'Primary'">nav1</nav><nav v-bind:aria-labelledby="'footer-heading'">nav2</nav>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_duplicate_bound_literal_labels() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<nav :aria-label="'Links'">nav1</nav><nav v-bind:aria-label="'Links'">nav2</nav>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_valid_single_nav() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<nav>navigation</nav>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_different_landmarks() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<header>h</header><main>m</main><footer>f</footer>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_duplicate_main() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<main>first</main><main>second</main>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_triple_main() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<main>1</main><main>2</main><main>3</main>"#, "test.vue");
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_invalid_unlabeled_navs() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<nav>nav1</nav><nav>nav2</nav>"#, "test.vue");
    // Both navs lack labels
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_invalid_one_unlabeled_nav() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<nav aria-label="Primary">nav1</nav><nav>nav2</nav>"#,
        "test.vue",
    );
    // Only the second nav lacks a label
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_role_main_duplicate() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<main>first</main><div role="main">second</div>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn unnamed_sections_are_not_region_landmarks() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<main><section><h2>First</h2></section><section><h2>Second</h2></section></main>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn named_sections_are_region_landmarks() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<section title="News">A</section><section aria-label="News">B</section>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 2);
}

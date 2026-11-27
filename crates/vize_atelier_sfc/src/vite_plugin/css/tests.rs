//! Tests for stylesheet import and asset resolution.

use super::*;

#[test]
fn resolves_custom_media_and_deep_selectors() {
    let css = "@custom-media --mobile (max-width: 768px);\n.foo { color: red; }\n@media (--mobile) { .foo :deep(.bar) { color: blue; } }";
    let result = resolve_css_imports(css, "/project/Component.vue", &[], false, None);

    assert!(!result.contains("@custom-media"));
    assert!(result.contains("@media (max-width: 768px)"));
    assert!(result.contains(".foo .bar"));
}

#[test]
fn resolves_css_imports_preserves_utf8_when_collapsing_blank_lines() {
    let css = ".icon::before { content: \"→\"; }\n\n\n.日本語 { color: red; }";
    let result = resolve_css_imports(css, "/project/Component.vue", &[], false, None);

    assert_eq!(
        result,
        ".icon::before { content: \"→\"; }\n\n.日本語 { color: red; }"
    );
}

#[test]
fn served_stylesheets_read_only_inside_their_roots() {
    let holder = tempfile::tempdir().unwrap();
    let root = holder.path().join("app");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/theme.css"), ".theme { color: red; }").unwrap();
    std::fs::write(root.join("src/logo.svg"), "<svg/>").unwrap();
    std::fs::write(
        holder.path().join("outside.css"),
        ".outside { color: blue; }",
    )
    .unwrap();
    let importer = root.join("src/App.vue");
    let importer = importer.to_str().unwrap();
    let css = "@import './theme.css';\n@import '../../outside.css';\n\
               .a { background: url(./logo.svg); }\n.b { background: url(../../outside.css); }";
    let roots = [root.clone()];

    let served = resolve_css_imports_in(css, importer, &[], true, None, Some(&roots));
    assert!(served.contains(".theme { color: red; }"));
    assert!(served.contains("@import '../../outside.css';"));
    assert!(served.contains("/@fs/"));
    assert!(served.contains("url(../../outside.css)"));

    // A build reads what the stylesheet names.
    let built = resolve_css_imports(css, importer, &[], false, None);
    assert!(built.contains(".outside { color: blue; }"));
}

#[test]
fn string_alias_does_not_match_package_prefix() {
    assert_eq!(
        matched_alias_suffix("@/asset.svg", "@").as_deref(),
        Some("asset.svg")
    );
    assert_eq!(matched_alias_suffix("@scope/icon.svg", "@"), None);
}

//! Tests for CSS compilation.

#[cfg(feature = "native")]
use std::{fs, path::PathBuf};
use vize_carton::ToCompactString;
use vize_carton::{Bump, BumpVec};

use super::scoped::{
    add_scope_to_element, apply_scoped_css, transform_deep, transform_global, transform_slotted,
};
use super::transform::extract_and_transform_v_bind;
#[cfg(feature = "native")]
use super::CssTargets;
use super::{bundle_css, compile_css, CssCompileOptions};

#[test]
fn test_compile_simple_css() {
    let css = ".foo { color: red; }";
    let result = compile_css(css, &CssCompileOptions::default());
    assert!(result.errors.is_empty());
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_compile_scoped_css() {
    let css = ".foo { color: red; }";
    let result = compile_css(
        css,
        &CssCompileOptions {
            scoped: true,
            scope_id: Some("data-v-123".to_compact_string()),
            ..Default::default()
        },
    );
    assert!(result.errors.is_empty());
    insta::assert_debug_snapshot!(result);
}

#[test]
#[cfg(feature = "native")]
fn test_compile_minified_css() {
    let css = ".foo {\n  color: red;\n  background: blue;\n}";
    let result = compile_css(
        css,
        &CssCompileOptions {
            minify: true,
            ..Default::default()
        },
    );
    assert!(result.errors.is_empty());
    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_v_bind_extraction() {
    let bump = Bump::new();
    let css = ".foo { color: v-bind(color); background: v-bind('bgColor'); }";
    let (transformed, vars) = extract_and_transform_v_bind(&bump, css);
    assert_eq!(vars.len(), 2);
    assert!(vars.contains(&"color".to_compact_string()));
    assert!(vars.contains(&"bgColor".to_compact_string()));
    insta::assert_snapshot!(transformed);
}

#[test]
fn test_scope_deep() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    transform_deep(&mut out, ":deep(.child)", 0, b"[data-v-123]");
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, "[data-v-123] .child");
}

#[test]
fn test_scope_global() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    transform_global(&mut out, ":global(.foo)", 0);
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, ".foo");
}

#[test]
fn test_scope_slotted() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    transform_slotted(&mut out, ":slotted(.child)", 0, b"[data-v-123]");
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, ".child[data-v-123-s]");
}

#[test]
fn test_scope_slotted_with_pseudo() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    transform_slotted(&mut out, ":slotted(.child):hover", 0, b"[data-v-abc]");
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, ".child[data-v-abc-s]:hover");
}

#[test]
fn test_scope_slotted_complex() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    transform_slotted(&mut out, ":slotted(div.foo)", 0, b"[data-v-12345678]");
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, "div.foo[data-v-12345678-s]");
}

#[test]
fn test_scope_with_pseudo_element() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    add_scope_to_element(&mut out, ".foo::before", b"[data-v-123]");
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, ".foo[data-v-123]::before");
}

#[test]
fn test_scope_with_pseudo_class() {
    let bump = Bump::new();
    let mut out = BumpVec::new_in(&bump);
    add_scope_to_element(&mut out, ".foo:hover", b"[data-v-123]");
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    assert_eq!(result, ".foo[data-v-123]:hover");
}

#[test]
#[cfg(feature = "native")]
fn test_compile_with_targets() {
    let css = ".foo { display: flex; }";
    let result = compile_css(
        css,
        &CssCompileOptions {
            targets: Some(CssTargets {
                chrome: Some(80),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    assert!(result.errors.is_empty());
    insta::assert_debug_snapshot!(result);
}

#[test]
#[cfg(feature = "native")]
fn test_bundle_css_inlines_imports_recursively() {
    let case_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("__agent_only")
        .join("tests")
        .join("css-bundle-native");
    let nested_dir = case_dir.join("nested");
    let entry_path = case_dir.join("entry.css");
    let base_path = nested_dir.join("base.css");
    let theme_path = case_dir.join("theme.css");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&nested_dir).unwrap();
    fs::write(&theme_path, ".theme { color: blue; }").unwrap();
    fs::write(
        &base_path,
        "@import \"../theme.css\";\n.base { display: flex; }",
    )
    .unwrap();
    fs::write(
        &entry_path,
        "@import \"./nested/base.css\";\n.entry { color: red; }",
    )
    .unwrap();

    let result = bundle_css(
        entry_path.to_string_lossy().as_ref(),
        &CssCompileOptions::default(),
    );

    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
    insta::assert_debug_snapshot!(result);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
#[cfg(not(feature = "native"))]
fn test_bundle_css_without_native_reports_error() {
    let result = bundle_css("entry.css", &CssCompileOptions::default());

    assert!(result.code.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].as_str(),
        "CSS bundling requires the `native` feature"
    );
}

#[test]
fn test_scoped_css_with_quoted_font_family() {
    let css = ".foo { font-family: 'JetBrains Mono', monospace; }";
    let result = compile_css(
        css,
        &CssCompileOptions {
            scoped: true,
            scope_id: Some("data-v-123".to_compact_string()),
            ..Default::default()
        },
    );
    println!("Result: {}", result.code);
    assert!(result.errors.is_empty());
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_apply_scoped_css_at_media() {
    let bump = Bump::new();
    // Root-level @media with selectors inside
    let css = ".foo { color: red; }\n@media (max-width: 768px) { .foo { color: blue; } }";
    let result = apply_scoped_css(&bump, css, "data-v-123");
    println!("@media result: {}", result);
    insta::assert_snapshot!(result);
}

#[test]
fn test_apply_scoped_css_at_media_custom_media() {
    let bump = Bump::new();
    // @media with custom media queries (like --mobile)
    let css = ".a { color: red; }\n@media (--mobile) { .a { font-size: 12px; } }";
    let result = apply_scoped_css(&bump, css, "data-v-abc");
    println!("Custom media result: {}", result);
    insta::assert_snapshot!(result);
}

#[test]
fn test_apply_scoped_css_multiple_selectors_in_media() {
    let bump = Bump::new();
    // Multiple selectors inside @media
    let css = "@media (--mobile) { .a { color: red; } .b { color: blue; } }";
    let result = apply_scoped_css(&bump, css, "data-v-xyz");
    println!("Multi selector result: {}", result);
    insta::assert_snapshot!(result);
}

#[test]
fn test_apply_scoped_css_with_quoted_string() {
    let bump = Bump::new();
    // Test the raw scoping function without LightningCSS
    let css = ".foo { font-family: 'JetBrains Mono', monospace; }";
    let result = apply_scoped_css(&bump, css, "data-v-123");
    println!("Scoped result: {}", result);
    insta::assert_snapshot!(result);
}

#[test]
fn test_apply_scoped_css_at_import() {
    let bump = Bump::new();
    // @import should be preserved and not treated as a block at-rule
    let css = "@import \"~/assets/styles/custom-media-query.css\";\n\nfooter { width: 100%; }";
    let result = apply_scoped_css(&bump, css, "data-v-123");
    println!("@import result: {}", result);
    insta::assert_snapshot!(result);
}

#[test]
fn test_apply_scoped_css_at_import_with_nested_css() {
    let bump = Bump::new();
    // @import followed by CSS nesting with @media
    let css = "@import \"custom.css\";\n\nfooter {\n  width: 100%;\n  @media (--mobile) {\n    padding: 1rem;\n  }\n}";
    let result = apply_scoped_css(&bump, css, "data-v-abc");
    println!("@import + nesting result: {}", result);
    insta::assert_snapshot!(result);
}

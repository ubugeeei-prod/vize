//! P2-11 installment 101 witness: **`isCustomElement`**.
//!
//! A matching tag stays an *element* that the tag rules would otherwise
//! make a component. The shipped lane settles this on `tag_type` in
//! `lane/element.rs` before its transform runs — after the
//! registered-component lookup, before the `component` / PascalCase /
//! hyphen / `is` heuristic — so this lane answers it in the **lowering**
//! rather than at emit time: a component op cannot be printed as an
//! element without duplicating the element path.
//!
//! The matcher lives in `vize_relief`, which the davinci stage crates do
//! not depend on, so `CustomElementPatterns` mirrors its glob rule. This
//! witness is the only place that can see both, and pins them against
//! each other over a shared pattern battery. Compared byte-for-byte with
//! the shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::ErrorCode;
use vize_atelier_core::options::{
    CodegenMode, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_and_template_syntax_and_codegen_options,
};
use vize_s0::{Allocator, String};
use vize_s1_to_s2::{CustomElementPatterns, DomEmitMode, DomEmitOptions};

/// `(name, patterns, template)`.
const BATTERY: &[(&str, &[&str], &str)] = &[
    // The shape the CLI test drives: a matched PascalCase tree.
    (
        "tres_tree",
        &["Tres*"],
        r#"<TresCanvas><TresMesh/><TresSpotLight/></TresCanvas>"#,
    ),
    // Only the matched tag flips; the unmatched sibling stays a component.
    (
        "matched_beside_unmatched",
        &["Tres*"],
        r#"<div><TresMesh/><OtherThing/></div>"#,
    ),
    // Exact patterns, and a hyphenated tag that would be a component.
    (
        "exact_hyphenated",
        &["ion-button"],
        r#"<div><ion-button/><ion-card/></div>"#,
    ),
    // A leading wildcard.
    ("leading_wildcard", &["*-icon"], r#"<app-icon/><app-card/>"#),
    // Several patterns at once.
    (
        "several_patterns",
        &["Tres*", "ion-*"],
        r#"<div><TresMesh/><ion-button/><Other/></div>"#,
    ),
    // Matched tags keep their props, children and directives as elements.
    (
        "matched_with_props_and_children",
        &["Tres*"],
        r#"<TresMesh :position="[0, 1, 2]" cast-shadow><span>x</span></TresMesh>"#,
    ),
    (
        "matched_with_structural_directives",
        &["Tres*"],
        r#"<TresMesh v-if="on"/><TresMesh v-else v-for="i in items" :key="i"/>"#,
    ),
    // A native tag is unaffected either way.
    ("native_unaffected", &["div"], r#"<div><p>x</p></div>"#),
    // No pattern at all: the rule is inert.
    ("no_patterns", &[], r#"<TresMesh/><div/>"#),
];

fn dom_options() -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        ..Default::default()
    }
}

fn shipped(src: &str, patterns: &[&str]) -> String {
    let allocator = Allocator::new();
    let matcher =
        CustomElementMatcher::from_patterns(patterns.iter().map(|p| String::from(*p)).collect());
    let (_, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_codegen_options(
            &allocator,
            src,
            dom_options(),
            TemplateSyntaxMode::Standard,
            matcher,
            CodegenOptions::default(),
        );
    // Treating a matched tag as an element makes `<Tag/>` invalid
    // self-closing HTML, which the shipped parser rewrites and records as
    // an extend point. That note is the option working, not a failure.
    let hard: Vec<_> = errors
        .iter()
        .filter(|error| !matches!(error.code, ErrorCode::ExtendPoint))
        .collect();
    assert!(hard.is_empty(), "shipped lane errors: {hard:?}");
    let mut out = String::from(result.preamble.as_str());
    out.push('\n');
    out.push_str(result.code.as_str());
    out
}

fn s2(src: &str, patterns: &[&str]) -> String {
    let allocator = Allocator::new();
    let options = DomEmitOptions {
        mode: DomEmitMode::Module,
        custom_elements: Some(CustomElementPatterns::new(patterns)),
        ..DomEmitOptions::DEFAULT
    };
    vize_s1_to_s2::emit_dom_source_with_options(
        &allocator,
        src,
        vize_s1_to_s2::LegacyCaps::VUE3,
        &options,
    )
    .expect("custom elements witness must emit")
    .assembled()
}

#[test]
fn custom_elements_match_the_shipped_dom_lane() {
    for (name, patterns, src) in BATTERY {
        assert_eq!(
            shipped(src, patterns).as_str(),
            s2(src, patterns).as_str(),
            "{name}: S2 DOM emit diverged from the shipped lane"
        );
    }
}

/// With the rule off, every case has to emit exactly what it emitted
/// before the option existed — the option costs nothing when unset.
#[test]
fn an_unset_rule_changes_nothing() {
    for (name, _, src) in BATTERY {
        assert_eq!(
            shipped(src, &[]).as_str(),
            s2(src, &[]).as_str(),
            "{name}: S2 DOM emit diverged with no custom-element rule"
        );
    }
}

/// The mirrored glob against the one it mirrors. This is the only crate
/// that can see both.
#[test]
fn the_mirrored_glob_answers_like_the_shipped_matcher() {
    const PATTERNS: &[&str] = &[
        "Tres*",
        "*-icon",
        "ion-button",
        "*",
        "",
        "a*z",
        "*mid*",
        "x*y*z",
    ];
    const TAGS: &[&str] = &[
        "TresMesh",
        "Tres",
        "MyTresMesh",
        "app-icon",
        "app-icons",
        "ion-button",
        "ion-buttons",
        "abcz",
        "az",
        "abc",
        "amidz",
        "mid",
        "xyz",
        "xAyBz",
        "xz",
        "",
        "div",
    ];
    for pattern in PATTERNS {
        let mine = CustomElementPatterns::new(core::slice::from_ref(pattern));
        let theirs = CustomElementMatcher::from_patterns(vec![String::from(*pattern)]);
        for tag in TAGS {
            assert_eq!(
                (pattern, tag, mine.matches(tag)),
                (pattern, tag, theirs.matches(tag)),
            );
        }
    }
}

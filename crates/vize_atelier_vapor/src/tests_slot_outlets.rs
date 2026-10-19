//! Snapshot coverage for `<slot>` outlet lowering in the Vapor compiler.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit; the small assertion helpers mirror the ones
//! defined there.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use super::compile_vapor;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn normalize_code(code: &str) -> String {
    code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_parses_as_module(code: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        code,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    )
    .parse();

    assert!(
        parsed.diagnostics.is_empty(),
        "generated code should parse, got: {:?}\n\n{}",
        parsed.diagnostics,
        code
    );
}

#[test]
fn test_compile_bare_default_slot_outlet() {
    let allocator = vize_carton::Allocator::new();
    let result = compile_vapor(&allocator, r#"<slot />"#, Default::default());

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn test_compile_named_slot_outlet_without_props() {
    let allocator = vize_carton::Allocator::new();
    let result = compile_vapor(&allocator, r#"<slot name="head" />"#, Default::default());

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn test_compile_default_slot_outlet_with_fallback_only() {
    let allocator = vize_carton::Allocator::new();
    let result = compile_vapor(
        &allocator,
        r#"<slot><div>fb</div></slot>"#,
        Default::default(),
    );

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn test_compile_slot_outlet_static_and_spread_props() {
    let allocator = vize_carton::Allocator::new();
    let result = compile_vapor(
        &allocator,
        r#"<slot kind="primary" :row="item" v-bind="extra" />"#,
        Default::default(),
    );

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn forwarding_slot_roots_match_the_retained_lane_and_runtime_flags() {
    // Exact RC.9 compiler-vapor boundaries: a stable sibling prevents
    // forwarding, while an outlet-only root inherits its owner's fallback.
    // Preserve direct-outlet loop refusal, and independently require native
    // admission for its equivalent authored template-loop neighbor.
    for (source, expected, legacy_reason) in [
        ("<slot></slot>", [0, 0, 0], None),
        ("<Comp><slot></slot></Comp>", [1, 0, 1], None),
        ("<Comp><div><slot></slot></div></Comp>", [0, 0, 0], None),
        ("<Comp><slot></slot><b>tail</b></Comp>", [0, 0, 0], None),
        ("<Comp><!--x--><slot></slot></Comp>", [1, 0, 1], None),
        (
            "<Comp><slot></slot><slot name=\"other\"></slot></Comp>",
            [0, 2, 1],
            None,
        ),
        ("<Comp><slot v-if=\"ok\"></slot></Comp>", [1, 0, 1], None),
        (
            "<Comp><slot v-for=\"item in items\" :key=\"item\"></slot></Comp>",
            [0, 1, 1],
            Some(crate::l3::LegacyReason::ControlFlow),
        ),
        (
            "<Comp><slot><slot name=\"other\"></slot></slot></Comp>",
            [2, 0, 1],
            None,
        ),
        ("<Comp><Child><slot></slot></Child></Comp>", [1, 0, 1], None),
        (
            "<Comp><component :is=\"which\"><slot></slot></component></Comp>",
            [1, 0, 2],
            None,
        ),
        (
            "<Comp><template v-for=\"item in items\" :key=\"item\"><slot></slot></template></Comp>",
            [0, 1, 1],
            None,
        ),
    ] {
        let allocator = vize_carton::Allocator::new();
        let status = crate::l3::lower_source_for_vapor(
            &allocator,
            source,
            crate::l3::VaporL3BridgeOptions {
                ssr: false,
                custom_renderer: false,
                experimental_in_tag_comments: false,
                experimental_patterned_template: false,
                template_syntax: vize_atelier_core::TemplateSyntaxMode::Standard,
                has_custom_elements: false,
                prefixed_binding_metadata: false,
                retained_lane: false,
                inline: false,
            },
        );
        match legacy_reason {
            None => assert!(
                matches!(status, crate::l3::VaporL3BridgeStatus::Accepted(_)),
                "the native regression must be admitted: {source}"
            ),
            Some(reason) => assert!(
                matches!(status, crate::l3::VaporL3BridgeStatus::Legacy(actual) if actual == reason),
                "the unsupported direct outlet must retain exact {reason:?}: {source}"
            ),
        }
        let selected = compile_vapor(&allocator, source, Default::default());
        let retained = compile_vapor(
            &allocator,
            source,
            crate::VaporCompilerOptions {
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert_eq!(selected.error_messages, Vec::<String>::new(), "{source}");
        assert_eq!(retained.error_messages, Vec::<String>::new(), "{source}");
        assert_eq!(selected.code, retained.code, "{source}");
        assert_parses_as_module(&selected.code);
        assert_eq!(
            [
                selected.code.matches("4 /* FORWARDED */").count(),
                selected
                    .code
                    .matches("12 /* FORWARDED, SHARED_FALLBACK */")
                    .count(),
                selected.code.matches("_extend(").count(),
            ],
            expected,
            "{source}: complete generated code:\n{}",
            selected.code
        );
    }
}

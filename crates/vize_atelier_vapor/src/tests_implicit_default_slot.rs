//! Reporter regression #7570: implicit default content beside named slots.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "regression assertions use std strings and format"
)]

use super::{compile_vapor, ir::OperationNode, lower::transform_to_ir};
use vize_atelier_core::parser::parse;

fn assert_valid_code(code: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(
        &allocator,
        code,
        oxc_span::SourceType::mjs().with_typescript(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{code}",
        parsed.diagnostics
    );
}

#[test]
fn reported_implicit_default_slot_survives_in_ir_and_generated_code() {
    let allocator = vize_carton::Allocator::new();
    let source = include_str!("../tests/fixtures/implicit-default-slot-named.input.txt").trim();
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty());
    let ir = transform_to_ir(&allocator, &root, source);
    let component = ir
        .block
        .operation
        .iter()
        .find_map(|op| match op {
            OperationNode::CreateComponent(node) => Some(node),
            _ => None,
        })
        .expect("Card component");
    assert_eq!(component.slots.len(), 2);
    assert_eq!(component.slots[0].name.content, "footer");
    assert_eq!(component.slots[1].name.content, "default");
    assert_eq!(component.slots[0].block.returns.len(), 1);
    assert_eq!(component.slots[1].block.returns.len(), 1);
    assert!(!component.slots[1].block.effect.is_empty());

    let result = compile_vapor(&allocator, source, Default::default());
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    assert!(
        result.code.contains("\"footer\": (_slotProps0) => {"),
        "{}",
        result.code
    );
    assert!(result.code.contains("_slotProps0.ok"), "{}", result.code);
    assert!(
        result.code.contains("\"default\": () => {"),
        "{}",
        result.code
    );
    assert!(result.code.contains("_ctx.x"), "{}", result.code);
    assert_valid_code(&result.code);
}

#[test]
fn implicit_default_content_preserves_order_and_excludes_slot_carriers() {
    for source in [
        "<Card><p>A</p><template #footer>F</template><p>B</p></Card>",
        "<Card><template #footer>F</template><p>A</p><p>B</p></Card>",
        "<Card><p>A</p><p>B</p><template #footer>F</template></Card>",
        "<Card>A {{ x }}<template #footer>F</template>B {{ y }}</Card>",
        "<Card><template #footer v-if=\"ok\">F</template><p>{{ x }}</p></Card>",
        "<Card><template #[name]>F</template><p>{{ x }}</p></Card>",
    ] {
        let allocator = vize_carton::Allocator::new();
        let result = compile_vapor(&allocator, source, Default::default());
        assert!(
            result.error_messages.is_empty(),
            "{source}: {:?}",
            result.error_messages
        );
        assert_eq!(
            result.code.matches("\"default\": () => {").count(),
            1,
            "{source}: {}",
            result.code
        );
        assert!(
            !result
                .templates
                .iter()
                .any(|template| template.contains("<template")),
            "{source}"
        );
        if source.contains("<p>A</p>") {
            let a = result
                .templates
                .iter()
                .position(|t| t == "<p>A</p>")
                .unwrap();
            let b = result
                .templates
                .iter()
                .position(|t| t == "<p>B</p>")
                .unwrap();
            assert!(a < b, "{source}");
        }
        assert_valid_code(&result.code);
    }
}

#[test]
fn named_only_and_explicit_default_slots_keep_their_existing_shape() {
    for (source, expected_defaults) in [
        ("<Card><template #footer>F</template></Card>", 0),
        (
            "<Card>\n<!-- note -->\n<template #footer>F</template>\n</Card>",
            0,
        ),
        (
            "<Card><template #default>D</template><template #footer>F</template></Card>",
            1,
        ),
    ] {
        let allocator = vize_carton::Allocator::new();
        let result = compile_vapor(&allocator, source, Default::default());
        assert!(
            result.error_messages.is_empty(),
            "{source}: {:?}",
            result.error_messages
        );
        assert_eq!(
            result.code.matches("\"default\": () => {").count(),
            expected_defaults,
            "{source}: {}",
            result.code
        );
        assert_valid_code(&result.code);
    }
}

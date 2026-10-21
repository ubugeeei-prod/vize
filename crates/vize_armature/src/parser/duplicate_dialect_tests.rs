//! Repeated attributes: Vue 3 keeps the first value, Vue 2 the last.

use super::parse_with_options;
use vize_l0::{Allocator, ErrorCode, String, config::VueVersion};
use vize_relief::{PropNode, TemplateChildNode, options::ParserOptions};

fn class_values(
    allocator: &Allocator,
    dialect: VueVersion,
) -> (std::vec::Vec<&str>, std::vec::Vec<String>) {
    let options = ParserOptions {
        dialect,
        ..ParserOptions::default()
    };
    let (root, errors) = parse_with_options(
        allocator,
        r#"<h4 class="" CLASS="shop_title">x</h4>"#,
        options,
    );
    let values = match root.children.first() {
        Some(TemplateChildNode::Element(element)) => element
            .props
            .iter()
            .filter_map(|prop| match prop {
                PropNode::Attribute(attr) => attr.value.as_ref().map(|value| value.content),
                _ => None,
            })
            .collect(),
        _ => std::vec::Vec::new(),
    };
    let diagnostics = errors
        .iter()
        .filter(|error| error.code == ErrorCode::DuplicateAttribute && error.is_recoverable())
        .map(|error| error.message.clone())
        .collect();
    (values, diagnostics)
}

#[test]
fn vue3_keeps_every_repeat_for_first_wins_codegen() {
    let allocator = Allocator::new();
    assert_eq!(
        class_values(&allocator, VueVersion::V3),
        (
            vec!["", "shop_title"],
            vec![String::from(
                "Duplicate attribute `CLASS`. Keeping the repeated attribute so parsing can continue."
            )]
        )
    );
}

#[test]
fn vue2_dialects_keep_the_last_value() {
    for dialect in [VueVersion::V2_7, VueVersion::V2] {
        let allocator = Allocator::new();
        assert_eq!(
            class_values(&allocator, dialect),
            (
                vec!["shop_title"],
                vec![String::from(
                    "Duplicate attribute `CLASS`. Vue 2 keeps the last value."
                )]
            ),
            "{dialect:?}"
        );
    }
}

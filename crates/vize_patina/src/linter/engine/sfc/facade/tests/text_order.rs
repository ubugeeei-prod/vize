//! Attribute and child-text warnings retain the legacy element-hook order.

use crate::linter::Linter;
use crate::rule::RuleRegistry;
use crate::rules::{html::DeprecatedAttr, vue::NoBareStringsInTemplate};

#[test]
fn text_and_attribute_rules_preserve_both_registry_orders() {
    for reversed in [false, true] {
        let mut registry = RuleRegistry::new();
        if reversed {
            registry.register(Box::new(NoBareStringsInTemplate));
            registry.register(Box::new(DeprecatedAttr));
        } else {
            registry.register(Box::new(DeprecatedAttr));
            registry.register(Box::new(NoBareStringsInTemplate));
        }
        let linter = Linter::with_registry(registry);
        for source in [
            r#"<img alt="Hello" title="World" align="left"/>"#,
            r#"<input placeholder="Find" align="left"/>"#,
            r#"<table summary="Description"><tr><td>Save</td></tr></table>"#,
            r#"<div title="日本語😀" align="center">hello {{ value }} world</div>"#,
        ] {
            let mut expected = linter.lint_template(source, "test.vue");
            crate::linter::engine::offset_result(&mut expected, 10);
            let sfc = vize_l0::cstr!("<template>{source}</template>");
            let actual = linter.lint_sfc(&sfc, "test.vue");
            assert_eq!(
                vize_l0::cstr!("{:?}", actual.diagnostics),
                vize_l0::cstr!("{:?}", expected.diagnostics),
                "{source}; reversed={reversed}"
            );
        }
    }
}

use super::{generate_element_template, generate_element_template_spanned};
use crate::lower::key::classify;
use vize_atelier_core::{TemplateChildNode, parser::parse};
use vize_carton::Allocator;

#[test]
fn classified_root_scope_preserves_full_templates_and_recursive_child_facts() {
    let cases = [
        (
            "<section><input :key=\"child\" /></section>",
            false,
            true,
            false,
            "<section></section>",
        ),
        (
            "<section v-once><input :key=\"child\" /></section>",
            false,
            true,
            true,
            "<section><input></section>",
        ),
        (
            "<section v-memo=\"deps\" v-memo=\"[]\"><input :key=\"child\" /></section>",
            false,
            true,
            true,
            "<section><input></section>",
        ),
        (
            "<section :key=\"epoch\"><input :key=\"child\" /></section>",
            false,
            false,
            false,
            "<section></section>",
        ),
        (
            "<section><div v-once><input :key=\"child\" /></div></section>",
            false,
            true,
            false,
            "<section><div><input></div></section>",
        ),
        (
            "<section><input :key=\"child\" /><p>after</p></section>",
            false,
            true,
            false,
            "<section><!----><p>after</p></section>",
        ),
        (
            "<section><input :key=\"child\" /></section>",
            true,
            true,
            true,
            "<section><input></section>",
        ),
    ];
    for (source, inherited, own_key, expected_scope, expected) in cases {
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "{errors:?}");
        let TemplateChildNode::Element(el) = &root.children[0] else {
            panic!("the complete original control must own its element");
        };
        let facts = classify(el, inherited, own_key);
        assert_eq!(facts.key_non_reactive, expected_scope, "{source}");
        let plain = generate_element_template(el, None, source, facts.key_non_reactive);
        let mapped = generate_element_template_spanned(el, None, source, facts.key_non_reactive);
        assert_eq!(plain.as_str(), expected, "{source}");
        assert_eq!(mapped.as_str(), expected, "{source}");
    }
}

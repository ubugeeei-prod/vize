use super::{classify, value};
use vize_atelier_core::{ElementType, TemplateChildNode, parser::parse};
use vize_carton::Allocator;

#[test]
fn refused_key_targets_keep_original_once_and_first_memo_diagnostics() {
    let memo_error = "v-memo with dependencies is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.";
    let cases = [
        (
            "<slot :key=\"epoch\"></slot>",
            ElementType::SlotOutlet,
            false,
            None,
        ),
        (
            "<template #default :key=\"epoch\"><i>held</i></template>",
            ElementType::Template,
            false,
            None,
        ),
        (
            "<component :is=\"selected\" :key=\"epoch\"></component>",
            ElementType::Element,
            false,
            None,
        ),
        (
            "<KeepAlive :key=\"epoch\"><i>held</i></KeepAlive>",
            ElementType::Component,
            false,
            None,
        ),
        (
            "<slot v-memo=\"deps\" :key=\"epoch\"></slot>",
            ElementType::SlotOutlet,
            false,
            Some(memo_error),
        ),
        (
            "<slot v-memo=\"deps\" v-once :key=\"epoch\"></slot>",
            ElementType::SlotOutlet,
            true,
            None,
        ),
    ];
    for (source, expected_type, expected_once, expected_error) in cases {
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "{errors:?}");
        let TemplateChildNode::Element(el) = &root.children[0] else {
            panic!("the complete original control must own its element");
        };
        assert_eq!(el.tag_type, expected_type, "{source}");
        let facts = classify(el, false, true);
        assert_eq!(facts.key.map(|key| key.content), None, "{source}");
        assert_eq!(value(el, false).map(|key| key.content), None, "{source}");
        assert_eq!(facts.should_lower_as_once, expected_once, "{source}");
        assert_eq!(facts.memo_error, expected_error, "{source}");
    }
}

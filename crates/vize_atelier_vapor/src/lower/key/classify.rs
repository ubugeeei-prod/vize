//! One existing property walk retains directive facts and the original key.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode, SimpleExpressionNode};
use vize_carton::String;

pub(in crate::lower) struct DirectiveAnalysis<'a, 'b> {
    pub(in crate::lower) should_lower_as_once: bool,
    pub(in crate::lower) memo_error: Option<String>,
    pub(in crate::lower) key: Option<&'b SimpleExpressionNode<'a>>,
}

pub(in crate::lower) fn classify<'a, 'b>(
    el: &'b ElementNode<'a>,
    inherited: bool,
    own_key: bool,
) -> DirectiveAnalysis<'a, 'b> {
    let read_key = own_key && !inherited;
    let mut key = None;
    let mut has_once = false;
    let mut first_memo = None;
    let mut has_for = false;
    let mut key_non_reactive = inherited;
    for prop in el.props.iter() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        match dir.name {
            "once" => {
                has_once = true;
                key_non_reactive = true;
            }
            "memo" => {
                if first_memo.is_none() {
                    first_memo = Some(dir);
                }
                key_non_reactive |= matches!(dir.exp.as_ref(), Some(ExpressionNode::Simple(exp))
                    if exp.content.trim() == "[]");
            }
            "for" if read_key => has_for = true,
            "bind" if read_key && key.is_none() => key = super::binding(dir),
            _ => {}
        }
    }
    let (should_lower_as_once, memo_error) = if has_once {
        (true, None)
    } else if let Some(dir) = first_memo {
        match dir.exp.as_ref() {
            Some(ExpressionNode::Simple(exp)) if exp.content.trim() == "[]" => (true, None),
            Some(ExpressionNode::Simple(_)) => (
                false,
                Some(String::from(
                    "v-memo with dependencies is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
                )),
            ),
            _ => (
                false,
                Some(String::from(
                    "v-memo is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
                )),
            ),
        }
    } else {
        (false, None)
    };
    DirectiveAnalysis {
        should_lower_as_once,
        memo_error,
        key: key.filter(|value| !key_non_reactive && !has_for && super::eligible_value(el, value)),
    }
}

#[cfg(test)]
mod tests {
    use super::classify;
    use vize_atelier_core::{TemplateChildNode, parser::parse};
    use vize_carton::Allocator;

    #[test]
    fn original_directives_keep_once_precedence_key_identity_and_refusals() {
        let memo = "v-memo with dependencies is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.";
        let missing = "v-memo is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.";
        let cases = [
            ("<i>ordinary</i>", false, true, None, false, None),
            (
                "<input title=\"a\" :value=\"value\" :key=\"epoch\" />",
                false,
                true,
                Some("epoch"),
                false,
                None,
            ),
            (
                "<Counter :key=\"epoch\" @click=\"event\" />",
                false,
                true,
                Some("epoch"),
                false,
                None,
            ),
            ("<i v-once :key=\"epoch\" />", false, true, None, true, None),
            ("<i :key=\"epoch\" v-once />", false, true, None, true, None),
            (
                "<i v-memo=\" [] \" :key=\"epoch\" />",
                false,
                true,
                None,
                true,
                None,
            ),
            (
                "<i :key=\"epoch\" v-memo=\"[dep]\" />",
                false,
                true,
                Some("epoch"),
                false,
                Some(memo),
            ),
            (
                "<i v-memo :key=\"epoch\" />",
                false,
                true,
                Some("epoch"),
                false,
                Some(missing),
            ),
            (
                "<i v-memo=\"[dep]\" :key=\"epoch\" v-once />",
                false,
                true,
                None,
                true,
                None,
            ),
            (
                "<i v-once :key=\"epoch\" v-memo=\"[dep]\" />",
                false,
                true,
                None,
                true,
                None,
            ),
            ("<i :key=\"epoch\" />", true, true, None, false, None),
            ("<i :key=\"epoch\" />", false, false, None, false, None),
            (
                "<i v-for=\"item in items\" :key=\"item.id\" />",
                false,
                true,
                None,
                false,
                None,
            ),
            ("<i :key=\"true\" />", false, true, None, false, None),
            ("<i key=\"literal\" />", false, true, None, false, None),
            (
                "<component :key=\"epoch\" :is=\"kind\" />",
                false,
                true,
                None,
                false,
                None,
            ),
            (
                "<Transition :key=\"epoch\" />",
                false,
                true,
                None,
                false,
                None,
            ),
            (
                "<template #default :key=\"epoch\" />",
                false,
                true,
                None,
                false,
                None,
            ),
        ];
        for (source, inherited, own_key, expected_key, expected_once, expected_error) in cases {
            let allocator = Allocator::new();
            let (root, errors) = parse(&allocator, source);
            assert!(errors.is_empty(), "{source}: {errors:?}");
            let [TemplateChildNode::Element(el)] = root.children.as_slice() else {
                panic!("one original element: {source}")
            };
            let facts = classify(el, inherited, own_key);
            assert_eq!(facts.should_lower_as_once, expected_once, "{source}");
            assert_eq!(facts.memo_error.as_deref(), expected_error, "{source}");
            assert_eq!(facts.key.map(|key| key.content), expected_key, "{source}");
            if let Some(key) = facts.key {
                assert_eq!(key.loc.span.slice(source), key.content, "{source}");
                assert_eq!(
                    key.loc.span.slice(source).as_ptr(),
                    key.content.as_ptr(),
                    "{source}"
                );
            }
        }
    }
}

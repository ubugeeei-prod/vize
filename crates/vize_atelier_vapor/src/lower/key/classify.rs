//! One existing property walk retains directive facts and the original key.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode, SimpleExpressionNode};

pub(in crate::lower) struct DirectiveAnalysis<'a, 'b> {
    pub(in crate::lower) should_lower_as_once: bool,
    pub(in crate::lower) memo_error: Option<&'static str>,
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
            "bind" if read_key && key.is_none() => key = super::binding_value(dir),
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
                Some(
                    "v-memo with dependencies is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
                ),
            ),
            _ => (
                false,
                Some(
                    "v-memo is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
                ),
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
    use vize_atelier_core::{ErrorCode, TemplateChildNode, parser::parse};
    use vize_carton::{Allocator, Span};

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
        // Keep every original Vue spelling. The direct HTML parser recovers
        // these self-closing non-void elements with its existing surface note.
        let recovered = [
            ("<i v-once :key=\"epoch\" />", 25),
            ("<i :key=\"epoch\" v-once />", 25),
            ("<i v-memo=\" [] \" :key=\"epoch\" />", 32),
            ("<i :key=\"epoch\" v-memo=\"[dep]\" />", 33),
            ("<i v-memo :key=\"epoch\" />", 25),
            ("<i v-memo=\"[dep]\" :key=\"epoch\" v-once />", 40),
            ("<i v-once :key=\"epoch\" v-memo=\"[dep]\" />", 40),
            ("<i :key=\"epoch\" />", 18),
            ("<i v-for=\"item in items\" :key=\"item.id\" />", 42),
            ("<i :key=\"true\" />", 17),
            ("<i key=\"literal\" />", 19),
            ("<component :key=\"epoch\" :is=\"kind\" />", 37),
        ];
        // Independently authored paired-tag counterparts exercise the same
        // directive facts without that separate parser recovery boundary.
        let paired = [
            (
                "<i v-once :key=\"epoch\"></i>",
                false,
                true,
                None,
                true,
                None,
            ),
            (
                "<i :key=\"epoch\" v-once></i>",
                false,
                true,
                None,
                true,
                None,
            ),
            (
                "<i v-memo=\" [] \" :key=\"epoch\"></i>",
                false,
                true,
                None,
                true,
                None,
            ),
            (
                "<i :key=\"epoch\" v-memo=\"[dep]\"></i>",
                false,
                true,
                Some("epoch"),
                false,
                Some(memo),
            ),
            (
                "<i v-memo :key=\"epoch\"></i>",
                false,
                true,
                Some("epoch"),
                false,
                Some(missing),
            ),
            (
                "<i v-memo=\"[dep]\" :key=\"epoch\" v-once></i>",
                false,
                true,
                None,
                true,
                None,
            ),
            (
                "<i v-once :key=\"epoch\" v-memo=\"[dep]\"></i>",
                false,
                true,
                None,
                true,
                None,
            ),
            ("<i :key=\"epoch\"></i>", true, true, None, false, None),
            ("<i :key=\"epoch\"></i>", false, false, None, false, None),
            (
                "<i v-for=\"item in items\" :key=\"item.id\"></i>",
                false,
                true,
                None,
                false,
                None,
            ),
            ("<i :key=\"true\"></i>", false, true, None, false, None),
            ("<i key=\"literal\"></i>", false, true, None, false, None),
            (
                "<component :key=\"epoch\" :is=\"kind\"></component>",
                false,
                true,
                None,
                false,
                None,
            ),
        ];
        for (source, inherited, own_key, expected_key, expected_once, expected_error) in
            cases.into_iter().chain(paired)
        {
            let allocator = Allocator::new();
            let (root, errors) = parse(&allocator, source);
            let expected_recovery = recovered
                .iter()
                .find_map(|(original, end)| (*original == source).then_some(*end));
            assert_eq!(
                errors.len(),
                usize::from(expected_recovery.is_some()),
                "{source}: {errors:?}"
            );
            if let Some(end) = expected_recovery {
                let [error] = errors.as_slice() else {
                    panic!("one original recovery: {source}")
                };
                assert_eq!(error.code, ErrorCode::ExtendPoint, "{source}");
                assert_eq!(
                    error.message.as_str(),
                    "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.",
                    "{source}"
                );
                assert_eq!(
                    error.loc.as_ref().map(|loc| loc.span),
                    Some(Span::new(0, end)),
                    "{source}"
                );
                assert_eq!(Span::new(0, end).slice(source), source, "{source}");
            }
            let [TemplateChildNode::Element(el)] = root.children.as_slice() else {
                panic!("one original element: {source}")
            };
            let facts = classify(el, inherited, own_key);
            assert_eq!(facts.should_lower_as_once, expected_once, "{source}");
            assert_eq!(facts.memo_error, expected_error, "{source}");
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

//! One existing property walk retains directive facts and the original key.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode, SimpleExpressionNode};

pub(in crate::lower) struct DirectiveAnalysis<'a, 'b> {
    pub(in crate::lower) should_lower_as_once: bool,
    pub(in crate::lower) memo_error: Option<&'static str>,
    pub(in crate::lower) key: Option<&'b SimpleExpressionNode<'a>>,
    /// ANY once/empty memo for this writer root; FIRST memo still owns errors.
    pub(in crate::lower) key_non_reactive: bool,
}

pub(in crate::lower) fn classify<'a, 'b>(
    el: &'b ElementNode<'a>,
    inherited: bool,
    own_key: bool,
) -> DirectiveAnalysis<'a, 'b> {
    if own_key && !inherited && super::eligible_target(el) {
        classify_role::<true>(el, inherited)
    } else {
        classify_role::<false>(el, inherited)
    }
}

// Hoist the actual target/scope role once; both paths retain the same property walk.
fn classify_role<'a, 'b, const READ_KEY: bool>(
    el: &'b ElementNode<'a>,
    inherited: bool,
) -> DirectiveAnalysis<'a, 'b> {
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
            "for" if READ_KEY => has_for = true,
            "bind" if READ_KEY && key.is_none() => key = super::binding_value(dir),
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
        key: key.filter(|value| !key_non_reactive && !has_for && super::eligible_value(value)),
        key_non_reactive,
    }
}

#[cfg(test)]
mod tests;

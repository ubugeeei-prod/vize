//! Original once precedence and first memo diagnostics, shared inside required walks.
use vize_atelier_core::{DirectiveNode, ExpressionNode};

pub(in crate::lower) struct ScopeDirectives<'a, 'b> {
    has_once: bool,
    first_memo: Option<&'b DirectiveNode<'a>>,
    key_non_reactive: bool,
}

impl<'a, 'b> ScopeDirectives<'a, 'b> {
    pub(in crate::lower) fn new(inherited: bool) -> Self {
        Self {
            has_once: false,
            first_memo: None,
            key_non_reactive: inherited,
        }
    }

    pub(in crate::lower) fn observe(&mut self, dir: &'b DirectiveNode<'a>) {
        match dir.name {
            "once" => {
                self.has_once = true;
                self.key_non_reactive = true;
            }
            "memo" => {
                if self.first_memo.is_none() {
                    self.first_memo = Some(dir);
                }
                self.key_non_reactive |= matches!(dir.exp.as_ref(), Some(ExpressionNode::Simple(exp))
                    if exp.content.trim() == "[]");
            }
            _ => {}
        }
    }

    pub(in crate::lower) fn finish(self) -> (bool, Option<&'static str>, bool) {
        let (once, error) = if self.has_once {
            (true, None)
        } else if let Some(dir) = self.first_memo {
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
        (once, error, self.key_non_reactive)
    }
}

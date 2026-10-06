//! Existing once/memo directive classification.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode};
use vize_carton::String;

pub(in crate::lower) struct NonReactiveDirective {
    pub(in crate::lower) should_lower_as_once: bool,
    pub(in crate::lower) memo_error: Option<String>,
}

pub(in crate::lower) fn classify_non_reactive_directive(
    el: &ElementNode<'_>,
) -> NonReactiveDirective {
    let has_once = el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "once"));
    if has_once {
        return NonReactiveDirective {
            should_lower_as_once: true,
            memo_error: None,
        };
    }

    for prop in el.props.iter() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name != "memo" {
            continue;
        }

        let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref() else {
            return NonReactiveDirective {
                should_lower_as_once: false,
                memo_error: Some(vize_carton::String::from(
                    "v-memo is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
                )),
            };
        };

        if exp.content.trim() == "[]" {
            return NonReactiveDirective {
                should_lower_as_once: true,
                memo_error: None,
            };
        }

        return NonReactiveDirective {
            should_lower_as_once: false,
            memo_error: Some(vize_carton::String::from(
                "v-memo with dependencies is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
            )),
        };
    }

    NonReactiveDirective {
        should_lower_as_once: false,
        memo_error: None,
    }
}

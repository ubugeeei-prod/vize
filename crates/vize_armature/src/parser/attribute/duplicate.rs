//! Duplicate attribute and directive detection before transform lowering.

use vize_l0::{FxHashSet, String, appends};
use vize_relief::{
    ExpressionNode, PropNode,
    errors::{CompilerError, ErrorCode},
};

use super::super::{CurrentDirective, CurrentElement, Parser};

impl<'a> Parser<'a> {
    pub(in crate::parser) fn report_duplicate_attributes(&mut self, current: &CurrentElement<'a>) {
        let mut seen = (current.props.len() >= 12).then(FxHashSet::<String>::default);
        for (index, prop) in current.props.iter().enumerate() {
            let PropNode::Attribute(attr) = prop else {
                continue;
            };
            let duplicate = if let Some(seen) = seen.as_mut() {
                !seen.insert(attr.name.to_ascii_lowercase().into())
            } else {
                current.props[..index].iter().any(|earlier| {
                    matches!(earlier, PropNode::Attribute(existing) if existing.name.eq_ignore_ascii_case(attr.name))
                })
            };
            if !duplicate {
                continue;
            }
            // Keep both nodes for linters; compilation retains the first.
            let mut message = String::with_capacity(attr.name.len() + 79);
            appends!(
                message,
                "Duplicate attribute `",
                attr.name,
                "`. Keeping the repeated attribute so parsing can continue."
            );
            self.errors.push(CompilerError::with_message(
                ErrorCode::DuplicateAttribute,
                message,
                Some(attr.name_loc.clone()),
            ));
        }
    }

    pub(super) fn report_duplicate_directive(&mut self, directive: &CurrentDirective<'a>) {
        // Event listeners and argumentless object spreads may repeat.
        if directive.name == "on" || (directive.name == "bind" && directive.arg.is_none()) {
            return;
        }
        let argument = directive
            .arg
            .map(|(content, _, _, is_dynamic)| (content, !is_dynamic));
        let synthetic_prop = directive.raw_name.starts_with('.')
            && !directive
                .modifiers
                .iter()
                .any(|(content, _, _)| *content == "prop");
        let duplicate = self.current_element.as_ref().is_some_and(|current| {
            current.props.iter().any(|prop| {
                let PropNode::Directive(existing) = prop else {
                    return false;
                };
                let existing_argument = match existing.arg.as_ref() {
                    Some(ExpressionNode::Simple(argument)) => {
                        Some((argument.content, argument.is_static))
                    }
                    _ => None,
                };
                existing.raw_name == Some(directive.raw_name)
                    && existing_argument == argument
                    && existing
                        .modifiers
                        .iter()
                        .map(|modifier| modifier.content)
                        .eq(synthetic_prop
                            .then_some("prop")
                            .into_iter()
                            .chain(directive.modifiers.iter().map(|(content, _, _)| *content)))
            })
        });
        if duplicate {
            let loc = self.create_loc(directive.name_start, directive.name_end);
            self.errors
                .push(CompilerError::new(ErrorCode::DuplicateAttribute, Some(loc)));
        }
    }
}

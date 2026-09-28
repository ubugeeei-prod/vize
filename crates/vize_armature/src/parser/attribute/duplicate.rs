//! Duplicate attribute and directive detection before transform lowering.

use vize_relief::{
    ExpressionNode, PropNode,
    errors::{CompilerError, ErrorCode},
};

use super::super::{CurrentDirective, Parser};

impl<'a> Parser<'a> {
    pub(super) fn has_duplicate_attribute(&mut self, name: &str) -> bool {
        let Some(current) = self.current_element.as_ref() else {
            return false;
        };
        // Small tags avoid a hash table. Wide tags switch once to a set so
        // checking every attribute does not repeatedly scan all prior props.
        if current.props.len() < 12 {
            return current.props.iter().any(|prop| {
                matches!(prop, PropNode::Attribute(existing) if existing.name.eq_ignore_ascii_case(name))
            });
        }
        let seen = self.seen_attr_names.get_or_insert_with(|| {
            current
                .props
                .iter()
                .filter_map(|prop| match prop {
                    PropNode::Attribute(existing) => {
                        Some(existing.name.to_ascii_lowercase().into())
                    }
                    _ => None,
                })
                .collect()
        });
        !seen.insert(name.to_ascii_lowercase().into())
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

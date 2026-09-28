//! Duplicate attribute and directive detection before transform lowering.

use vize_l0::FxHashSet;
use vize_relief::{
    ExpressionNode, PropNode,
    errors::{CompilerError, ErrorCode},
};

use super::super::{CurrentDirective, Parser};

impl<'a> Parser<'a> {
    pub(super) fn has_duplicate_attribute(&mut self, name: &str) -> bool {
        let Some(current) = self.current_element.as_mut() else {
            return false;
        };
        if let Some(names) = current.seen_attribute_names.as_mut() {
            return !names.insert(name.to_ascii_lowercase().into());
        }
        if current.props.len() < 8 {
            return current.props.iter().any(|prop| {
                matches!(prop, PropNode::Attribute(existing) if existing.name.eq_ignore_ascii_case(name))
            });
        }
        let mut names = FxHashSet::default();
        names.reserve(current.props.len() + 1);
        for prop in &current.props {
            if let PropNode::Attribute(existing) = prop {
                names.insert(existing.name.to_ascii_lowercase().into());
            }
        }
        let duplicate = !names.insert(name.to_ascii_lowercase().into());
        current.seen_attribute_names = Some(names);
        duplicate
    }

    pub(super) fn report_duplicate_directive(&mut self, directive: &CurrentDirective<'a>) {
        // Event listeners and argumentless object spreads may repeat.
        if directive.name == "on" || (directive.name == "bind" && directive.arg.is_none()) {
            return;
        }
        let repeated_name = self.current_element.as_mut().is_some_and(|current| {
            if let Some(names) = current.seen_directive_names.as_mut() {
                return !names.insert(directive.raw_name);
            }
            if current.props.len() < 8 {
                return current.props.iter().any(|prop| {
                    matches!(prop, PropNode::Directive(existing) if existing.raw_name == Some(directive.raw_name))
                });
            }
            let mut names = FxHashSet::default();
            names.reserve(current.props.len() + 1);
            for prop in &current.props {
                if let PropNode::Directive(existing) = prop
                    && let Some(raw_name) = existing.raw_name
                {
                    names.insert(raw_name);
                }
            }
            let repeated = !names.insert(directive.raw_name);
            current.seen_directive_names = Some(names);
            repeated
        });
        if !repeated_name {
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

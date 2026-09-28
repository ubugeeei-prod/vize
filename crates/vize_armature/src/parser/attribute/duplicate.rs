//! Duplicate attribute and directive detection before transform lowering.

use vize_l0::FxHashSet;
use vize_relief::{
    ExpressionNode, PropNode,
    errors::{CompilerError, ErrorCode},
};

use super::super::{CurrentDirective, Parser};

impl<'a> Parser<'a> {
    #[inline]
    pub(super) fn check_duplicate_directive(&mut self, directive: &CurrentDirective<'a>) {
        let first = self.current_element.as_mut().is_some_and(|current| {
            if current.directive_name_count != 0 {
                return false;
            }
            current.directive_names[0] = Some((
                directive.raw_name,
                directive
                    .arg
                    .map(|(content, _, _, is_dynamic)| (content, !is_dynamic)),
            ));
            current.directive_name_count = 1;
            true
        });
        if !first {
            self.report_duplicate_directive(directive);
        }
    }

    pub(super) fn has_duplicate_attribute(&self, name: &str) -> bool {
        self.current_element.as_ref().is_some_and(|current| {
            current.props.iter().any(|prop| {
                matches!(prop, PropNode::Attribute(existing) if existing.name.eq_ignore_ascii_case(name))
            })
        })
    }

    pub(super) fn report_duplicate_directive(&mut self, directive: &CurrentDirective<'a>) {
        // Event listeners and argumentless object spreads may repeat.
        if directive.name == "on" || (directive.name == "bind" && directive.arg.is_none()) {
            return;
        }
        let argument = directive
            .arg
            .map(|(content, _, _, is_dynamic)| (content, !is_dynamic));
        let key = (directive.raw_name, argument);
        let repeated_name = self.current_element.as_mut().is_some_and(|current| {
            if current.directive_name_count < current.directive_names.len() {
                let repeated =
                    current.directive_names[..current.directive_name_count].contains(&Some(key));
                current.directive_names[current.directive_name_count] = Some(key);
                current.directive_name_count += 1;
                return repeated;
            }
            if let Some(names) = current.seen_directive_names.as_mut() {
                return !names.insert(key);
            }
            let mut names = FxHashSet::default();
            names.reserve(current.directive_name_count + 1);
            for &seen_key in current.directive_names.iter().flatten() {
                names.insert(seen_key);
            }
            let repeated = !names.insert(key);
            current.seen_directive_names = Some(names);
            repeated
        });
        if !repeated_name {
            return;
        }
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

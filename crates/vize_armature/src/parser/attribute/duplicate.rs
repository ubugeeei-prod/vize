//! Duplicate attribute and directive detection before transform lowering.

use vize_relief::{
    ExpressionNode, PropNode,
    errors::{CompilerError, ErrorCode},
};

use super::super::{CurrentDirective, Parser};

impl<'a> Parser<'a> {
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

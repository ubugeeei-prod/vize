//! Duplicate attribute and directive detection before transform lowering.

use vize_relief::{
    ExpressionNode, PropNode, SourceLocation,
    errors::{CompilerError, ErrorCode},
};

use super::super::{CurrentDirective, Parser};

impl<'a> Parser<'a> {
    pub(super) fn has_duplicate_attribute(&self, name: &str) -> bool {
        self.current_element.as_ref().is_some_and(|current| {
            current.props.iter().any(|prop| {
                matches!(
                    prop,
                    PropNode::Attribute(existing)
                        if existing.name.eq_ignore_ascii_case(name)
                )
            })
        })
    }

    pub(super) fn report_duplicate_directive(
        &mut self,
        directive: &CurrentDirective<'a>,
        loc: SourceLocation,
    ) {
        let argument = directive.arg.map(|(content, _, _, _)| content);
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
                    Some(ExpressionNode::Simple(argument)) => Some(argument.content),
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
            self.errors
                .push(CompilerError::new(ErrorCode::DuplicateAttribute, Some(loc)));
        }
    }
}

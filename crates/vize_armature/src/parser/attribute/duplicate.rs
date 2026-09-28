//! Duplicate attribute and directive detection before transform lowering.

use vize_relief::{
    PropNode, SourceLocation,
    errors::{CompilerError, ErrorCode},
};

use super::super::Parser;

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

    pub(super) fn report_duplicate_directive(&mut self, raw_name: &str, loc: SourceLocation) {
        let duplicate = self.current_element.as_ref().is_some_and(|current| {
            current.props.iter().any(|prop| {
                matches!(prop, PropNode::Directive(existing) if existing.raw_name == Some(raw_name))
            })
        });
        if duplicate {
            self.errors
                .push(CompilerError::new(ErrorCode::DuplicateAttribute, Some(loc)));
        }
    }
}

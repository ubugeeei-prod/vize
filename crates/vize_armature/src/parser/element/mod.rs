//! Element processing methods for the parser.
//!
//! Handles text, interpolation, open/close tags, element type determination,
//! comments, and error reporting.

mod classify;
mod comment;
mod errors;
mod formatting;
mod scope;
mod table;
mod tags;
mod text;

use vize_relief::{
    SourceLocation,
    errors::{CompilerError, ErrorCode},
};

use super::Parser;

impl<'a> Parser<'a> {
    fn report_tree_construction_recovery(&mut self, loc: &SourceLocation, message: &str) {
        self.errors.push(CompilerError::with_message(
            ErrorCode::ExtendPoint,
            message,
            Some(loc.clone()),
        ));
    }

    fn tag_in(tag: &str, candidates: &[&str]) -> bool {
        candidates
            .iter()
            .any(|candidate| tag.eq_ignore_ascii_case(candidate))
    }
}

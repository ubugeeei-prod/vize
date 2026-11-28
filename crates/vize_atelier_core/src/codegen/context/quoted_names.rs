//! Authored names written between quotes: prop keys, event keys, dynamic
//! prop names, directive arguments and modifiers. A name is final text, so a
//! name with a backslash is spelled through the string-literal writer; every
//! other name is copied as it is.

use vize_l0::Span;
use vize_l0::general::quoted_name::{escaped_name_contents, name_needs_escape};

use super::CodegenContext;
use crate::codegen::helpers::is_valid_js_identifier;

impl CodegenContext {
    /// Push `name` as a double-quoted JavaScript string literal.
    #[inline(always)]
    pub(in crate::codegen) fn push_string_literal(&mut self, name: &str) {
        if name_needs_escape(name) {
            self.push_escaped_string_literal(name);
        } else {
            self.push("\"");
            self.push(name);
            self.push("\"");
        }
    }

    #[cold]
    #[inline(never)]
    fn push_escaped_string_literal(&mut self, name: &str) {
        self.push("\"");
        self.push(&escaped_name_contents(name));
        self.push("\"");
    }

    /// Push `key` as an object key: bare when it is an identifier, otherwise
    /// as the string literal that spells it.
    #[inline(always)]
    pub(in crate::codegen) fn push_key(&mut self, key: &str) {
        if is_valid_js_identifier(key) {
            self.push(key);
        } else {
            self.push_string_literal(key);
        }
    }

    /// [`Self::push_key`], with the key generated from the authored range
    /// `span` that spells the symbol `name`.
    #[inline(always)]
    pub(in crate::codegen) fn push_named_key(&mut self, key: &str, span: Span, name: &str) {
        let needs_quotes = !is_valid_js_identifier(key);
        if needs_quotes && name_needs_escape(key) {
            self.push_escaped_named_key(key, span, name);
            return;
        }
        if needs_quotes {
            self.push("\"");
        }
        self.push_named(key, span, name);
        if needs_quotes {
            self.push("\"");
        }
    }

    #[cold]
    #[inline(never)]
    fn push_escaped_named_key(&mut self, key: &str, span: Span, name: &str) {
        self.push("\"");
        self.push_named(&escaped_name_contents(key), span, name);
        self.push("\"");
    }
}

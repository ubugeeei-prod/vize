//! Authored names written between quotes: prop keys, event keys, dynamic
//! prop names, directive arguments and modifiers. A name is final text, so it
//! is spelled as a JavaScript string literal. Whether any name of this
//! compile can need escaping is decided once, from its source, the first time
//! a name is quoted; until a source holds a backslash, names are copied as
//! they are.

use vize_l0::general::quoted_name::{escaped_name_contents, source_may_need_name_escapes};
use vize_l0::{Span, String};

use super::CodegenContext;
use crate::codegen::helpers::is_valid_js_identifier;

impl CodegenContext {
    /// Whether a quoted name of this compile may need escaping.
    #[inline]
    fn names_may_need_escapes(&self) -> bool {
        match self.quoted_name_escapes.get() {
            Some(escapes) => escapes,
            None => self.scan_source_for_name_escapes(),
        }
    }

    #[cold]
    #[inline(never)]
    fn scan_source_for_name_escapes(&self) -> bool {
        let escapes = source_may_need_name_escapes(&self.source);
        self.quoted_name_escapes.set(Some(escapes));
        escapes
    }

    /// Push `name` as a double-quoted JavaScript string literal.
    #[inline]
    pub(in crate::codegen) fn push_string_literal(&mut self, name: &str) {
        if self.names_may_need_escapes() {
            self.push_escaped_string_literal(name);
            return;
        }
        self.push("\"");
        self.push(name);
        self.push("\"");
    }

    #[cold]
    #[inline(never)]
    fn push_escaped_string_literal(&mut self, name: &str) {
        let contents = escaped_name_contents(name);
        self.push("\"");
        self.push(contents.as_deref().unwrap_or(name));
        self.push("\"");
    }

    /// Push `key` as an object key: bare when it is an identifier, otherwise
    /// as the string literal that spells it.
    #[inline]
    pub(in crate::codegen) fn push_key(&mut self, key: &str) {
        if is_valid_js_identifier(key) {
            self.push(key);
        } else {
            self.push_string_literal(key);
        }
    }

    /// [`Self::push_key`], with the key generated from the authored range
    /// `span` that spells the symbol `name`.
    #[inline]
    pub(in crate::codegen) fn push_named_key(&mut self, key: &str, span: Span, name: &str) {
        if is_valid_js_identifier(key) {
            self.push_named(key, span, name);
            return;
        }
        if self.names_may_need_escapes() {
            self.push_escaped_named_key(key, span, name);
            return;
        }
        self.push("\"");
        self.push_named(key, span, name);
        self.push("\"");
    }

    #[cold]
    #[inline(never)]
    fn push_escaped_named_key(&mut self, key: &str, span: Span, name: &str) {
        let contents = escaped_name_contents(key);
        self.push("\"");
        self.push_named(contents.as_deref().unwrap_or(key), span, name);
        self.push("\"");
    }

    /// Append `key` to `out` as an object key, as [`Self::push_key`] does.
    #[inline]
    pub(in crate::codegen) fn write_key(&self, out: &mut String, key: &str) {
        if is_valid_js_identifier(key) {
            out.push_str(key);
            return;
        }
        let contents = if self.names_may_need_escapes() {
            escaped_name_contents(key)
        } else {
            None
        };
        out.push('"');
        out.push_str(contents.as_deref().unwrap_or(key));
        out.push('"');
    }
}

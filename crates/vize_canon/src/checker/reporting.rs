//! Type reporting, completion, and utility methods for the TypeChecker.
//!
//! Contains methods for type lookup at positions, completion generation,
//! identifier extraction, and helper predicates.

use crate::{
    context::TypeContext,
    types::{CompletionItem, CompletionKind, TypeInfo},
};

use super::runner::TypeChecker;
use vize_carton::cstr;
use vize_carton::String;

impl TypeChecker {
    /// Get type information at a specific offset.
    ///
    /// Returns the type of the expression or identifier at the given position.
    pub fn get_type_at(&self, template: &str, offset: u32, ctx: &TypeContext) -> Option<TypeInfo> {
        // Find what's at the offset
        let offset = offset as usize;

        // Check if we're in an interpolation
        if let Some((expr, expr_start)) = self.find_expression_at(template, offset) {
            let relative_offset = offset - expr_start;
            return self.get_type_in_expression(&expr, relative_offset, ctx);
        }

        None
    }

    /// Find the expression containing the given offset.
    fn find_expression_at(&self, template: &str, offset: usize) -> Option<(String, usize)> {
        // Check interpolations
        let mut pos = 0;
        while let Some(start) = template[pos..].find("{{") {
            let abs_start = pos + start;
            if let Some(end) = template[abs_start..].find("}}") {
                let expr_start = abs_start + 2;
                let expr_end = abs_start + end;

                if offset >= expr_start && offset <= expr_end {
                    return Some((template[expr_start..expr_end].trim().into(), expr_start));
                }

                pos = abs_start + end + 2;
            } else {
                break;
            }
        }

        // Check directive values
        for directive in ["v-if", "v-else-if", "v-show", "v-for"] {
            if let Some((expr, start)) = self.find_directive_expr_at(template, directive, offset) {
                return Some((expr, start));
            }
        }

        None
    }

    /// Find a directive expression at offset.
    fn find_directive_expr_at(
        &self,
        template: &str,
        directive: &str,
        offset: usize,
    ) -> Option<(String, usize)> {
        let pattern = cstr!("{directive}=\"");
        let mut pos = 0;

        while let Some(start) = template[pos..].find(pattern.as_str()) {
            let abs_start = pos + start + pattern.len();
            if let Some(end) = template[abs_start..].find('"') {
                if offset >= abs_start && offset <= abs_start + end {
                    return Some((template[abs_start..abs_start + end].into(), abs_start));
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }

        None
    }

    /// Get type information within an expression.
    fn get_type_in_expression(
        &self,
        expr: &str,
        offset: usize,
        ctx: &TypeContext,
    ) -> Option<TypeInfo> {
        // Find the identifier at the offset
        let ident = self.find_identifier_at(expr, offset)?;

        // Look up the type
        if let Some(binding) = ctx.get_binding(&ident) {
            return Some(binding.type_info.clone());
        }

        if let Some(type_info) = ctx.globals.get(&ident) {
            return Some(type_info.clone());
        }

        None
    }

    /// Find identifier at offset within an expression.
    fn find_identifier_at(&self, expr: &str, offset: usize) -> Option<String> {
        if offset >= expr.len() {
            return None;
        }

        let bytes = expr.as_bytes();

        // Check if we're on an identifier character
        if !Self::is_ident_char(bytes[offset] as char) {
            return None;
        }

        // Find the start of the identifier
        let mut start = offset;
        while start > 0 && Self::is_ident_char(bytes[start - 1] as char) {
            start -= 1;
        }

        // Find the end of the identifier
        let mut end = offset;
        while end < bytes.len() && Self::is_ident_char(bytes[end] as char) {
            end += 1;
        }

        // First char must be a valid start char
        if !Self::is_ident_start(bytes[start] as char) {
            return None;
        }

        Some(expr[start..end].into())
    }

    /// Get completions at a specific offset.
    pub fn get_completions(
        &self,
        _template: &str,
        _offset: u32,
        ctx: &TypeContext,
    ) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add all bindings as completions
        for (name, binding) in &ctx.bindings {
            let kind = match binding.kind {
                crate::context::BindingKind::Function => CompletionKind::Function,
                crate::context::BindingKind::Class => CompletionKind::Class,
                crate::context::BindingKind::Const
                | crate::context::BindingKind::Let
                | crate::context::BindingKind::Var => CompletionKind::Variable,
                crate::context::BindingKind::Ref
                | crate::context::BindingKind::Computed
                | crate::context::BindingKind::Reactive => CompletionKind::Variable,
                crate::context::BindingKind::Import => CompletionKind::Module,
                crate::context::BindingKind::Prop => CompletionKind::Property,
                _ => CompletionKind::Variable,
            };

            completions.push(
                CompletionItem::new(name.as_str(), kind)
                    .with_detail(binding.type_info.display.as_str())
                    .with_priority(10),
            );
        }

        // Add components
        for name in ctx.components.keys() {
            completions.push(
                CompletionItem::new(name.as_str(), CompletionKind::Component).with_priority(20),
            );
        }

        // Add globals
        for (name, type_info) in &ctx.globals {
            completions.push(
                CompletionItem::new(name.as_str(), CompletionKind::Variable)
                    .with_detail(type_info.display.as_str())
                    .with_priority(30),
            );
        }

        // Sort by priority then name
        completions.sort_by(|a, b| {
            a.sort_priority
                .cmp(&b.sort_priority)
                .then_with(|| a.label.cmp(&b.label))
        });

        completions
    }

    /// Extract identifiers from an expression.
    pub(crate) fn extract_identifiers(expr: &str) -> Vec<(&str, usize)> {
        let mut identifiers = Vec::new();
        let bytes = expr.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Skip non-identifier characters
            while i < bytes.len() && !Self::is_ident_start(bytes[i] as char) {
                i += 1;
            }

            if i >= bytes.len() {
                break;
            }

            let start = i;

            // Read the identifier
            while i < bytes.len() && Self::is_ident_char(bytes[i] as char) {
                i += 1;
            }

            if start < i {
                identifiers.push((&expr[start..i], start));
            }
        }

        identifiers
    }

    /// Check if a string is a simple identifier (no dots, brackets, etc.)
    pub(crate) fn is_simple_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        let first = chars.next().expect("non-empty string checked above");

        if !Self::is_ident_start(first) {
            return false;
        }

        chars.all(Self::is_ident_char)
    }

    /// Check if a character can start an identifier.
    pub(crate) fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    }

    /// Check if a character can be part of an identifier.
    pub(crate) fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    }

    /// Check if a string is a keyword or literal.
    pub(crate) fn is_keyword_or_literal(s: &str) -> bool {
        matches!(
            s,
            "true"
                | "false"
                | "null"
                | "undefined"
                | "this"
                | "if"
                | "else"
                | "for"
                | "while"
                | "do"
                | "switch"
                | "case"
                | "default"
                | "break"
                | "continue"
                | "return"
                | "throw"
                | "try"
                | "catch"
                | "finally"
                | "new"
                | "delete"
                | "typeof"
                | "instanceof"
                | "in"
                | "of"
                | "void"
                | "function"
                | "class"
                | "extends"
                | "const"
                | "let"
                | "var"
                | "import"
                | "export"
                | "async"
                | "await"
                | "yield"
        )
    }
}

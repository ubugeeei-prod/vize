//! Binding location types and extraction utilities.
//!
//! Provides types for describing where bindings are defined and
//! utilities for extracting binding locations from script content.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

/// Location of a binding definition.
#[derive(Debug, Clone)]
pub struct BindingLocation {
    /// The binding name.
    pub name: String,
    /// Byte offset in the content.
    pub offset: usize,
    /// Kind of binding.
    pub kind: BindingKind,
}

/// Kind of binding definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// const declaration
    Const,
    /// let declaration
    Let,
    /// var declaration
    Var,
    /// function declaration
    Function,
    /// Destructuring pattern
    Destructure,
    /// Import binding
    Import,
    /// Unknown
    Unknown,
}

impl BindingKind {
    pub(crate) fn from_pattern(pattern: &str) -> Self {
        if pattern.starts_with("const") {
            BindingKind::Const
        } else if pattern.starts_with("let") {
            BindingKind::Let
        } else if pattern.starts_with("var") {
            BindingKind::Var
        } else if pattern.starts_with("function") {
            BindingKind::Function
        } else {
            BindingKind::Unknown
        }
    }
}

/// Extract bindings with their locations from script content.
pub fn extract_bindings_with_locations(content: &str, is_setup: bool) -> Vec<BindingLocation> {
    let mut bindings = Vec::new();

    if !is_setup {
        return bindings;
    }

    let content_start = super::helpers::skip_virtual_header(content);
    let search_content = &content[content_start..];

    for line in search_content.lines() {
        let trimmed = line.trim();
        let line_start = search_content[..search_content.find(line).unwrap_or(0)].len();

        // const/let/var declarations
        for keyword in &["const ", "let ", "var "] {
            if trimmed.starts_with(keyword) {
                if let Some(rest) = trimmed.strip_prefix(keyword) {
                    // Handle destructuring: { a, b }
                    if rest.starts_with('{') {
                        if let Some(end) = rest.find('}') {
                            let inner = &rest[1..end];
                            for part in inner.split(',') {
                                let name = part.split(':').next().unwrap_or("").trim();
                                if !name.is_empty() && is_valid_identifier(name) {
                                    if let Some(name_pos) = line.find(name) {
                                        bindings.push(BindingLocation {
                                            name: name.to_string(),
                                            offset: content_start + line_start + name_pos,
                                            kind: BindingKind::Destructure,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Simple: const x = ...
                    else if let Some(name) = rest.split(['=', ':', ' ']).next() {
                        let name = name.trim();
                        if is_valid_identifier(name) {
                            if let Some(name_pos) = line.find(name) {
                                let kind = match *keyword {
                                    "const " => BindingKind::Const,
                                    "let " => BindingKind::Let,
                                    "var " => BindingKind::Var,
                                    _ => BindingKind::Unknown,
                                };
                                bindings.push(BindingLocation {
                                    name: name.to_string(),
                                    offset: content_start + line_start + name_pos,
                                    kind,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Function declarations
        if trimmed.starts_with("function ") {
            if let Some(rest) = trimmed.strip_prefix("function ") {
                if let Some(name) = rest.split('(').next() {
                    let name = name.trim();
                    if is_valid_identifier(name) {
                        if let Some(name_pos) = line.find(name) {
                            bindings.push(BindingLocation {
                                name: name.to_string(),
                                offset: content_start + line_start + name_pos,
                                kind: BindingKind::Function,
                            });
                        }
                    }
                }
            }
        }
    }

    bindings
}

/// Check if a string is a valid JavaScript identifier.
pub(crate) fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}
